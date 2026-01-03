import type { GenerateParams } from './pool'
import type { GenerateResult } from './protocol'
import { createPool } from './pool'

export type { GenerateParams, ImageMapPool, PoolOptions } from './pool'
export { createPool } from './pool'
export type * from './protocol'

export class ImageMap {
  static async generate(params: GenerateParams): Promise<GenerateResult> {
    const pool = createPool({ concurrency: 1 })
    pool.add(params)
    const [result] = await pool.run()
    return result!
  }
}
