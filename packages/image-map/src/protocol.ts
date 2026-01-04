export type TileFormat = 'png' | 'jpg' | 'jpeg' | 'webp'

export type Origin = 'topLeft' | 'center'

/** Resize filter used for downscaling between zoom levels. */
export type ResizeFilter
  = | 'lanczos3'
    | 'catmullRom'
    | 'mitchell'
    | 'hamming'
    | 'bilinear'
    | 'box'
    | 'gaussian'

/** Options for downscale sharpening. */
export interface DownscaleSharpenOptions {
  /** Whether downscale sharpening is enabled. */
  enabled?: boolean
  /** Gaussian blur sigma for unsharp mask. */
  sigma?: number
  /** Unsharp mask amount multiplier. */
  amount?: number
  /** Threshold for minimal brightness change that will be sharpened. */
  threshold?: number
}

export interface GenerateOptions {
  /** Resize filter for building lower zoom levels. */
  resizeFilter: ResizeFilter
  /** Downscale sharpening configuration. */
  downscaleSharpen: Required<DownscaleSharpenOptions>
  /** Tile size in pixels. */
  tileSize: number
  /** Output formats. */
  formats: TileFormat[]
  /** Origin position. */
  origin: Origin
  /** Minimum zoom level. */
  minZoom: number
  /** Maximum zoom level. */
  maxZoom: number
}

export interface GenerateResult {
  /** Total number of tiles generated. */
  tilesGenerated: number
  /** Output directory path. */
  outputDir: string
}

export interface GenerateRequestMessage {
  /** Message type. */
  type: 'generate'
  /** Request id for correlating responses. */
  id: string
  /** Input image path. */
  input: string
  /** Output directory path. */
  output: string
  /** Generation options. */
  options: GenerateOptions
}

export type RequestMessage = GenerateRequestMessage

export interface ProgressResponseMessage {
  /** Message type. */
  type: 'progress'
  /** Request id for correlating responses. */
  id: string
  /** Current progress value. */
  current: number
  /** Total progress value. */
  total: number
  /** Human readable message. */
  message: string
}

export interface CompleteResponseMessage {
  /** Message type. */
  type: 'complete'
  /** Request id for correlating responses. */
  id: string
  /** Generation result payload. */
  result: GenerateResult
}

export interface ErrorResponseMessage {
  /** Message type. */
  type: 'error'
  /** Request id for correlating responses. */
  id: string
  /** Error message. */
  error: string
}

export type ResponseMessage
  = | ProgressResponseMessage
    | CompleteResponseMessage
    | ErrorResponseMessage
