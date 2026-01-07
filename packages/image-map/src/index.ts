import type { GenerateParams, ResizeParams } from './pool'
import type { GenerateResult, ResizeResult } from './protocol'
import { createPool } from './pool'

export type { GenerateParams, ImageMapPool, PoolOptions, ResizeParams } from './pool'
export { createPool } from './pool'
export type * from './protocol'

export class ImageMap {
  /**
   * Generate tiles for an image.
   */
  static async generate(params: GenerateParams): Promise<GenerateResult> {
    const pool = createPool({ concurrency: 1 })
    pool.add(params)
    const [result] = await pool.run()
    return result as GenerateResult
  }

  /**
   * Resize an image without tiling.
   */
  static async resize(params: ResizeParams): Promise<ResizeResult> {
    const pool = createPool({ concurrency: 1 })
    pool.addResize(params)
    const [result] = await pool.run()
    return result as ResizeResult
  }
}
