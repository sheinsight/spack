import { useMemo, useRef, useState } from 'react'
import { adaptToStandardBundleData } from '@/lib/data-adapter'
import type { NavigationParams } from '@/lib/navigation'
import type { StandardBundleData } from '@/types/bundle-data-standard'

// Hooks
import { useOptimizationData, useOptimizationScore, useOptimizationImpact, useQuickActions } from '@/hooks/use-optimization-data'
import { buildOptimizationMetrics } from '@/lib/optimization-metrics'

// 组件
import { QuickActionsPanel } from './optimization/quick-actions-panel'
import { OptimizationScoreCard } from './optimization/optimization-score-card'
import { PotentialSavingsCard } from './optimization/potential-savings-card'
import { ComparisonCards } from './optimization/comparison-cards'
import { BenefitsCard } from './optimization/benefits-card'
import { ChunkQualityCard } from './optimization/chunk-quality-card'
import { DuplicatePackagesTable } from './optimization/tables/duplicate-packages-table'
import { LargePackagesTable } from './optimization/tables/large-packages-table'
import { LargeModulesTable } from './optimization/tables/large-modules-table'
import { Card, CardContent, CardHeader, CardTitle } from './ui/card'
import { Badge } from './ui/badge'
import { formatBytes } from '@/lib/utils/format'
import { cn } from '@/lib/utils'

interface OptimizationSuggestionsProps {
  data: StandardBundleData
  navigationParams?: NavigationParams
}

export function OptimizationSuggestions({ data, navigationParams }: OptimizationSuggestionsProps) {
  // 计算所有数据（一次性聚合）
  const { packageStats, duplicatePackages, largePackages, largeModules, chunkQuality } = useOptimizationData(data)
  const optimizationScore = useOptimizationScore(duplicatePackages, chunkQuality, largePackages)
  const optimizationImpact = useOptimizationImpact(chunkQuality, duplicatePackages, optimizationScore)
  const quickActions = useQuickActions(packageStats, duplicatePackages, chunkQuality, largePackages)

  const highlightNames = useMemo(() => {
    return navigationParams?.highlight?.split(',').map((name) => name.trim()).filter(Boolean) || []
  }, [navigationParams?.highlight])

  const [compareData, setCompareData] = useState<StandardBundleData | null>(null)
  const [compareError, setCompareError] = useState<string | null>(null)
  const compareInputRef = useRef<HTMLInputElement>(null)
  const [showLargePackages, setShowLargePackages] = useState(false)
  const [showLargeModules, setShowLargeModules] = useState(false)
  const compareMetrics = useMemo(
    () => (compareData ? buildOptimizationMetrics(compareData) : null),
    [compareData],
  )

  const totalPotentialSavings = duplicatePackages.reduce((sum, p) => sum + p.potentialSavings, 0)
  const entryChunkSavings = Math.max(0, optimizationImpact.current.entrySize - optimizationImpact.optimized.entrySize)

  return (
    <div className="space-y-6 animate-fade-in">
      {/* 快速行动清单 */}
      <QuickActionsPanel actions={quickActions} />

      {/* 版本对比入口 */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            版本对比
            {compareData && (
              <Badge variant="outline" className="text-xs">
                已导入对比数据
              </Badge>
            )}
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="flex flex-wrap items-center gap-2">
            <input
              ref={compareInputRef}
              type="file"
              accept=".json"
              className="hidden"
              onChange={(event) => {
                const file = event.target.files?.[0]
                if (!file) return
                setCompareError(null)
                const reader = new FileReader()
                reader.onload = (e) => {
                  try {
                    const json = JSON.parse(e.target?.result as string)
                    if (!json.modules || !json.chunks || !json.assets) {
                      throw new Error('文件格式不正确，缺少必需字段（modules, chunks, assets）')
                    }
                    setCompareData(adaptToStandardBundleData(json))
                  } catch (error) {
                    const message = error instanceof Error ? error.message : '无效的 JSON 文件'
                    setCompareError(`解析失败：${message}`)
                  }
                }
                reader.onerror = () => setCompareError('文件读取失败，请重试')
                reader.readAsText(file)
              }}
            />
            <button
              className="px-3 py-1.5 bg-primary text-primary-foreground hover:bg-primary/90 rounded-md transition-colors text-xs font-medium"
              onClick={() => compareInputRef.current?.click()}
            >
              导入第二份数据
            </button>
            {compareData && (
              <button
                className="px-3 py-1.5 bg-muted hover:bg-muted/80 rounded-md transition-colors text-xs font-medium"
                onClick={() => {
                  setCompareData(null)
                  setCompareError(null)
                  if (compareInputRef.current) compareInputRef.current.value = ''
                }}
              >
                清除对比
              </button>
            )}
            <span className="text-xs text-muted-foreground">用于对比优化前后体积与问题变化</span>
          </div>
          {compareError && <div className="text-xs text-destructive">{compareError}</div>}
          {compareMetrics && (
            <div className="grid gap-3 md:grid-cols-3 text-sm">
              <CompareMetric
                label="总体积"
                current={formatBytes(data.summary.totalSize)}
                compare={formatBytes(compareData!.summary.totalSize)}
                delta={data.summary.totalSize - compareData!.summary.totalSize}
              />
              <CompareMetric
                label="入口体积"
                current={formatBytes(chunkQuality.entrySize)}
                compare={formatBytes(compareMetrics.chunkQuality.entrySize)}
                delta={chunkQuality.entrySize - compareMetrics.chunkQuality.entrySize}
              />
              <CompareMetric
                label="重复依赖数"
                current={`${duplicatePackages.length} 个`}
                compare={`${compareMetrics.duplicatePackages.length} 个`}
                delta={duplicatePackages.length - compareMetrics.duplicatePackages.length}
              />
            </div>
          )}
        </CardContent>
      </Card>

      {/* 优化评分 & 潜在节省空间 */}
      <div className="grid gap-6 md:grid-cols-2">
        <OptimizationScoreCard score={optimizationScore.score} />
        <PotentialSavingsCard
          duplicateSavings={totalPotentialSavings}
          duplicatePackagesCount={duplicatePackages.length}
          entryChunkSavings={entryChunkSavings}
          totalSize={data.summary.totalSize}
        />
      </div>

      {/* 发现的问题 */}
      {optimizationScore.issues.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              发现的问题
              <Badge variant="outline" className="text-xs">
                {optimizationScore.issues.length}
              </Badge>
            </CardTitle>
          </CardHeader>
          <CardContent>
            <ul className="space-y-2">
              {optimizationScore.issues.map((issue, idx) => (
                <li key={idx} className="flex items-start gap-3 p-2 rounded-lg hover:bg-muted/50 transition-colors">
                  <span
                    className={cn(
                      'mt-1.5 h-2.5 w-2.5 rounded-full shrink-0',
                      issue.impact === 'critical'
                        ? 'bg-destructive'
                        : issue.impact === 'high'
                          ? 'bg-orange-500'
                          : issue.impact === 'medium'
                            ? 'bg-amber-400'
                            : 'bg-emerald-500',
                    )}
                  />
                  <div className="flex-1 min-w-0">
                    <div className="flex items-start justify-between gap-2">
                      <span className="text-sm font-medium">{issue.message}</span>
                      <Badge variant="outline" className="text-xs shrink-0 font-mono">
                        -{issue.points}
                      </Badge>
                    </div>
                    <span className="text-xs text-muted-foreground">
                      {issue.impact === 'critical'
                        ? '严重影响'
                        : issue.impact === 'high'
                          ? '高优先级'
                          : issue.impact === 'medium'
                            ? '中等优先级'
                            : '低优先级'}
                    </span>
                  </div>
                </li>
              ))}
            </ul>
          </CardContent>
        </Card>
      )}

      {/* 对比卡片：当前 vs 优化后 */}
      <ComparisonCards optimizationImpact={optimizationImpact} />

      {/* 收益量化卡片 */}
      <BenefitsCard sizeSaving={optimizationImpact.benefits.sizeSaving} />

      {/* 代码分割质量 */}
      <div id="chunk-quality-section">
        <ChunkQualityCard chunkQuality={chunkQuality} />
      </div>

      {/* 多版本依赖 */}
      <DuplicatePackagesTable
        duplicatePackages={duplicatePackages}
        totalPotentialSavings={totalPotentialSavings}
        highlightNames={highlightNames}
      />

      {/* 大型依赖包 */}
      <SectionToggle
        title="大型依赖包"
        count={largePackages.length}
        open={showLargePackages}
        onToggle={() => setShowLargePackages((prev) => !prev)}
      />
      {showLargePackages && <LargePackagesTable largePackages={largePackages} />}

      {/* 大型模块 */}
      <SectionToggle
        title="大型模块"
        count={largeModules.length}
        open={showLargeModules}
        onToggle={() => setShowLargeModules((prev) => !prev)}
      />
      {showLargeModules && <LargeModulesTable largeModules={largeModules} />}
    </div>
  )
}

function SectionToggle({
  title,
  count,
  open,
  onToggle,
}: {
  title: string
  count: number
  open: boolean
  onToggle: () => void
}) {
  return (
    <div className="flex items-center justify-between gap-2 rounded-md border bg-background px-3 py-2">
      <div className="flex items-center gap-2">
        <span className="text-sm font-medium">{title}</span>
        <Badge variant="outline" className="text-xs">
          {count}
        </Badge>
      </div>
      <button
        className="text-xs text-muted-foreground hover:text-foreground transition-colors"
        onClick={onToggle}
        type="button"
      >
        {open ? '收起' : '展开'}
      </button>
    </div>
  )
}

function CompareMetric({
  label,
  current,
  compare,
  delta,
}: {
  label: string
  current: string
  compare: string
  delta: number
}) {
  const isImproved = delta <= 0
  return (
    <div className="rounded-md border bg-muted/30 p-3 space-y-1">
      <p className="text-xs text-muted-foreground">{label}</p>
      <div className="flex items-center justify-between gap-2 text-sm">
        <span className="font-medium">{current}</span>
        <span className="text-muted-foreground">对比 {compare}</span>
      </div>
      <p className={cn('text-xs font-medium', isImproved ? 'text-emerald-600' : 'text-destructive')}>
        {delta === 0 ? '无变化' : isImproved ? `改善 ${formatDelta(delta)}` : `增加 ${formatDelta(delta)}`}
      </p>
    </div>
  )
}

function formatDelta(delta: number) {
  const abs = Math.abs(delta)
  if (abs > 1024) {
    return formatBytes(abs)
  }
  return `${abs}`
}
