export type TileFormat = 'png' | 'jpg' | 'jpeg' | 'webp'

export type Origin = 'topLeft' | 'center'

export interface GenerateOptions {
  tileSize: number
  formats: TileFormat[]
  origin: Origin
  minZoom: number
  maxZoom: number
}

export interface GenerateResult {
  tilesGenerated: number
  outputDir: string
}

export interface GenerateRequestMessage {
  type: 'generate'
  id: string
  input: string
  output: string
  options: GenerateOptions
}

export type RequestMessage = GenerateRequestMessage

export interface ProgressResponseMessage {
  type: 'progress'
  id: string
  current: number
  total: number
  message: string
}

export interface CompleteResponseMessage {
  type: 'complete'
  id: string
  result: GenerateResult
}

export interface ErrorResponseMessage {
  type: 'error'
  id: string
  error: string
}

export type ResponseMessage
  = | ProgressResponseMessage
    | CompleteResponseMessage
    | ErrorResponseMessage
