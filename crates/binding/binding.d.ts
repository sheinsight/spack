export interface JsDuplicateDependencyPluginResp {
  libraries: Array<JsLibrary>
  duration: number
}

export interface JsLibrary {
  dir: string
  name: string
  version: string
}

export interface RawBundleAnalyzerPluginOpts {

}

export interface RawCaseSensitivePathsPluginOpts {

}

export interface RawDuplicateDependencyPluginOpts {
  onDetected?: async (response: JsDuplicateDependencyPluginResp) => Promise<void>
}

export declare function registerBundleAnalyzerPlugin(): void

export declare function registerCaseSensitivePathsPlugin(): void

export declare function registerDuplicateDependencyPlugin(): void
