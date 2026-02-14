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
  /** Whether to auto-orient input image pixels using EXIF orientation metadata. */
  autoOrient: boolean
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

/** Resize by percentage of original size. */
export interface ResizeModePercentage {
  type: 'percentage'
  /** Percentage value (e.g., 50 means 50% of original size). */
  value: number
}

/** Resize by specifying the long edge in pixels. */
export interface ResizeModeLongEdge {
  type: 'longEdge'
  /** Target long edge size in pixels. */
  pixels: number
}

/** Resize by specifying both width and height (fit within, keep aspect ratio). */
export interface ResizeModeFit {
  type: 'fit'
  /** Maximum width in pixels. */
  width: number
  /** Maximum height in pixels. */
  height: number
}

/** Resize mode specification. */
export type ResizeMode = ResizeModePercentage | ResizeModeLongEdge | ResizeModeFit

/** Options for resizing an image (without tiling). */
export interface ResizeImageOptions {
  /** The resize mode specifying how to calculate output dimensions. */
  mode: ResizeMode
  /** Output format for the resized image. */
  format: TileFormat
  /** Whether to auto-orient input image pixels using EXIF orientation metadata. */
  autoOrient: boolean
  /** Resize filter for downscaling. */
  resizeFilter: ResizeFilter
  /** Sharpening configuration for downscaling. */
  sharpen: Required<DownscaleSharpenOptions>
}

/** Result payload for a completed resize request. */
export interface ResizeResult {
  /** Output file path. */
  outputPath: string
  /** Input image width after EXIF auto-orientation (if enabled). */
  originalWidth: number
  /** Input image height after EXIF auto-orientation (if enabled). */
  originalHeight: number
  /** Resized image width. */
  width: number
  /** Resized image height. */
  height: number
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

export interface ResizeRequestMessage {
  /** Message type. */
  type: 'resize'
  /** Request id for correlating responses. */
  id: string
  /** Input image path. */
  input: string
  /** Output file path. */
  output: string
  /** Resize options. */
  options: ResizeImageOptions
}

export type RequestMessage = GenerateRequestMessage | ResizeRequestMessage

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

export interface ResizeCompleteResponseMessage {
  /** Message type. */
  type: 'resizeComplete'
  /** Request id for correlating responses. */
  id: string
  /** Resize result payload. */
  result: ResizeResult
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
    | ResizeCompleteResponseMessage
    | ErrorResponseMessage
