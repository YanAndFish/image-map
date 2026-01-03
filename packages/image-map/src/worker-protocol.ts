import type { MessagePort } from 'node:worker_threads'

import type { Origin, TileFormat } from './protocol'

/**
 * Serialized task params sent to the Tinypool worker.
 */
export interface WorkerParams {
  /** Input file path. */
  input: string
  /** Output directory path. */
  output: string
  /** Tile size in pixels. */
  tileSize?: number
  /** Output formats. */
  formats?: TileFormat[]
  /** Origin position. */
  origin?: Origin
  /** Minimum zoom level. */
  minZoom?: number
  /** Maximum zoom level. */
  maxZoom?: number
}

/**
 * Payload passed into a Tinypool worker task.
 */
export interface WorkerTask {
  /** Serializable task params. */
  params: WorkerParams
  /** Progress channel for this task. */
  port?: MessagePort
}

/**
 * Progress message sent from worker to main thread.
 */
export interface WorkerProgressMessage {
  /** Message type. */
  type: 'progress'
  /** Current progress value. */
  current: number
  /** Total progress value. */
  total: number
  /** Human readable message. */
  message: string
}
