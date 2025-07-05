export interface JsDuplicateDependencyPluginResponse {
  libraries: Array<JsLibrary>
  duration: number
}

export interface JsLibrary {
  dir: string
  name: string
  version: string
}

export interface RawCaseSensitivePathsPluginOptions {
  debug: boolean
  useCache: boolean
}

export interface RawDuplicateDependencyPluginOptions {
  onDetected?: (response: DuplicateDependencyPluginResponse) => void
}

export declare function registerCaseSensitivePathsPlugin(): void

export declare function registerDuplicateDependencyPlugin(): void
