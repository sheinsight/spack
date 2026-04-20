import { resolve } from 'node:path'
import { readFileSync } from 'node:fs'

import type { Plugin } from 'vite'

export interface InjectDataPluginOptions {
  dataPath?: string
  targetHtml?: string
  injectInDev?: boolean
}

export function injectDataPlugin(options: InjectDataPluginOptions = {}): Plugin {
  const { dataPath, targetHtml = 'index.test.html', injectInDev = true } = options
  let bundleData: string | null = null
  let isDev = false

  return {
    name: 'vite-plugin-inject-data',
    enforce: 'post',

    configResolved(config) {
      isDev = config.mode === 'development'
      // Load data file if path is provided
      if (dataPath) {
        try {
          const absolutePath = resolve(process.cwd(), dataPath)
          const data = readFileSync(absolutePath, 'utf-8')
          // Minify JSON and escape for injection
          bundleData = JSON.stringify(JSON.parse(data))
        } catch (error) {
          console.error(`Failed to load data from ${dataPath}:`, error)
        }
      }
    },

    transformIndexHtml: {
      order: 'post',
      handler(html, ctx) {
        // In dev mode, inject to all HTML files if injectInDev is true
        // In build mode, only inject to target HTML
        const isTargetHtml = ctx.filename?.includes(targetHtml)
        const shouldInject = bundleData && ((isDev && injectInDev) || isTargetHtml)

        if (shouldInject) {
          // Replace the placeholder with actual data
          return html.replace(
            /window\.__bundle_viewer_data__\s*=\s*null/,
            `window.__bundle_viewer_data__ = ${bundleData}`,
          )
        }

        return html
      },
    },
  }
}
