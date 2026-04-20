/**
 * 项目常量定义
 * 集中管理所有阈值、配置项等魔法数字
 */

/**
 * 体积阈值常量
 * 基于 Core Web Vitals 和行业最佳实践
 */
export const SIZE_THRESHOLDS = {
  /** Entry chunk 警告阈值: 244KB
   * 来源: 基于 3G 网络下的 FCP 性能基准
   * 参考: https://web.dev/performance-budgets-101/
   */
  ENTRY_CHUNK_WARNING: 244 * 1024,

  /** Entry chunk 严重警告: 500KB */
  ENTRY_CHUNK_CRITICAL: 500 * 1024,

  /** 大型依赖包: 100KB */
  LARGE_PACKAGE: 100 * 1024,

  /** 超大依赖包: 1MB */
  HUGE_PACKAGE: 1024 * 1024,

  /** 大型模块: 50KB */
  LARGE_MODULE: 50 * 1024,

  /** 文件大小 Badge 颜色阈值 */
  BADGE_THRESHOLDS: {
    HUGE: 500 * 1024, // > 500KB = destructive
    LARGE: 100 * 1024, // > 100KB = default
    MEDIUM: 10 * 1024, // > 10KB = secondary
    // <= 10KB = outline
  },
} as const

/**
 * 性能优化评分权重
 * 基于对 Core Web Vitals (FCP/LCP) 的影响程度
 */
export const OPTIMIZATION_WEIGHTS = {
  /** 重复依赖影响总体积 */
  duplicateDeps: {
    max: 25,
    critical: 5, // 同包 3+ 版本
    high: 3, // 同包 2 版本
  },

  /** Entry chunk 大小直接影响 FCP/LCP */
  entryChunkSize: {
    max: 30,
    perChunk: 15,
  },

  /** Entry 占比反映代码分割质量 */
  entryPercentage: {
    max: 20,
    threshold: 50, // 超过 50% 扣分
  },

  /** 懒加载利用率 */
  asyncChunks: {
    max: 15,
    threshold: 3, // 至少 3 个 async chunk
  },

  /** 超大包影响 */
  largePackages: {
    max: 10,
    threshold: 1024 * 1024, // 1MB
  },
} as const

/**
 * UI 显示限制
 */
export const UI_LIMITS = {
  /** 默认分页大小 */
  DEFAULT_PAGE_SIZE: 20,

  /** 表格最大显示行数 */
  MAX_TABLE_ROWS: 100,

  /** Top N 列表大小 */
  TOP_ITEMS_COUNT: 10,

  /** 搜索结果最大显示数 */
  MAX_SEARCH_RESULTS: 50,
} as const

/**
 * 性能指标基准
 * 用于估算优化影响
 */
export const PERFORMANCE_BASELINES = {
  /** 3G 网络下载速度 (bytes/ms) */
  NETWORK_3G_SPEED: (50 * 1024) / 1000, // 50KB/s = 50 bytes/ms

  /** 解析/执行时间系数 (相对于下载时间) */
  PARSE_EXECUTION_RATIO: 1.5,

  /** FCP 改善系数 (针对 Entry chunk 优化) */
  FCP_IMPROVEMENT_FACTOR: 1.0,

  /** LCP 改善系数 */
  LCP_IMPROVEMENT_FACTOR: 0.7,

  /** 包大小减少对整体性能的影响系数 */
  BUNDLE_SIZE_IMPACT: 0.5,
} as const

/**
 * 文件类型分类
 */
export const FILE_CATEGORIES = {
  SOURCE_EXTENSIONS: ['js', 'jsx', 'ts', 'tsx', 'mjs', 'cjs', 'vue', 'svelte', 'css', 'scss', 'sass', 'less'],

  IMAGE_EXTENSIONS: ['png', 'jpg', 'jpeg', 'gif', 'svg', 'webp', 'ico', 'avif'],

  FONT_EXTENSIONS: ['woff', 'woff2', 'ttf', 'eot', 'otf'],
} as const

/**
 * 模块重复度阈值
 */
export const DUPLICATION_THRESHOLDS = {
  /** 严重重复: 模块在 5+ 个 chunk 中 */
  SEVERE: 5,

  /** 高度重复: 模块在 3-4 个 chunk 中 */
  HIGH: 3,

  /** 中度重复: 模块在 2 个 chunk 中 */
  MODERATE: 2,
} as const

/**
 * Chunk 类型优先级
 */
export const CHUNK_PRIORITY = {
  ENTRY: 100,
  INITIAL: 80,
  RUNTIME: 60,
  ASYNC: 40,
} as const

/**
 * Badge 样式常量 - 保证视觉一致性
 * 统一所有列表、树形图的右侧 Badge 样式
 */
export const BADGE_STYLES = {
  /** 列表项和树节点右侧 Badge（最常用） */
  COMPACT: 'text-xs py-0 h-4 px-1.5',

  /** 体积 Badge - 固定宽度 + mono 字体 + 右对齐 */
  SIZE: 'text-xs py-0 h-4 px-1.5 min-w-20 text-right font-mono',

  /** 数字类 Badge（计数、版本号等） */
  NUMERIC: 'text-xs py-0 h-4 px-1.5 font-mono',

  /** 卡片头部 Badge - 与列表项统一 */
  HEADER: 'text-xs py-0 h-4 px-1.5',
} as const
