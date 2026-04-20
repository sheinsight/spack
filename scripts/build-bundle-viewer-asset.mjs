import { mkdirSync, readFileSync, writeFileSync } from 'node:fs'
import path from 'node:path'
import { spawnSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const rootDir = path.resolve(__dirname, '..')
const viewerDir = path.join(rootDir, 'packages', 'bundle-viewer')
const viewerDistHtml = path.join(viewerDir, 'dist', 'index.html')
const outputDir = path.join(rootDir, '.generated', 'bundle-viewer')
const outputHtml = path.join(outputDir, 'bundle-viewer.html')
const pnpmBin = process.platform === 'win32' ? 'pnpm.cmd' : 'pnpm'

const buildResult = spawnSync(pnpmBin, ['run', 'build:prod'], {
  cwd: viewerDir,
  stdio: 'inherit',
})

if (buildResult.status !== 0) {
  process.exit(buildResult.status ?? 1)
}

const nextHtml = readFileSync(viewerDistHtml, 'utf8')
if (!/window\.__bundle_viewer_data__\s*=\s*null\b/.test(nextHtml)) {
  throw new Error('bundle-viewer build output is missing the data injection placeholder')
}

mkdirSync(outputDir, { recursive: true })
writeFileSync(outputHtml, nextHtml)

console.log(`wrote bundle-viewer asset: ${path.relative(rootDir, outputHtml)}`)
