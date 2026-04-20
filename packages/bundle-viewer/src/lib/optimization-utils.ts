/**
 * 优化建议相关的工具函数和类型定义
 */

// 包管理器类型
export type PackageManager = 'npm' | 'yarn' | 'pnpm' | 'bun'

// 重复依赖包数据类型
export interface DuplicatePackage {
  name: string
  versions: {
    version: string
    size: number
    moduleCount: number
  }[]
  totalSize: number
  potentialSavings: number
  canDedupe?: boolean
  hasDifferentMajors?: boolean
  majorVersionCount?: number
}

/**
 * 根据包管理器生成命令
 */
export function getCommand(pm: PackageManager, type: 'dedupe' | 'reinstall'): string {
  const commands = {
    npm: {
      dedupe: 'npm dedupe',
      reinstall: 'rm -rf node_modules package-lock.json && npm install',
    },
    yarn: {
      dedupe: 'yarn dedupe',
      reinstall: 'rm -rf node_modules yarn.lock && yarn install',
    },
    pnpm: {
      dedupe: 'pnpm dedupe',
      reinstall: 'rm -rf node_modules pnpm-lock.yaml && pnpm install',
    },
    bun: {
      dedupe:
        '# bun 暂无内置 dedupe 命令\n# 建议：检查 package.json 中的依赖版本，手动统一版本号\n# 或使用 npm/pnpm 执行 dedupe 后再用 bun 安装',
      reinstall: 'rm -rf node_modules bun.lockb && bun install',
    },
  }
  return commands[pm][type]
}

/**
 * 提取主版本号（major version）
 */
export function getMajorVersion(version: string): string {
  const match = version.match(/^(\d+)/)
  return match ? match[1] : version
}

/**
 * 获取评分标签
 */
export function getScoreLabel(score: number): string {
  if (score >= 80) return '优秀'
  if (score >= 60) return '良好'
  if (score >= 40) return '一般'
  return '需要改进'
}
