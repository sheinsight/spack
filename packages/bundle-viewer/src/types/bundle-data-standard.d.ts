/**
 * Bundle Viewer 标准数据模型
 *
 * 设计原则：
 * 1. 字段命名清晰直观，避免工具特定术语
 * 2. 核心字段必需，扩展字段可选
 * 3. 类型安全，减少运行时错误
 * 4. 关联关系明确，易于理解
 */

/**
 * 标准化的构建数据根对象
 */
export interface StandardBundleData {
  /** 数据生成时间戳 */
  timestamp: number

  /** 构建摘要统计 */
  summary: BuildSummary

  /** 模块列表 */
  modules: Module[]

  /** 代码块列表 */
  chunks: Chunk[]

  /** npm 包列表 */
  packages: Package[]

  /** 静态资源列表 */
  assets: Asset[]

  /** 快速查找映射（使用 ID 作为 key，性能优化） */
  moduleMap: Map<number, Module>
  chunkMap: Map<string, Chunk>
  packageMap: Map<string, Package[]> // 同一个包可能有多个版本

  /** 扩展元数据（工具特定信息） */
  ext?: {
    /** 构建工具标识 */
    buildTool?: 'lego' | 'webpack' | 'rollup' | 'vite' | 'esbuild' | string
    /** 构建工具版本 */
    buildToolVersion?: string
    /** 其他元数据 */
    [key: string]: any
  }
}

/**
 * 构建摘要统计
 */
export interface BuildSummary {
  /** 总体积（字节） */
  totalSize: number
  /** Gzip 压缩后体积（字节） */
  totalGzipSize: number
  /** 静态资源数量 */
  totalAssets: number
  /** 模块总数 */
  totalModules: number
  /** 代码块数量 */
  totalChunks: number
  /** 构建总耗时（毫秒） */
  buildTime: number
  /** 详细耗时统计 */
  timings: {
    collectAssetsMs: number
    collectModulesMs: number
    collectChunksMs: number
    analyzePackagesMs: number
    totalMs: number
  }
}

/**
 * 模块类型
 */
export type ModuleType =
  | 'javascript' // JS/JSX/TS/TSX 模块
  | 'json' // JSON 数据文件
  | 'css' // CSS/SCSS/LESS 样式
  | 'asset' // 图片、字体等静态资源
  | 'wasm' // WebAssembly
  | 'unknown' // 未知类型

/**
 * 标准化的模块对象
 */
export interface Module {
  // ========== 核心标识字段 ==========
  /** 模块唯一 ID */
  id: number
  /** 模块路径/名称（统一表示） */
  name: string

  // ========== 体积信息 ==========
  /** 模块体积（字节） */
  size: number

  // ========== 类型和分类 ==========
  /** 模块类型 */
  type: ModuleType
  /** 是否为 node_modules 中的第三方模块 */
  isNodeModule: boolean

  // ========== 关联关系 ==========
  /** 所属代码块 ID 列表 */
  chunkIds: string[]
  /** 所属 npm 包名称（仅第三方模块） */
  packageName?: string

  // ========== 扩展字段（工具特定或调试信息） ==========
  ext?: {
    /** lego nameForCondition（原始路径） */
    nameForCondition?: string
    /** package.json 路径 */
    packageJsonPath?: string | null
    /** 合并的子模块（lego concatenation） */
    concatenatedModules?: Array<{
      id: number
      size: number
      name: string
      isNodeModule: boolean
      packageJsonPath?: string | null
    }> | null
    /** 原始导入请求（调试用） */
    rawRequest?: string | null
    /** 被引用原因列表（调试用） */
    reasons?: number[]
    /** 其他扩展字段 */
    [key: string]: any
  }
}

/**
 * 代码块类型
 */
export type ChunkType =
  | 'entry' // 入口代码块（直接影响首屏加载）
  | 'initial' // 初始代码块（与入口同步加载）
  | 'async' // 异步代码块（懒加载）
  | 'runtime' // 运行时代码块（lego runtime）

/**
 * 标准化的代码块对象
 */
export interface Chunk {
  // ========== 核心标识字段 ==========
  /** 代码块唯一 ID */
  id: string
  /** 代码块名称列表 */
  names: string[]

  // ========== 体积信息 ==========
  /** 代码块体积（字节） */
  size: number

  // ========== 类型标识 ==========
  /** 代码块类型（统一枚举） */
  type: ChunkType

  // ========== 关联关系 ==========
  /** 包含的模块 ID 列表 */
  moduleIds: number[]
  /** 父代码块 ID 列表 */
  parentIds: string[]
  /** 子代码块 ID 列表 */
  childIds: string[]

  // ========== 扩展字段 ==========
  ext?: {
    /** 输出文件名列表 */
    files?: string[]
    /** 创建原因（调试用） */
    reason?: string
    /** 其他扩展字段 */
    [key: string]: any
  }
}

/**
 * 标准化的 npm 包对象
 */
export interface Package {
  // ========== 核心标识字段 ==========
  /** 包名称 */
  name: string
  /** 包版本 */
  version: string

  // ========== 体积信息 ==========
  /** 包总体积（字节） */
  size: number
  /** 包含的模块数量 */
  moduleCount: number

  // ========== 关联关系 ==========
  /** 包含的模块 ID 列表 */
  moduleIds: number[]

  // ========== 扩展字段 ==========
  ext?: {
    /** package.json 路径 */
    packageJsonPath?: string
    /** 其他扩展字段 */
    [key: string]: any
  }
}

/**
 * 标准化的静态资源对象
 */
export interface Asset {
  // ========== 核心标识字段 ==========
  /** 资源文件名 */
  name: string

  // ========== 体积信息 ==========
  /** 资源体积（字节） */
  size: number

  // ========== 关联关系 ==========
  /** 关联的代码块 ID 列表 */
  chunkIds: string[]

  // ========== 扩展字段（可选的优化信息） ==========
  ext?: {
    /** Gzip 压缩后体积 */
    gzipSize?: number
    /** Brotli 压缩后体积 */
    brotliSize?: number
    /** 是否被输出 */
    emitted?: boolean
    /** 其他扩展字段 */
    [key: string]: any
  }
}

export {}
