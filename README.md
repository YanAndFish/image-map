# image-map

Rust 驱动的图片瓦片生成器，提供 Node.js SDK + CLI，适合批量切图与多级缩放瓦片生成。

## 仓库结构

- `packages/image-map`：Node.js SDK 与 CLI。
- `packages/image-map-<platform>`：平台原生二进制包（可选依赖）。
- `src-native`：Rust 核心实现。

## 安装

```bash
pnpm add @yafh/image-map
```

## CLI 使用

```bash
image-map generate --input <file> --output <dir> [options]
```

常用选项：

- `--tile-size <n>`：瓦片尺寸，默认 `256`。
- `--format <fmt>`：可重复，`png | jpg | jpeg | webp`，默认 `webp`。
- `--origin <o>`：`topLeft | center`，默认 `topLeft`。
- `--min-zoom <n>`：最小缩放级别，默认 `0`。
- `--max-zoom <n>`：最大缩放级别，默认 `0`。
- `-h, --help`：显示帮助。

示例：

```bash
image-map generate \
  --input ./input.png \
  --output ./tiles \
  --tile-size 256 \
  --format webp \
  --min-zoom 0 \
  --max-zoom 4
```

### 仅缩放（不切瓦片）

```bash
image-map resize --input <file> --output <file> [options]
```

保持原图比例，只做整体缩放，支持两种常用模式：

- 百分比缩放：`--mode percentage --value 50`（缩小到原始尺寸的 50%）
- 长边像素：`--mode longEdge --pixels 1200`（长边缩放到 1200px，短边按比例缩放）

常用选项（部分）：

- `--mode <m>`：`percentage | longEdge | fit`，默认 `percentage`。
- `--format <fmt>`：`png | jpg | jpeg | webp`，默认 `webp`。
- `--resize-filter <f>`：同 `generate`，默认 `catmullRom`。
- `--sharpen <bool>`：是否对缩小后的图做锐化，默认 `true`。
- `--sharpen-sigma / --sharpen-amount / --sharpen-threshold`：与瓦片缩放的锐化参数保持一致。

示例：

```bash
# 按 50% 等比缩小
image-map resize \
  --input ./input.png \
  --output ./resized-50.webp \
  --mode percentage \
  --value 50

# 将长边缩放到 1200px
image-map resize \
  --input ./input.png \
  --output ./resized-1200.webp \
  --mode longEdge \
  --pixels 1200
```

## SDK 使用

单任务：

```ts
import { ImageMap } from '@yafh/image-map'

const result = await ImageMap.generate({
  input: './input.png',
  output: './tiles',
  tileSize: 256,
  formats: ['webp'],
  origin: 'topLeft',
  minZoom: 0,
  maxZoom: 4,
  onProgress: (current, total, message) => {
    console.log(`${current}/${total}`, message)
  },
})

console.log(result)

// 仅缩放一张图（不切瓦片）
const resized = await ImageMap.resize({
  input: './input.png',
  output: './resized.png',
  // 百分比缩放：缩小到 50%
  mode: { type: 'percentage', value: 50 },
  // 或者指定长边像素：mode: { type: 'longEdge', pixels: 1200 },
  format: 'webp',
  onProgress: (current, total, message) => {
    console.log(`${current}/${total}`, message)
  },
})

console.log(resized)
```

并发任务：

```ts
import { createPool } from '@yafh/image-map'

const pool = createPool({ concurrency: 2 })

pool.add({ input: './a.png', output: './out/a' })
pool.add({ input: './b.png', output: './out/b' })

const results = await pool.run()
console.log(results)
```

`ImageMap.generate` 与 `pool.run()` 的返回值：

- `tilesGenerated`：生成的瓦片数量。
- `outputDir`：输出目录路径。

## 开发

```bash
pnpm install
pnpm lint
pnpm lint:fix
pnpm tsc
pnpm update-deps
```

Rust 侧（`src-native`）：

```bash
cd src-native
cargo build
cargo test
```
