import { mkdirSync, readFileSync, writeFileSync } from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { spawnSync } from 'node:child_process'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const rootDir = path.resolve(__dirname, '..')
const viewerDir = path.join(rootDir, 'packages', 'bundle-viewer')
const viewerDistHtml = path.join(viewerDir, 'dist', 'index.html')
const targetDir = path.join(rootDir, 'crates', 'spack_plugin_bundle_analyzer', 'assets')
const targetHtml = path.join(targetDir, 'bundle-viewer.html')
const pnpmBin = process.platform === 'win32' ? 'pnpm.cmd' : 'pnpm'

const buildResult = spawnSync(pnpmBin, ['run', 'build:prod'], {
  cwd: viewerDir,
  stdio: 'inherit',
})

if (buildResult.status !== 0) {
  process.exit(buildResult.status ?? 1)
}

const nextHtml = readFileSync(viewerDistHtml, 'utf8')
const currentHtml = tryRead(targetHtml)

if (!/window\.__bundle_viewer_data__\s*=\s*null\b/.test(nextHtml)) {
  throw new Error('bundle-viewer build output is missing the data injection placeholder')
}

mkdirSync(targetDir, { recursive: true })

if (currentHtml === nextHtml) {
  console.log(`bundle-viewer asset already up to date: ${path.relative(rootDir, targetHtml)}`)
  process.exit(0)
}

writeFileSync(targetHtml, nextHtml)
console.log(`synced bundle-viewer asset: ${path.relative(rootDir, targetHtml)}`)

function tryRead(file) {
  try {
    return readFileSync(file, 'utf8')
  } catch {
    return null
  }
}
