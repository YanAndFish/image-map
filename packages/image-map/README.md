# @yafh/image-map

Rust 驱动的图片瓦片生成器，提供 Node.js SDK + CLI，适合批量切图与多级缩放瓦片生成。

## 安装

```bash
pnpm add @yafh/image-map
```

## CLI

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

## SDK

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
})

console.log(result)
```

## 平台依赖

平台原生二进制由可选依赖提供，安装时会按需拉取对应平台包。

## 相关链接

- GitHub：https://github.com/YanAndFish/image-map
