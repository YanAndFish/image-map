import type {
  GenerateOptions,
  GenerateResult,
  Origin,
  RequestMessage,
  ResponseMessage,
  TileFormat,
} from './protocol'
import { spawn } from 'node:child_process'
import { randomUUID } from 'node:crypto'
import { once } from 'node:events'

import readline from 'node:readline'
import { resolveBinaryPath } from './binary'

export interface GenerateParams {
  input: string
  output: string
  tileSize?: number
  formats?: TileFormat[]
  origin?: Origin
  minZoom?: number
  maxZoom?: number
  onProgress?: (current: number, total: number, message: string) => void
}

export interface PoolOptions {
  concurrency?: number
}

export interface ImageMapPool {
  add: (task: GenerateParams) => void
  run: () => Promise<GenerateResult[]>
}

export function createPool(options: PoolOptions = {}): ImageMapPool {
  return new Pool(options)
}

class Pool implements ImageMapPool {
  private readonly concurrency: number
  private readonly queue: GenerateParams[] = []

  constructor(options: PoolOptions) {
    this.concurrency = Math.max(1, options.concurrency ?? 1)
  }

  add(task: GenerateParams) {
    this.queue.push(task)
  }

  async run(): Promise<GenerateResult[]> {
    const tasks = this.queue.splice(0, this.queue.length)
    if (tasks.length === 0)
      return []

    const workers = Array.from({ length: Math.min(this.concurrency, tasks.length) }, () => new ImageMapWorker())
    try {
      const results: GenerateResult[] = Array.from({ length: tasks.length })
      let nextIndex = 0

      await Promise.all(
        workers.map(async (worker) => {
          // Each worker processes tasks sequentially.
          while (true) {
            const i = nextIndex++
            if (i >= tasks.length)
              return
            results[i] = await worker.generate(tasks[i])
          }
        }),
      )

      return results
    }
    finally {
      await Promise.allSettled(workers.map(w => w.dispose()))
    }
  }
}

interface PendingTask {
  onProgress?: (current: number, total: number, message: string) => void
  resolve: (result: GenerateResult) => void
  reject: (error: Error) => void
}

class ImageMapWorker {
  private readonly child = spawn(resolveBinaryPath(), [], {
    stdio: ['pipe', 'pipe', 'pipe'],
  })

  private readonly pending = new Map<string, PendingTask>()

  private last: Promise<unknown> = Promise.resolve()
  private disposed = false

  constructor() {
    const rl = readline.createInterface({ input: this.child.stdout })
    rl.on('line', (line) => {
      this.handleLine(line)
    })

    const failAll = (err: Error) => {
      for (const [id, task] of this.pending) {
        this.pending.delete(id)
        task.reject(err)
      }
    }

    this.child.on('error', (err) => {
      failAll(err instanceof Error ? err : new Error(String(err)))
    })

    this.child.on('exit', (code, signal) => {
      const msg = `image-map worker exited (code=${code ?? 'null'}, signal=${signal ?? 'null'})`
      failAll(new Error(msg))
    })
  }

  generate(params: GenerateParams): Promise<GenerateResult> {
    const task = this.last.then(() => this.generateImpl(params))
    // Keep the queue alive even if a task fails.
    this.last = task.then(
      () => undefined,
      () => undefined,
    )
    return task
  }

  async dispose(): Promise<void> {
    if (this.disposed)
      return
    this.disposed = true

    try {
      this.child.stdin.end()
    }
    catch {}

    if (!this.child.killed)
      this.child.kill()

    try {
      await once(this.child, 'exit')
    }
    catch {}
  }

  private generateImpl(params: GenerateParams): Promise<GenerateResult> {
    if (this.disposed)
      return Promise.reject(new Error('image-map worker is disposed'))

    const id = randomUUID()
    const request: RequestMessage = {
      type: 'generate',
      id,
      input: params.input,
      output: params.output,
      options: normalizeOptions(params),
    }

    return new Promise<GenerateResult>((resolve, reject) => {
      this.pending.set(id, {
        onProgress: params.onProgress,
        resolve,
        reject,
      })

      const line = `${JSON.stringify(request)}\n`
      this.child.stdin.write(line, (err) => {
        if (!err)
          return
        this.pending.delete(id)
        reject(err instanceof Error ? err : new Error(String(err)))
      })
    })
  }

  private handleLine(line: string) {
    const trimmed = line.trim()
    if (!trimmed)
      return

    let msg: ResponseMessage
    try {
      msg = JSON.parse(trimmed) as ResponseMessage
    }
    catch {
      return
    }

    if (!msg || typeof msg !== 'object')
      return

    if (!('id' in msg) || typeof (msg as any).id !== 'string')
      return

    const id = (msg as any).id as string
    const pending = this.pending.get(id)
    if (!pending)
      return

    if (msg.type === 'progress') {
      pending.onProgress?.(msg.current, msg.total, msg.message)
      return
    }

    this.pending.delete(id)

    if (msg.type === 'complete') {
      pending.resolve(msg.result)
      return
    }

    if (msg.type === 'error') {
      pending.reject(new Error(msg.error))
    }
  }
}

function normalizeOptions(params: GenerateParams): GenerateOptions {
  const tileSize = params.tileSize ?? 256
  const formats = params.formats ?? ['webp']
  const origin = params.origin ?? 'topLeft'
  const minZoom = params.minZoom ?? 0
  const maxZoom = params.maxZoom ?? 0

  if (!Number.isFinite(tileSize) || tileSize <= 0)
    throw new Error('tileSize must be a positive number')

  if (formats.length === 0)
    throw new Error('formats must contain at least one format')

  if (minZoom < 0 || maxZoom < 0)
    throw new Error('minZoom/maxZoom must be >= 0')

  if (minZoom > maxZoom)
    throw new Error('minZoom must be <= maxZoom')

  return {
    tileSize,
    formats,
    origin,
    minZoom,
    maxZoom,
  }
}
