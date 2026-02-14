import type {
  DownscaleSharpenOptions,
  GenerateResult,
  Origin,
  ResizeFilter,
  ResizeMode,
  ResizeResult,
  TileFormat,
} from './protocol'
import type {
  WorkerGenerateParams,
  WorkerProgressMessage,
  WorkerResizeParams,
  WorkerTask,
  WorkerTaskParams,
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
  /** Whether to auto-orient input image pixels using EXIF orientation metadata. */
  autoOrient?: boolean
  /** Progress callback. */
  onProgress?: (current: number, total: number, message: string) => void
}

/**
 * Parameters for resizing an image (without tiling).
 */
export interface ResizeParams {
  /** Input file path. */
  input: string
  /** Output file path. */
  output: string
  /** Resize mode (percentage, longEdge, or fit). */
  mode: ResizeMode
  /** Output format. */
  format?: TileFormat
  /** Whether to auto-orient input image pixels using EXIF orientation metadata. */
  autoOrient?: boolean
  /** Resize filter for downscaling. */
  resizeFilter?: ResizeFilter
  /** Sharpening configuration for downscaling. */
  sharpen?: DownscaleSharpenOptions
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
  /** Add a generate task into the queue. */
  add: (task: GenerateParams) => void
  /** Add a resize task into the queue. */
  addResize: (task: ResizeParams) => void
  /** Run queued tasks and return ordered results (GenerateResult or ResizeResult). */
  run: () => Promise<(GenerateResult | ResizeResult)[]>
  /** Run queued generate tasks and return ordered results. */
  runGenerate: () => Promise<GenerateResult[]>
  /** Run queued resize tasks and return ordered results. */
  runResize: () => Promise<ResizeResult[]>
}

/**
 * Create a new image map task pool.
 */
export function createPool(options: PoolOptions = {}): ImageMapPool {
  return new Pool(options)
}

/** Task type for internal queue management. */
type QueuedTask
  = | { type: 'generate', params: GenerateParams }
    | { type: 'resize', params: ResizeParams }

/**
 * Default implementation backed by Tinypool.
 */
class Pool implements ImageMapPool {
  private readonly concurrency: number
  private readonly queue: QueuedTask[] = []

  /**
   * Create a pool with configured concurrency.
   */
  constructor(options: PoolOptions) {
    this.concurrency = Math.max(1, options.concurrency ?? 1)
  }

  /** Add a generate task into the pool queue. */
  add(task: GenerateParams) {
    this.queue.push({ type: 'generate', params: task })
  }

  /** Add a resize task into the pool queue. */
  addResize(task: ResizeParams) {
    this.queue.push({ type: 'resize', params: task })
  }

  /** Execute all queued tasks with a Tinypool worker pool. */
  async run(): Promise<(GenerateResult | ResizeResult)[]> {
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
      const results: (GenerateResult | ResizeResult)[] = Array.from({ length: tasks.length })
      await Promise.all(
        tasks.map((task, index) => this.runTask(pool, task, index, results)),
      )
      return results
    }
    finally {
      await pool.destroy()
    }
  }

  /** Execute all queued generate tasks only. */
  async runGenerate(): Promise<GenerateResult[]> {
    const generateTasks = this.queue.filter(t => t.type === 'generate')
    this.queue.length = 0
    generateTasks.forEach(t => this.queue.push(t))
    return await this.run() as GenerateResult[]
  }

  /** Execute all queued resize tasks only. */
  async runResize(): Promise<ResizeResult[]> {
    const resizeTasks = this.queue.filter(t => t.type === 'resize')
    this.queue.length = 0
    resizeTasks.forEach(t => this.queue.push(t))
    return await this.run() as ResizeResult[]
  }

  /**
   * Run a single task in Tinypool and store its result.
   */
  private async runTask(
    pool: Tinypool,
    task: QueuedTask,
    index: number,
    results: (GenerateResult | ResizeResult)[],
  ): Promise<void> {
    const payload: WorkerTask = {
      params: toWorkerParams(task),
    }

    const onProgress = task.params.onProgress
    if (!onProgress) {
      results[index] = await pool.run(payload)
      return
    }

    const { port1, port2 } = new MessageChannel()
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
function toWorkerParams(task: QueuedTask): WorkerTaskParams {
  if (task.type === 'generate') {
    const { onProgress: _onProgress, ...rest } = task.params
    return { type: 'generate', ...rest } as WorkerGenerateParams
  }
  else {
    const { onProgress: _onProgress, ...rest } = task.params
    return { type: 'resize', ...rest } as WorkerResizeParams
  }
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
