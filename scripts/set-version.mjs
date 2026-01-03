import fs from 'node:fs'
import path from 'node:path'
import process from 'node:process'
import { fileURLToPath } from 'node:url'

const version = process.argv[2]
if (!version) {
  process.stderr.write('Usage: node scripts/set-version.mjs <version>\n')
  process.exit(1)
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')

const packages = [
  'image-map',
  'image-map-darwin-arm64',
  'image-map-darwin-x64',
  'image-map-linux-x64',
  'image-map-win32-x64',
].map(name => path.join(repoRoot, 'packages', name, 'package.json'))

for (const pkgJsonPath of packages) {
  updatePackageVersion(pkgJsonPath, version)
}

// Keep optionalDependencies versions in sync.
const mainPkgPath = path.join(repoRoot, 'packages', 'image-map', 'package.json')
const mainPkg = readJson(mainPkgPath)
if (mainPkg.optionalDependencies && typeof mainPkg.optionalDependencies === 'object') {
  for (const key of Object.keys(mainPkg.optionalDependencies)) {
    mainPkg.optionalDependencies[key] = version
  }
}
writeJson(mainPkgPath, mainPkg)

process.stdout.write(`Updated package versions to ${version}\n`)

function updatePackageVersion(filePath, v) {
  const pkg = readJson(filePath)
  pkg.version = v
  writeJson(filePath, pkg)
}

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, 'utf8'))
}

function writeJson(filePath, data) {
  fs.writeFileSync(filePath, `${JSON.stringify(data, null, 2)}\n`, 'utf8')
}
