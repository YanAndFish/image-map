/* eslint-disable no-console */
import { resolve } from 'node:path'
import { ImageMap } from '@yafh/image-map'
import { rimraf } from 'rimraf'

/**
 * 执行本地示例任务并输出结果。
 */
async function main() {
  await rimraf(resolve(import.meta.dirname, '../output'))

  console.time('generate')

  const result = await ImageMap.generate({
    input: resolve(import.meta.dirname, '../input/input.jpg'),
    output: resolve(import.meta.dirname, '../output'),
    formats: ['png'],
    tileSize: 400,
    origin: 'topLeft',
    maxZoom: 4,
    resizeFilter: 'lanczos3',
    onProgress(current, total, message) {
      console.timeLog('generate', `${current}/${total} ${message}`)
    },
  })

  console.log(result)
  console.timeEnd('generate')
}

main().catch((err) => {
  console.error(err)
})
