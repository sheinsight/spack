export interface BundleData {
  timestamp: number
  summary: {
    totalSize: number
    totalGzipSize: number
    totalAssets: number
    totalModules: number
    totalChunks: number
    buildTime: number
    timings: {
      collectAssetsMs: number
      collectModulesMs: number
      collectChunksMs: number
      analyzePackagesMs: number
      totalMs: number
    }
  }
  moduleIdMap: Record<number, string>
  assets: {
    name: string
    size: number
    gzipSize: null
    brotliSize: null
    chunks: string[]
    emitted: boolean
  }[]
  modules: {
    id: number
    size: number
    chunks: string[]
    moduleKind: string
    isNodeModule: boolean
    nameForCondition: string
    concatenatedModules:
      | {
          id: number
          size: number
          isNodeModule: boolean
          nameForCondition: string
          packageJsonPath: null | string
        }[]
      | null
    packageJsonPath: string | null
    rawRequest: string | null
    reasons: number[]
  }[]
  chunks: {
    id: string
    names: string[]
    size: number
    modules: number[]
    entry: boolean
    initial: boolean
    reason: string
    files: string[]
    asyncChunks: boolean
    runtime: boolean
    parents: string[]
    children: string[]
  }[]
  packages: {
    name: string
    version: string
    size: number
    moduleCount: number
    modules: number[]
    packageJsonPath: string
  }[]
}

declare global {
  interface Window {
    __bundle_viewer_data__: BundleData | null
  }
}

export {}
