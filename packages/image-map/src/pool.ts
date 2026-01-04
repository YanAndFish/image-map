import type {
  DownscaleSharpenOptions,
  GenerateResult,
  Origin,
  ResizeFilter,
  TileFormat,
} from './protocol'
import type {
  WorkerParams,
  WorkerProgressMessage,
  WorkerTask,
} from './worker-protocol'

import fs from 'node:fs'
import { fileURLToPath } from 'node:url'
import { MessageChannel } from 'node:worker_threads'

import Tinypool from 'tinypool'

/**
 * Parameters for generating tiles.
 */
export interface GenerateParams {
  /** Input file path. */
  input: string
  /** Output directory path. */
  output: string
  /** Tile size in pixels. */
  tileSize?: number
  /** Resize filter for building lower zoom levels. */
  resizeFilter?: ResizeFilter
  /** Downscale sharpening configuration. */
  downscaleSharpen?: DownscaleSharpenOptions
  /** Output formats. */
  formats?: TileFormat[]
  /** Origin position. */
  origin?: Origin
  /** Minimum zoom level. */
  minZoom?: number
  /** Maximum zoom level. */
  maxZoom?: number
  /** Progress callback. */
  onProgress?: (current: number, total: number, message: string) => void
}

/**
 * Pool configuration options.
 */
export interface PoolOptions {
  /** Maximum concurrent workers. */
  concurrency?: number
}

/**
 * Image map task pool.
 */
export interface ImageMapPool {
  /** Add a task into the queue. */
  add: (task: GenerateParams) => void
  /** Run queued tasks and return ordered results. */
  run: () => Promise<GenerateResult[]>
}

/**
 * Create a new image map task pool.
 */
export function createPool(options: PoolOptions = {}): ImageMapPool {
  return new Pool(options)
}

/**
 * Default implementation backed by Tinypool.
 */
class Pool implements ImageMapPool {
  private readonly concurrency: number
  private readonly queue: GenerateParams[] = []

  /**
   * Create a pool with configured concurrency.
   */
  constructor(options: PoolOptions) {
    this.concurrency = Math.max(1, options.concurrency ?? 1)
  }

  /** Add a task into the pool queue. */
  add(task: GenerateParams) {
    this.queue.push(task)
  }

  /** Execute all queued tasks with a Tinypool worker pool. */
  async run(): Promise<GenerateResult[]> {
    const tasks = this.queue.splice(0, this.queue.length)
    if (tasks.length === 0)
      return []

    const threadCount = Math.min(this.concurrency, tasks.length)
    const pool = new Tinypool({
      filename: resolveWorkerUrl(),
      minThreads: threadCount,
      maxThreads: threadCount,
      concurrentTasksPerWorker: 1,
      teardown: 'teardown',
    })

    try {
      const results: GenerateResult[] = Array.from({ length: tasks.length })
      await Promise.all(
        tasks.map((task, index) => this.runTask(pool, task, index, results)),
      )
      return results
    }
    finally {
      await pool.destroy()
    }
  }

  /**
   * Run a single task in Tinypool and store its result.
   */
  private async runTask(
    pool: Tinypool,
    task: GenerateParams,
    index: number,
    results: GenerateResult[],
  ): Promise<void> {
    const payload: WorkerTask = {
      params: toWorkerParams(task),
    }

    if (!task.onProgress) {
      results[index] = await pool.run(payload)
      return
    }

    const { port1, port2 } = new MessageChannel()
    const onProgress = task.onProgress
    payload.port = port1

    const handleMessage = (message: unknown) => {
      if (!isWorkerProgressMessage(message))
        return
      onProgress(message.current, message.total, message.message)
    }

    port2.on('message', handleMessage)
    try {
      results[index] = await pool.run(payload, {
        transferList: {
          transfer: [port1],
        },
      })
    }
    finally {
      port2.off('message', handleMessage)
      port2.close()
    }
  }
}

/**
 * Remove non-serializable fields from params before sending to workers.
 */
function toWorkerParams(params: GenerateParams): WorkerParams {
  const { onProgress: _onProgress, ...rest } = params
  return rest
}

/**
 * Resolve the worker entry url, preferring the built .mjs output.
 */
function resolveWorkerUrl(): string {
  const mjsUrl = new URL('./pool-worker.mjs', import.meta.url)
  if (fs.existsSync(fileURLToPath(mjsUrl)))
    return mjsUrl.href

  const tsUrl = new URL('./pool-worker.ts', import.meta.url)
  if (fs.existsSync(fileURLToPath(tsUrl)))
    return tsUrl.href

  return mjsUrl.href
}

/**
 * Narrow unknown messages into worker progress messages.
 */
function isWorkerProgressMessage(value: unknown): value is WorkerProgressMessage {
  if (!value || typeof value !== 'object')
    return false

  const msg = value as WorkerProgressMessage
  if (msg.type !== 'progress')
    return false

  return (
    typeof msg.current === 'number'
    && typeof msg.total === 'number'
    && typeof msg.message === 'string'
  )
}
