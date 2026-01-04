import { defineConfig } from 'tsdown'

export default defineConfig({
  entry: {
    'index': 'src/index.ts',
    'cli': 'src/cli.ts',
    'pool-worker': 'src/pool-worker.ts',
  },
  dts: true,
  exports: false,
  platform: 'node',
})
