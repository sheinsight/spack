/**
 * 格式化工具函数
 * 包含数字、字节、时间等格式化功能（安全版本）
 */

/**
 * 安全的除法运算
 */
export function safeDivide(numerator: number, denominator: number, defaultValue: number = 0): number {
  if (denominator === 0 || !isFinite(denominator)) {
    return defaultValue
  }
  const result = numerator / denominator
  return isFinite(result) ? result : defaultValue
}

/**
 * 安全的百分比计算
 */
export function safePercentage(part: number, total: number, defaultValue: number = 0): number {
  return safeDivide(part, total, defaultValue / 100) * 100
}

/**
 * 安全的平均值计算
 */
export function safeAverage(sum: number, count: number, defaultValue: number = 0): number {
  return safeDivide(sum, count, defaultValue)
}

/**
 * 限制数值范围
 */
export function clamp(value: number, min: number, max: number): number {
  if (!isFinite(value)) return min
  return Math.min(Math.max(value, min), max)
}

/**
 * 安全的 toFixed
 */
export function safeToFixed(value: number, decimals: number = 2, defaultValue: string = '0'): string {
  if (!isFinite(value)) {
    return defaultValue
  }
  return value.toFixed(decimals)
}

/**
 * 格式化字节大小（安全版本）
 */
export function formatBytes(bytes: number): string {
  if (!isFinite(bytes) || bytes < 0) {
    return '0 B'
  }

  if (bytes === 0) return '0 B'

  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  const sizeIndex = Math.min(i, sizes.length - 1)

  return `${(bytes / Math.pow(k, sizeIndex)).toFixed(2)} ${sizes[sizeIndex]}`
}

/**
 * 格式化时间（安全版本）
 */
export function formatTime(ms: number): string {
  if (!isFinite(ms) || ms < 0) {
    return '0ms'
  }

  if (ms < 1000) return `${ms.toFixed(2)}ms`
  return `${(ms / 1000).toFixed(2)}s`
}

/**
 * 格式化数字（安全版本，带千分位）
 */
export function formatNumber(num: number): string {
  if (!isFinite(num)) {
    return '0'
  }
  return new Intl.NumberFormat('en-US').format(Math.round(num))
}

/**
 * 格式化百分比
 */
export function formatPercentage(value: number, decimals = 1): string {
  if (!isFinite(value)) {
    return '0%'
  }
  return `${value.toFixed(decimals)}%`
}

/**
 * 截断文本
 */
export function truncateText(text: string, maxLength: number): string {
  if (text.length <= maxLength) return text
  return text.slice(0, maxLength) + '...'
}

/**
 * 根据文件大小返回对应的 Badge 变体
 */
export function getSizeBadgeVariant(bytes: number): 'default' | 'secondary' | 'destructive' | 'outline' {
  const kb = bytes / 1024
  if (kb > 500) return 'destructive' // > 500KB 红色警告
  if (kb > 100) return 'default' // > 100KB 主题色
  if (kb > 10) return 'secondary' // > 10KB 灰色
  return 'outline' // 小文件
}
