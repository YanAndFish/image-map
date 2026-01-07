import type {
  DownscaleSharpenOptions,
  GenerateOptions,
  GenerateResult,
  RequestMessage,
  ResizeFilter,
  ResizeImageOptions,
  ResizeResult,
  ResponseMessage,
  TileFormat,
} from './protocol'
import type {
  WorkerGenerateParams,
  WorkerProgressMessage,
  WorkerResizeParams,
  WorkerTask,
} from './worker-protocol'

import { spawn } from 'node:child_process'
import { once } from 'node:events'
import readline from 'node:readline'

import { nanoid } from 'nanoid'

import { resolveBinaryPath } from './binary'

/** Union result type for generate/resize tasks. */
type TaskResult = GenerateResult | ResizeResult

/**
 * Internal tracking for an in-flight task.
 */
interface PendingTask {
  /** Task type for resolving the correct response type. */
  taskType: 'generate' | 'resize'
  /** Progress channel for this task. */
  port?: WorkerTask['port']
  /** Resolve callback. */
  resolve: (result: TaskResult) => void
  /** Reject callback. */
  reject: (error: Error) => void
}

/**
 * Worker process wrapper that talks to the Rust binary.
 */
class ImageMapWorker {
  private readonly child = spawn(resolveBinaryPath(), [], {
    stdio: ['pipe', 'pipe', 'pipe'],
  })

  private readonly pending = new Map<string, PendingTask>()
  private last: Promise<unknown> = Promise.resolve()
  private disposed = false

  /**
   * Create a worker process and hook into its output.
   */
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

  /**
   * Enqueue a generate request on the underlying binary.
   */
  generate(params: Omit<WorkerGenerateParams, 'type'>, port?: WorkerTask['port']): Promise<GenerateResult> {
    const task = this.last.then(() => this.generateImpl(params, port))
    // Keep the queue alive even if a task fails.
    this.last = task.then(
      () => undefined,
      () => undefined,
    )
    return task
  }

  /**
   * Enqueue a resize request on the underlying binary.
   */
  resize(params: Omit<WorkerResizeParams, 'type'>, port?: WorkerTask['port']): Promise<ResizeResult> {
    const task = this.last.then(() => this.resizeImpl(params, port))
    // Keep the queue alive even if a task fails.
    this.last = task.then(
      () => undefined,
      () => undefined,
    )
    return task
  }

  /**
   * Dispose the child process and clear pending tasks.
   */
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

  /**
   * Send a generate request to the Rust binary and await the response.
   */
  private generateImpl(params: Omit<WorkerGenerateParams, 'type'>, port?: WorkerTask['port']): Promise<GenerateResult> {
    if (this.disposed)
      return Promise.reject(new Error('image-map worker is disposed'))

    const id = nanoid()
    const request: RequestMessage = {
      type: 'generate',
      id,
      input: params.input,
      output: params.output,
      options: normalizeGenerateOptions(params),
    }

    return new Promise<GenerateResult>((resolve, reject) => {
      this.pending.set(id, {
        taskType: 'generate',
        port,
        resolve: resolve as (result: TaskResult) => void,
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

  /**
   * Send a resize request to the Rust binary and await the response.
   */
  private resizeImpl(params: Omit<WorkerResizeParams, 'type'>, port?: WorkerTask['port']): Promise<ResizeResult> {
    if (this.disposed)
      return Promise.reject(new Error('image-map worker is disposed'))

    const id = nanoid()
    const request: RequestMessage = {
      type: 'resize',
      id,
      input: params.input,
      output: params.output,
      options: normalizeResizeOptions(params),
    }

    return new Promise<ResizeResult>((resolve, reject) => {
      this.pending.set(id, {
        taskType: 'resize',
        port,
        resolve: resolve as (result: TaskResult) => void,
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

  /**
   * Handle one line of output from the Rust binary.
   */
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
      pending.port?.postMessage(toProgressMessage(msg))
      return
    }

    this.pending.delete(id)

    if (msg.type === 'complete') {
      pending.resolve(msg.result)
      return
    }

    if (msg.type === 'resizeComplete') {
      pending.resolve(msg.result)
      return
    }

    if (msg.type === 'error') {
      pending.reject(new Error(msg.error))
    }
  }
}

const worker = new ImageMapWorker()

/**
 * Tinypool entry point. Runs a single task inside the worker.
 */
export default async function run(task: WorkerTask): Promise<TaskResult> {
  try {
    const params = task.params
    if ('type' in params && params.type === 'resize') {
      return await worker.resize(params, task.port)
    }
    // Default to generate (for backwards compatibility with WorkerParams without type)
    const generateParams = params as Omit<WorkerGenerateParams, 'type'>
    return await worker.generate(generateParams, task.port)
  }
  finally {
    task.port?.close()
  }
}

/**
 * Tinypool teardown hook to dispose of the binary process.
 */
export async function teardown(): Promise<void> {
  await worker.dispose()
}

/**
 * Normalize generate options for the Rust binary.
 */
function normalizeGenerateOptions(params: Omit<WorkerGenerateParams, 'type'>): GenerateOptions {
  const tileSize = params.tileSize ?? 256
  const formats = params.formats ?? ['webp']
  const origin = params.origin ?? 'topLeft'
  const minZoom = params.minZoom ?? 0
  const maxZoom = params.maxZoom ?? 0
  const resizeFilter = normalizeResizeFilter(params.resizeFilter)
  const downscaleSharpen = normalizeDownscaleSharpen(params.downscaleSharpen)

  if (!Number.isFinite(tileSize) || tileSize <= 0)
    throw new Error('tileSize must be a positive number')

  if (formats.length === 0)
    throw new Error('formats must contain at least one format')

  if (minZoom < 0 || maxZoom < 0)
    throw new Error('minZoom/maxZoom must be >= 0')

  if (minZoom > maxZoom)
    throw new Error('minZoom must be <= maxZoom')

  return {
    resizeFilter,
    downscaleSharpen,
    tileSize,
    formats,
    origin,
    minZoom,
    maxZoom,
  }
}

/**
 * Normalize and validate downscale sharpening options.
 */
function normalizeDownscaleSharpen(
  input?: DownscaleSharpenOptions,
): Required<DownscaleSharpenOptions> {
  const enabled = input?.enabled ?? true
  const sigma = input?.sigma ?? 0.5
  const amount = input?.amount ?? 0.35
  const threshold = input?.threshold ?? 2

  if (typeof enabled !== 'boolean')
    throw new Error('downscaleSharpen.enabled must be a boolean')

  if (!Number.isFinite(sigma))
    throw new Error('downscaleSharpen.sigma must be a finite number')

  if (!Number.isFinite(amount))
    throw new Error('downscaleSharpen.amount must be a finite number')

  if (!Number.isFinite(threshold) || !Number.isInteger(threshold) || threshold < 0 || threshold > 255) {
    throw new Error('downscaleSharpen.threshold must be an integer between 0 and 255')
  }

  if (enabled) {
    if (sigma <= 0)
      throw new Error('downscaleSharpen.sigma must be a positive number')

    if (amount < 0)
      throw new Error('downscaleSharpen.amount must be a non-negative number')
  }

  return {
    enabled,
    sigma,
    amount,
    threshold,
  }
}

/**
 * Normalize and validate resize filter inputs.
 */
function normalizeResizeFilter(value?: ResizeFilter): ResizeFilter {
  const filter = value ?? 'catmullRom'
  const allowed: ResizeFilter[] = [
    'lanczos3',
    'catmullRom',
    'mitchell',
    'hamming',
    'bilinear',
    'box',
    'gaussian',
  ]

  if (!allowed.includes(filter))
    throw new Error(`resizeFilter must be one of: ${allowed.join(', ')}`)

  return filter
}

/**
 * Normalize and validate image format inputs.
 */
function normalizeFormat(value?: TileFormat): TileFormat {
  const format = value ?? 'webp'
  const allowed: TileFormat[] = ['png', 'jpg', 'jpeg', 'webp']

  if (!allowed.includes(format))
    throw new Error(`format must be one of: ${allowed.join(', ')}`)

  return format
}

/**
 * Normalize resize options for the Rust binary.
 */
function normalizeResizeOptions(params: Omit<WorkerResizeParams, 'type'>): ResizeImageOptions {
  const mode = params.mode
  const format = normalizeFormat(params.format)
  const resizeFilter = normalizeResizeFilter(params.resizeFilter)
  const sharpen = normalizeDownscaleSharpen(params.sharpen)

  // Validate mode
  if (!mode || typeof mode !== 'object' || !('type' in mode))
    throw new Error('mode must be a valid resize mode object')

  if (mode.type === 'percentage') {
    if (typeof mode.value !== 'number' || !Number.isFinite(mode.value) || mode.value <= 0)
      throw new Error('mode.value must be a positive number for percentage mode')
  }
  else if (mode.type === 'longEdge') {
    if (typeof mode.pixels !== 'number' || !Number.isInteger(mode.pixels) || mode.pixels <= 0)
      throw new Error('mode.pixels must be a positive integer for longEdge mode')
  }
  else if (mode.type === 'fit') {
    if (typeof mode.width !== 'number' || !Number.isInteger(mode.width) || mode.width <= 0)
      throw new Error('mode.width must be a positive integer for fit mode')
    if (typeof mode.height !== 'number' || !Number.isInteger(mode.height) || mode.height <= 0)
      throw new Error('mode.height must be a positive integer for fit mode')
  }
  else {
    throw new Error('mode.type must be one of: percentage, longEdge, fit')
  }

  return {
    mode,
    format,
    resizeFilter,
    sharpen,
  }
}

/**
 * Convert protocol progress message to worker progress message.
 */
function toProgressMessage(message: Extract<ResponseMessage, { type: 'progress' }>): WorkerProgressMessage {
  return {
    type: 'progress',
    current: message.current,
    total: message.total,
    message: message.message,
  }
}
