import type {
  DownscaleSharpenOptions,
  Origin,
  ResizeFilter,
  TileFormat,
} from './protocol'

import process from 'node:process'
import { ImageMap } from './index'

export async function main(argv: string[] = process.argv.slice(2)) {
  const { command, args } = parseCommand(argv)

  if (!command || command === 'help' || hasHelpFlag(args)) {
    writeHelp()
    return
  }

  if (command !== 'generate') {
    throw new Error(`Unknown command: ${command}`)
  }

  const input = getStringArg(args, '--input')
  const output = getStringArg(args, '--output')

  if (!input)
    throw new Error('Missing required option: --input')
  if (!output)
    throw new Error('Missing required option: --output')

  const tileSize = getNumberArg(args, '--tile-size', 256)
  const formats = getMultiStringArg(args, '--format', ['webp']) as TileFormat[]
  const origin = (getStringArg(args, '--origin') ?? 'topLeft') as Origin
  const minZoom = getNumberArg(args, '--min-zoom', 0)
  const maxZoom = getNumberArg(args, '--max-zoom', 0)
  const resizeFilter = getStringArg(args, '--resize-filter') as ResizeFilter | undefined
  const downscaleSharpen: DownscaleSharpenOptions = {
    enabled: getBooleanArg(args, '--downscale-sharpen', true),
    sigma: getNumberArg(args, '--downscale-sharpen-sigma', 0.5),
    amount: getNumberArg(args, '--downscale-sharpen-amount', 0.35),
    threshold: getNumberArg(args, '--downscale-sharpen-threshold', 2),
  }

  const result = await ImageMap.generate({
    input,
    output,
    tileSize,
    formats,
    origin,
    minZoom,
    maxZoom,
    resizeFilter,
    downscaleSharpen,
    onProgress: (current, total, message) => {
      process.stderr.write(`${current}/${total} ${message}\n`)
    },
  })

  process.stdout.write(`${JSON.stringify(result)}\n`)
}

function parseCommand(argv: string[]) {
  if (argv.length === 0)
    return { command: null as null | string, args: argv }

  const first = argv[0]
  if (first === 'generate' || first === 'gen')
    return { command: 'generate', args: argv.slice(1) }

  if (first === 'help' || first === '--help' || first === '-h')
    return { command: 'help', args: argv.slice(1) }

  // Default to `generate` for convenience.
  return { command: 'generate', args: argv }
}

function hasHelpFlag(args: string[]) {
  return args.includes('--help') || args.includes('-h')
}

function getStringArg(args: string[], name: string): string | undefined {
  const i = args.indexOf(name)
  if (i === -1)
    return undefined
  return args[i + 1]
}

function getNumberArg(args: string[], name: string, fallback: number): number {
  const value = getStringArg(args, name)
  if (value == null)
    return fallback
  const n = Number(value)
  if (!Number.isFinite(n))
    throw new Error(`Invalid number for ${name}: ${value}`)
  return n
}

/**
 * Read a boolean argument, accepting true/false or 1/0.
 */
function getBooleanArg(args: string[], name: string, fallback: boolean): boolean {
  const value = getStringArg(args, name)
  if (value == null)
    return fallback
  if (value === 'true' || value === '1')
    return true
  if (value === 'false' || value === '0')
    return false
  throw new Error(`Invalid boolean for ${name}: ${value}`)
}

function getMultiStringArg(args: string[], name: string, fallback: string[]): string[] {
  const out: string[] = []
  for (let i = 0; i < args.length; i++) {
    if (args[i] !== name)
      continue
    const value = args[i + 1]
    if (value)
      out.push(value)
    i++
  }
  return out.length ? out : fallback
}

function writeHelp() {
  process.stdout.write(
    [
      'image-map (Rust-powered image tile generator)',
      '',
      'Usage:',
      '  image-map generate --input <file> --output <dir> [options]',
      '',
      'Options:',
      '  --tile-size <n>     Tile size in pixels (default: 256)',
      '  --format <fmt>      Repeatable. One of: png | jpg | jpeg | webp (default: webp)',
      '  --origin <o>        One of: topLeft | center (default: topLeft)',
      '  --min-zoom <n>      Minimum zoom level (default: 0)',
      '  --max-zoom <n>      Maximum zoom level (default: 0)',
      '  --resize-filter <f> One of: lanczos3 | catmullRom | mitchell | hamming | bilinear | box | gaussian (default: catmullRom)',
      '  --downscale-sharpen <bool>        Enable downscale sharpen (default: true)',
      '  --downscale-sharpen-sigma <n>     Gaussian blur sigma (default: 0.5)',
      '  --downscale-sharpen-amount <n>    Unsharp amount (default: 0.35)',
      '  --downscale-sharpen-threshold <n> Threshold 0-255 (default: 2)',
      '  -h, --help          Show this help',
      '',
    ].join('\n'),
  )
}
