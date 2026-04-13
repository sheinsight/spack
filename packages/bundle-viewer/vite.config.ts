import path from 'path'
import tailwindcss from '@tailwindcss/vite'
import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'
import { viteSingleFile } from 'vite-plugin-singlefile'
import { injectDataPlugin } from './vite-plugins/inject-data'

// Check if we're building the test version with injected data
const isTestBuild = process.env.BUILD_TEST === 'true'
const isDev = process.env.NODE_ENV !== 'production'
const inputFile = isTestBuild ? 'index.test.html' : 'index.html'

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    react(),
    tailwindcss(),
    injectDataPlugin({
      dataPath: isDev || isTestBuild ? 'data/bmas_db.json' : undefined,
      targetHtml: 'index.test.html',
    }),
    viteSingleFile(),
  ],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  build: {
    target: 'esnext',
    assetsInlineLimit: 100_000_000,
    chunkSizeWarningLimit: 100_000_000,
    cssCodeSplit: false,
    outDir: 'dist',
    emptyOutDir: !isTestBuild, // Only empty on first build
    rollupOptions: {
      input: path.resolve(__dirname, inputFile),
      output: {
        inlineDynamicImports: true,
      },
    },
  },
})
