/**
 * 优化建议数据计算 Hooks
 * 将所有的 useMemo 数据计算逻辑集中管理
 */

import { useMemo } from 'react'
import { getModuleName } from '@/lib/utils/path-utils'
import { formatBytes, safePercentage, safeAverage, clamp } from '@/lib/utils/format'
import { SIZE_THRESHOLDS, OPTIMIZATION_WEIGHTS } from '@/lib/constants'
import { getCommand, getMajorVersion, type DuplicatePackage, type PackageManager } from '@/lib/optimization-utils'
import type { TabId } from '@/lib/navigation'
import { buildOptimizationMetrics } from '@/lib/optimization-metrics'
import type { StandardBundleData } from '@/types/bundle-data-standard'

/**
 * Packages 统计
 */
export function usePackageStats(data: StandardBundleData) {
  return useMemo(() => {
    const grouped = new Map<string, Set<string>>()
    for (const pkg of data.packages) {
      if (!grouped.has(pkg.name)) {
        grouped.set(pkg.name, new Set())
      }
      grouped.get(pkg.name)!.add(pkg.version)
    }

    const multiVersion = Array.from(grouped.entries()).filter(([, versions]) => versions.size > 1)

    return {
      total: grouped.size,
      multiVersion: multiVersion.length,
      totalPackages: data.packages.length,
    }
  }, [data.packages])
}

/**
 * 一次性计算优化数据（减少重复遍历）
 */
export function useOptimizationData(data: StandardBundleData) {
  return useMemo(() => buildOptimizationMetrics(data), [data])
}

/**
 * 分析多版本依赖
 */
export function useDuplicatePackages(data: StandardBundleData) {
  return useMemo(() => {
    // 使用 reduce 一次遍历完成分组
    const grouped = data.packages.reduce((map, pkg) => {
      if (!map.has(pkg.name)) {
        map.set(pkg.name, {
          name: pkg.name,
          versions: [],
          totalSize: 0,
          potentialSavings: 0,
        })
      }

      const group = map.get(pkg.name)!
      group.versions.push({
        version: pkg.version,
        size: pkg.size,
        moduleCount: pkg.moduleCount,
      })
      group.totalSize += pkg.size
      return map
    }, new Map<string, DuplicatePackage>())

    // 只保留多版本的包
    const duplicates = Array.from(grouped.values())
      .filter((g) => g.versions.length > 1)
      .map((g) => {
        // 按主版本号分组 - 使用 reduce 优化
        const byMajor = g.versions.reduce((map, v) => {
          const major = getMajorVersion(v.version)
          if (!map.has(major)) {
            map.set(major, [])
          }
          map.get(major)!.push(v)
          return map
        }, new Map<string, typeof g.versions>())

        // 计算潜在节省空间
        let potentialSavings = 0
        let canDedupe = false

        for (const versions of byMajor.values()) {
          if (versions.length > 1) {
            const sortedVersions = versions.toSorted((a, b) => b.size - a.size)
            potentialSavings += sortedVersions.slice(1).reduce((sum, v) => sum + v.size, 0)
            canDedupe = true
          }
        }

        const hasDifferentMajors = byMajor.size > 1

        return {
          ...g,
          versions: g.versions.toSorted((a, b) => b.size - a.size),
          potentialSavings,
          canDedupe,
          hasDifferentMajors,
          majorVersionCount: byMajor.size,
        }
      })
      .toSorted((a, b) => b.potentialSavings - a.potentialSavings)

    return duplicates
  }, [data.packages])
}

/**
 * 分析大型依赖包
 */
export function useLargePackages(data: StandardBundleData) {
  return useMemo(() => {
    const totalSize = data.packages.reduce((sum, p) => sum + p.size, 0)
    const threshold = SIZE_THRESHOLDS.LARGE_PACKAGE

    return data.packages
      .filter((p) => p.size > threshold)
      .map((p) => ({
        name: p.name,
        version: p.version,
        size: p.size,
        moduleCount: p.moduleCount,
        percentage: safePercentage(p.size, totalSize),
      }))
      .toSorted((a, b) => b.size - a.size)
      .slice(0, 20)
  }, [data.packages])
}

/**
 * 分析大型模块
 */
export function useLargeModules(data: StandardBundleData) {
  return useMemo(() => {
    const totalSize = data.modules.reduce((sum, m) => sum + m.size, 0)
    const threshold = SIZE_THRESHOLDS.LARGE_MODULE

    return data.modules
      .filter((m) => m.size > threshold)
      .map((m) => {
        const [name] = getModuleName(m.name, data.modules)
        return {
          name: m.name,
          displayName: name,
          size: m.size,
          chunkIds: m.chunkIds,
          type: m.type,
          isNodeModule: m.isNodeModule,
          percentage: safePercentage(m.size, totalSize),
        }
      })
      .toSorted((a, b) => b.size - a.size)
      .slice(0, 20)
  }, [data.modules])
}

/**
 * 分析 Chunk 质量
 */
export function useChunkQuality(data: StandardBundleData) {
  return useMemo(() => {
    const entryChunks = data.chunks.filter((c) => c.type === 'entry')
    const asyncChunks = data.chunks.filter((c) => c.type === 'async')

    const entrySize = entryChunks.reduce((sum, c) => sum + c.size, 0)
    const asyncSize = asyncChunks.reduce((sum, c) => sum + c.size, 0)
    const totalSize = data.chunks.reduce((sum, c) => sum + c.size, 0)

    const largeEntryChunks = entryChunks.filter((c) => c.size > SIZE_THRESHOLDS.ENTRY_CHUNK_WARNING)
    const tooManyModulesChunks = data.chunks.filter((c) => c.moduleIds.length > 1000)

    return {
      entrySize,
      asyncSize,
      totalSize,
      entryPercentage: safePercentage(entrySize, totalSize),
      asyncPercentage: safePercentage(asyncSize, totalSize),
      largeEntryChunks,
      tooManyModulesChunks,
      asyncChunkCount: asyncChunks.length,
    }
  }, [data.chunks])
}

/**
 * 计算总体优化分数
 */
export function useOptimizationScore(
  duplicatePackages: DuplicatePackage[],
  chunkQuality: ReturnType<typeof useChunkQuality>,
  largePackages: ReturnType<typeof useLargePackages>,
) {
  return useMemo(() => {
    let score = 100
    const issues: Array<{
      message: string
      impact: 'critical' | 'high' | 'medium' | 'low'
      points: number
    }> = []

    // 1. 多版本依赖
    if (duplicatePackages.length > 0) {
      const criticalDupes = duplicatePackages.filter((p) => p.potentialSavings > SIZE_THRESHOLDS.LARGE_PACKAGE)
      const deduction = Math.min(
        criticalDupes.length * OPTIMIZATION_WEIGHTS.duplicateDeps.critical +
          (duplicatePackages.length - criticalDupes.length) * OPTIMIZATION_WEIGHTS.duplicateDeps.high,
        OPTIMIZATION_WEIGHTS.duplicateDeps.max,
      )
      score -= deduction
      issues.push({
        message: `${duplicatePackages.length} 个包存在多个版本`,
        impact: criticalDupes.length > 0 ? 'critical' : 'high',
        points: deduction,
      })
    }

    // 2. 入口代码块过大
    if (chunkQuality.largeEntryChunks.length > 0) {
      const deduction = Math.min(
        chunkQuality.largeEntryChunks.length * OPTIMIZATION_WEIGHTS.entryChunkSize.perChunk,
        OPTIMIZATION_WEIGHTS.entryChunkSize.max,
      )
      score -= deduction
      const avgSize = safeAverage(
        chunkQuality.largeEntryChunks.reduce((s, c) => s + c.size, 0),
        chunkQuality.largeEntryChunks.length,
      )
      issues.push({
        message: `${chunkQuality.largeEntryChunks.length} 个入口代码块 > 244KB (平均: ${formatBytes(avgSize)})`,
        impact: avgSize > SIZE_THRESHOLDS.ENTRY_CHUNK_CRITICAL ? 'critical' : 'high',
        points: deduction,
      })
    }

    // 3. 入口代码块占比过高
    if (chunkQuality.entryPercentage > OPTIMIZATION_WEIGHTS.entryPercentage.threshold) {
      const excessPercentage = chunkQuality.entryPercentage - OPTIMIZATION_WEIGHTS.entryPercentage.threshold
      const deduction = Math.min((excessPercentage / 10) * 5, OPTIMIZATION_WEIGHTS.entryPercentage.max)
      score -= deduction
      issues.push({
        message: `入口代码块占比过高 (${chunkQuality.entryPercentage.toFixed(1)}%)`,
        impact: chunkQuality.entryPercentage > 70 ? 'high' : 'medium',
        points: deduction,
      })
    }

    // 4. 异步 chunk 太少
    if (chunkQuality.asyncChunkCount < OPTIMIZATION_WEIGHTS.asyncChunks.threshold) {
      const deduction = OPTIMIZATION_WEIGHTS.asyncChunks.max
      score -= deduction
      issues.push({
        message: `懒加载利用不足 (仅 ${chunkQuality.asyncChunkCount} 个异步代码块)`,
        impact: 'medium',
        points: deduction,
      })
    }

    // 5. 超大包检测
    const megaPackages = largePackages.filter((p) => p.size > OPTIMIZATION_WEIGHTS.largePackages.threshold)
    if (megaPackages.length > 0) {
      const deduction = Math.min(megaPackages.length * 5, OPTIMIZATION_WEIGHTS.largePackages.max)
      score -= deduction
      issues.push({
        message: `${megaPackages.length} 个包体积超过 1MB`,
        impact: 'medium',
        points: deduction,
      })
    }

    return {
      score: clamp(Math.round(score), 0, 100),
      issues,
      breakdown: {
        total: 100,
        deductions: 100 - clamp(Math.round(score), 0, 100),
      },
    }
  }, [duplicatePackages, chunkQuality, largePackages])
}

/**
 * 计算优化影响和收益
 */
export function useOptimizationImpact(
  chunkQuality: ReturnType<typeof useChunkQuality>,
  duplicatePackages: DuplicatePackage[],
  optimizationScore: ReturnType<typeof useOptimizationScore>,
) {
  const totalPotentialSavings = duplicatePackages.reduce((sum, p) => sum + p.potentialSavings, 0)

  return useMemo(() => {
    const currentEntrySize = chunkQuality.entrySize
    const currentDuplicatePackages = duplicatePackages.length
    const currentAsyncChunks = chunkQuality.asyncChunkCount
    const currentScore = optimizationScore.score

    const targetEntrySize = SIZE_THRESHOLDS.ENTRY_CHUNK_WARNING
    const optimizedEntrySize = Math.min(currentEntrySize, targetEntrySize)
    const optimizedDuplicatePackages = 0
    const optimizedAsyncChunks = Math.max(currentAsyncChunks, 5)

    let optimizedScore = 100
    if (optimizedEntrySize <= targetEntrySize) optimizedScore = 100
    else optimizedScore -= Math.min(chunkQuality.largeEntryChunks.length * 15, 30)

    const sizeSaving = totalPotentialSavings + Math.max(0, currentEntrySize - optimizedEntrySize)
    const scoreDelta = optimizedScore - currentScore

    return {
      current: {
        entrySize: currentEntrySize,
        duplicatePackages: currentDuplicatePackages,
        asyncChunks: currentAsyncChunks,
        score: currentScore,
      },
      optimized: {
        entrySize: optimizedEntrySize,
        duplicatePackages: optimizedDuplicatePackages,
        asyncChunks: optimizedAsyncChunks,
        score: Math.min(optimizedScore, 100),
      },
      benefits: {
        sizeSaving,
        duplicateSavings: totalPotentialSavings,
        scoreDelta: Math.max(0, scoreDelta),
      },
    }
  }, [chunkQuality, duplicatePackages, totalPotentialSavings, optimizationScore.score])
}

/**
 * 快速行动清单分析
 */
export function useQuickActions(
  packageStats: ReturnType<typeof usePackageStats>,
  duplicatePackages: DuplicatePackage[],
  chunkQuality: ReturnType<typeof useChunkQuality>,
  largePackages: ReturnType<typeof useLargePackages>,
) {
  return useMemo(() => {
    const actions: Array<{
      priority: 'P0' | 'P1' | 'P2'
      title: string
      problem: string
      impact: string
      suggestion: string
      getCommand: (pm: PackageManager) => string
      disclaimer: string
      estimatedTime: string
      estimatedSaving: string
      steps: Array<{ label: string; code?: string; note?: string }>
      target?: {
        tab: TabId
        sectionId: string
        highlight?: string
      }
      ctaLabel?: string
    }> = []

    // P0: 检查重复依赖
    if (packageStats.multiVersion > 0) {
      const wastedSize = duplicatePackages.reduce((sum, pkg) => sum + pkg.potentialSavings, 0)
      actions.push({
        priority: 'P0',
        title: '解决重复依赖',
        problem: `${duplicatePackages.length} 个包存在多个版本`,
        impact: `浪费 ${formatBytes(wastedSize)}，影响首屏加载`,
        suggestion:
          '运行包管理器的 dedupe 命令，让工具自动将同主版本的依赖提升到最高兼容版本，通常是成本最低的优化手段。',
        getCommand: (pm) => getCommand(pm, 'dedupe'),
        disclaimer: '以实际包管理器及其文档说明为准，命令仅供参考',
        estimatedTime: '5 分钟',
        estimatedSaving: formatBytes(wastedSize),
        target: {
          tab: 'optimization',
          sectionId: 'duplicate-packages-section',
          highlight: duplicatePackages
            .slice(0, 5)
            .map((pkg) => pkg.name)
            .join(','),
        },
        ctaLabel: '定位重复依赖',
        steps: [
          { label: '运行依赖去重', code: 'npm dedupe\n# 或\npnpm dedupe' },
          { label: '重新安装依赖', code: 'rm -rf node_modules && npm install' },
          { label: '验证效果', note: '重新构建并对比体积变化，确认优化效果。' },
        ],
      })
    }

    // P1: 检查大型 Entry chunk
    if (chunkQuality.largeEntryChunks.length > 0) {
      const totalEntrySize = chunkQuality.largeEntryChunks.reduce((sum, c) => sum + c.size, 0)
      actions.push({
        priority: 'P1',
        title: '拆分大型 Entry Chunk',
        problem: `${chunkQuality.largeEntryChunks.length} 个入口块超过 244KB`,
        impact: `阻塞渲染，当前 ${formatBytes(totalEntrySize)}，直接影响 FCP`,
        suggestion:
          '通过 splitChunks 将第三方库提取为独立 vendors chunk，并对路由页面使用动态 import，让入口块仅保留启动代码。',
        getCommand: () => '',
        disclaimer: '以实际构建工具文档为准，配置仅供参考',
        estimatedTime: '30 分钟',
        estimatedSaving: `约 ${formatBytes(totalEntrySize * 0.3)}`,
        target: {
          tab: 'optimization',
          sectionId: 'chunk-quality-section',
        },
        ctaLabel: '查看入口块',
        steps: [
          {
            label: '提取第三方库为独立 Chunk',
            note: "在构建工具中配置 splitChunks（chunks: 'all'），将 node_modules 中的依赖提取为独立 vendors chunk，避免污染入口体积。",
          },
          {
            label: '路由级动态导入',
            code: "// React\nconst Dashboard = lazy(() => import('./pages/Dashboard'))\nconst Settings  = lazy(() => import('./pages/Settings'))\n\n// Vue\nconst Dashboard = () => import('./pages/Dashboard.vue')",
          },
          {
            label: '验证 Entry 体积',
            note: '重新构建后，入口块应低于 244KB。可通过 webpack-bundle-analyzer 等工具确认分割效果。',
          },
        ],
      })
    }

    // P1: Entry 占比过高（代码分割质量差）
    if (chunkQuality.entryPercentage > 70 && chunkQuality.largeEntryChunks.length === 0) {
      actions.push({
        priority: 'P1',
        title: '优化代码分割策略',
        problem: `入口块占总体积 ${chunkQuality.entryPercentage.toFixed(0)}%，代码分割利用不足`,
        impact: '大量代码在首屏同步加载，阻碍 LCP 和 TTI',
        suggestion: '合理设置 splitChunks 策略，将非首屏必要代码拆分为异步 Chunk；同时检查是否有功能模块可以延迟加载。',
        getCommand: () => '',
        disclaimer: '配置方式因构建工具不同而异，请参考对应文档',
        estimatedTime: '1-2 小时',
        estimatedSaving: `约 ${formatBytes(chunkQuality.entrySize * 0.2)}`,
        steps: [
          {
            label: '按功能模块拆分 Chunk',
            note: '将非首屏必须的功能模块（如管理后台、用户中心、数据图表等）通过动态 import 拆分为异步加载。',
          },
          {
            label: '配置 splitChunks 缓存分组',
            code: "// webpack.config.js\noptimization: {\n  splitChunks: {\n    chunks: 'all',\n    cacheGroups: {\n      vendor: {\n        test: /node_modules/,\n        name: 'vendors',\n        chunks: 'all',\n      },\n    },\n  },\n}",
          },
        ],
      })
    }

    // P2: 懒加载利用不足（阈值从 3 提高到 5）
    if (chunkQuality.asyncChunkCount < 5) {
      actions.push({
        priority: 'P2',
        title: '增加路由懒加载',
        problem: `仅 ${chunkQuality.asyncChunkCount} 个异步块，懒加载利用率低`,
        impact: '初始 JS 体积偏大，影响 TTI（可交互时间）',
        suggestion:
          '对路由页面和低频功能模块使用 React.lazy / Vue 动态路由，将非关键代码延迟到实际访问时再加载，可显著降低初始包体积。',
        getCommand: () => '',
        disclaimer: '以实际框架（React/Vue 等）文档为准，示例代码仅供参考',
        estimatedTime: '1-2 小时',
        estimatedSaving: '约 200-500KB',
        steps: [
          {
            label: 'React 路由懒加载',
            code: "import { lazy, Suspense } from 'react'\n\nconst Dashboard = lazy(() => import('./pages/Dashboard'))\n\n// 使用 Suspense 包裹\n<Suspense fallback={<Loading />}>\n  <Dashboard />\n</Suspense>",
          },
          {
            label: 'Vue 路由懒加载',
            code: "// router/index.ts\nconst routes = [\n  {\n    path: '/dashboard',\n    component: () => import('./pages/Dashboard.vue'),\n  },\n]",
          },
          {
            label: '识别懒加载候选模块',
            note: '优先对以下类型模块实施懒加载：富文本编辑器、图表库、PDF 预览、大型表单页面、低频访问的管理功能。',
          },
        ],
      })
    }

    // P2: 替换超大依赖包（> 500KB）
    const hugePackages = largePackages.filter((p) => p.size > SIZE_THRESHOLDS.ENTRY_CHUNK_CRITICAL)
    if (hugePackages.length > 0) {
      const totalHugeSize = hugePackages.reduce((sum, p) => sum + p.size, 0)
      const hugeNames = hugePackages
        .slice(0, 3)
        .map((p) => p.name)
        .join('、')
      actions.push({
        priority: 'P2',
        title: '替换或按需加载超大依赖',
        problem: `${hugePackages.length} 个包体积超过 500KB（${hugeNames}${hugePackages.length > 3 ? ' 等' : ''}）`,
        impact: `这些包合计 ${formatBytes(totalHugeSize)}，显著增加 JS 解析耗时`,
        suggestion:
          '优先评估是否可以替换为轻量级库（如 dayjs 替代 moment.js）；若无法替换，通过动态 import 将其延迟到实际使用时加载。',
        getCommand: () => '',
        disclaimer: '替代方案需结合实际业务需求评估，请充分测试后再迁移',
        estimatedTime: '2-4 小时',
        estimatedSaving: '',
        steps: [
          {
            label: '常见轻量替代方案',
            code: 'moment.js  (~289 KB) → dayjs        (~7 KB)\nlodash     (~531 KB) → lodash-es  (按需 tree-shaking)\naxios       (~31 KB) → 原生 fetch API\njquery     (~88 KB)  → 原生 DOM API\nnumeraljs  (~60 KB)  → Intl.NumberFormat',
          },
          {
            label: '图表库按需引入',
            code: "// ❌ 全量引入\nimport * as echarts from 'echarts'\n\n// ✅ 按需引入，减少 60-80%\nimport * as echarts from 'echarts/core'\nimport { BarChart } from 'echarts/charts'\nimport { GridComponent } from 'echarts/components'",
          },
          {
            label: '使用 import() 动态加载',
            note: '若无法替换，可通过动态 import 将大型依赖延迟到实际使用时加载，避免阻塞首屏渲染。',
          },
        ],
      })
    }

    // P2: 模块数过多的 Chunk
    if (chunkQuality.tooManyModulesChunks.length > 0) {
      const worstChunk = chunkQuality.tooManyModulesChunks[0]
      actions.push({
        priority: 'P2',
        title: '优化过度聚合的 Chunk',
        problem: `${chunkQuality.tooManyModulesChunks.length} 个 Chunk 包含超过 1000 个模块`,
        impact: `模块过多导致 JS 引擎解析压力大，影响 TTI（最多含 ${worstChunk.moduleIds.length} 个模块）`,
        suggestion:
          '调整 splitChunks 的 maxSize 限制，将过大的 Chunk 分解为更细粒度的独立块；同时识别并提取多处共用的公共模块。',
        getCommand: () => '',
        disclaimer: '拆分策略需结合业务访问路径设计，建议配合性能测量工具验证',
        estimatedTime: '2-4 小时',
        estimatedSaving: '提升 TTI 约 200-800ms',
        steps: [
          {
            label: '分析 Chunk 组成',
            note: '使用 webpack-bundle-analyzer 或 rollup-plugin-visualizer 找出 Chunk 中体积最大的模块，识别不必要的聚合。',
          },
          {
            label: '配置更细粒度的 splitChunks',
            code: "optimization: {\n  splitChunks: {\n    chunks: 'all',\n    maxSize: 500 * 1024,    // 单个 Chunk 最大 500KB\n    minSize: 20 * 1024,     // 最小 Chunk 20KB\n    cacheGroups: {\n      commons: {\n        minChunks: 2,        // 2+ 次引用才提取\n        priority: -10,\n      },\n    },\n  },\n}",
          },
        ],
      })
    }

    return actions
  }, [chunkQuality, duplicatePackages, largePackages, packageStats])
}
