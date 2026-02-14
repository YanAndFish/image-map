import type { MessagePort } from 'node:worker_threads'

import type {
  DownscaleSharpenOptions,
  Origin,
  ResizeFilter,
  ResizeMode,
  TileFormat,
} from './protocol'

/**
 * Serialized generate task params sent to the Tinypool worker.
 */
export interface WorkerGenerateParams {
  /** Task type. */
  type: 'generate'
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
}

/**
 * Serialized resize task params sent to the Tinypool worker.
 */
export interface WorkerResizeParams {
  /** Task type. */
  type: 'resize'
  /** Input file path. */
  input: string
  /** Output file path. */
  output: string
  /** Resize mode. */
  mode: ResizeMode
  /** Output format. */
  format?: TileFormat
  /** Whether to auto-orient input image pixels using EXIF orientation metadata. */
  autoOrient?: boolean
  /** Resize filter for downscaling. */
  resizeFilter?: ResizeFilter
  /** Sharpening configuration for downscaling. */
  sharpen?: DownscaleSharpenOptions
}

/**
 * @deprecated Use WorkerGenerateParams instead.
 */
export type WorkerParams = Omit<WorkerGenerateParams, 'type'>

/**
 * Unified worker params.
 */
export type WorkerTaskParams = WorkerGenerateParams | WorkerResizeParams

/**
 * Payload passed into a Tinypool worker task.
 */
export interface WorkerTask {
  /** Serializable task params. */
  params: WorkerTaskParams
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
