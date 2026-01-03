import fs from 'node:fs'
import { createRequire } from 'node:module'
import path from 'node:path'
import process from 'node:process'
import { fileURLToPath } from 'node:url'

const require = createRequire(import.meta.url)

export type SupportedPlatformPackage
  = | '@yafh/image-map-darwin-arm64'
    | '@yafh/image-map-darwin-x64'
    | '@yafh/image-map-freebsd-arm64'
    | '@yafh/image-map-freebsd-x64'
    | '@yafh/image-map-linux-arm'
    | '@yafh/image-map-linux-arm64'
    | '@yafh/image-map-linux-ia32'
    | '@yafh/image-map-linux-loong64'
    | '@yafh/image-map-linux-mips64el'
    | '@yafh/image-map-linux-ppc64'
    | '@yafh/image-map-linux-riscv64'
    | '@yafh/image-map-linux-s390x'
    | '@yafh/image-map-linux-x64'
    | '@yafh/image-map-win32-arm64'
    | '@yafh/image-map-win32-ia32'
    | '@yafh/image-map-win32-x64'

export function getPlatformPackageName(
  platform: NodeJS.Platform = process.platform,
  arch: string = process.arch,
): SupportedPlatformPackage {
  if (platform === 'darwin' && arch === 'arm64')
    return '@yafh/image-map-darwin-arm64'
  if (platform === 'darwin' && arch === 'x64')
    return '@yafh/image-map-darwin-x64'
  if (platform === 'freebsd' && arch === 'arm64')
    return '@yafh/image-map-freebsd-arm64'
  if (platform === 'freebsd' && arch === 'x64')
    return '@yafh/image-map-freebsd-x64'
  if (platform === 'linux' && arch === 'arm')
    return '@yafh/image-map-linux-arm'
  if (platform === 'linux' && arch === 'arm64')
    return '@yafh/image-map-linux-arm64'
  if (platform === 'linux' && arch === 'ia32')
    return '@yafh/image-map-linux-ia32'
  if (platform === 'linux' && arch === 'loong64')
    return '@yafh/image-map-linux-loong64'
  if (platform === 'linux' && arch === 'mips64el')
    return '@yafh/image-map-linux-mips64el'
  if (platform === 'linux' && arch === 'ppc64')
    return '@yafh/image-map-linux-ppc64'
  if (platform === 'linux' && arch === 'riscv64')
    return '@yafh/image-map-linux-riscv64'
  if (platform === 'linux' && arch === 's390x')
    return '@yafh/image-map-linux-s390x'
  if (platform === 'linux' && arch === 'x64')
    return '@yafh/image-map-linux-x64'
  if (platform === 'win32' && arch === 'arm64')
    return '@yafh/image-map-win32-arm64'
  if (platform === 'win32' && arch === 'ia32')
    return '@yafh/image-map-win32-ia32'
  if (platform === 'win32' && arch === 'x64')
    return '@yafh/image-map-win32-x64'

  throw new Error(`Unsupported platform: ${platform}-${arch}`)
}

export function resolveBinaryPath(): string {
  const envBinary = process.env.IMAGE_MAP_BINARY_PATH
  if (envBinary) {
    if (!fs.existsSync(envBinary)) {
      throw new Error(
        `IMAGE_MAP_BINARY_PATH points to a non-existent file: ${envBinary}`,
      )
    }
    return envBinary
  }

  const platformPkg = getPlatformPackageName()

  const fromPlatformPackage = tryResolveFromPlatformPackage(platformPkg)
  if (fromPlatformPackage)
    return fromPlatformPackage

  const fromMonorepo = tryResolveFromMonorepoBuild()
  if (fromMonorepo)
    return fromMonorepo

  throw new Error(
    [
      `Failed to locate image-map binary for ${process.platform}-${process.arch}.`,
      `Tried: ${platformPkg}/bin/image-map`,
      `If you are developing locally, set IMAGE_MAP_BINARY_PATH to your built Rust binary.`,
    ].join('\n'),
  )
}

function tryResolveFromPlatformPackage(pkgName: SupportedPlatformPackage): string | null {
  try {
    const pkgJsonPath = require.resolve(`${pkgName}/package.json`)
    const pkgDir = path.dirname(pkgJsonPath)

    const candidates = process.platform === 'win32'
      ? [
          path.join(pkgDir, 'bin', 'image-map.exe'),
          path.join(pkgDir, 'bin', 'image-map'),
        ]
      : [path.join(pkgDir, 'bin', 'image-map')]

    for (const candidate of candidates) {
      if (fs.existsSync(candidate))
        return candidate
    }
    return null
  }
  catch {
    return null
  }
}

function tryResolveFromMonorepoBuild(): string | null {
  // `packages/image-map/src/binary.ts` -> repo root
  const here = path.dirname(fileURLToPath(import.meta.url))
  const repoRoot = path.resolve(here, '..', '..', '..')

  const candidates = process.platform === 'win32'
    ? [
        path.join(repoRoot, 'src-native', 'target', 'release', 'image-map.exe'),
        path.join(repoRoot, 'src-native', 'target', 'debug', 'image-map.exe'),
      ]
    : [
        path.join(repoRoot, 'src-native', 'target', 'release', 'image-map'),
        path.join(repoRoot, 'src-native', 'target', 'debug', 'image-map'),
      ]

  for (const candidate of candidates) {
    if (fs.existsSync(candidate))
      return candidate
  }

  return null
}
