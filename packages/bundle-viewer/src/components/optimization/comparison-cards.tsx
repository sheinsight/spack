/**
 * 对比卡片：当前状态 vs 优化后预期（合并版）
 */

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { formatBytes } from '@/lib/utils/format'
import { cn } from '@/lib/utils'

interface OptimizationImpact {
  current: {
    entrySize: number
    duplicatePackages: number
    asyncChunks: number
    score: number
  }
  optimized: {
    entrySize: number
    duplicatePackages: number
    asyncChunks: number
    score: number
  }
  benefits: {
    sizeSaving: number
    duplicateSavings: number
    scoreDelta: number
  }
}

interface ComparisonCardsProps {
  optimizationImpact: OptimizationImpact
}

interface MetricRowProps {
  label: string
  current: React.ReactNode
  optimized: React.ReactNode
  delta?: React.ReactNode
  isLast?: boolean
}

function MetricRow({ label, current, optimized, delta, isLast }: MetricRowProps) {
  return (
    <div
      className={cn('grid grid-cols-[1fr_1fr_auto_1fr_1fr] items-center gap-x-4 py-3 text-sm', !isLast && 'border-b')}
    >
      <span className="text-muted-foreground">{label}</span>

      {/* 当前值 */}
      <div className="text-right">{current}</div>

      {/* 箭头 */}
      <span className="text-muted-foreground/40 text-base">→</span>

      {/* 优化后值 */}
      <div className="text-right">{optimized}</div>

      {/* 变化量 */}
      <div className="text-right">{delta}</div>
    </div>
  )
}

export function ComparisonCards({ optimizationImpact }: ComparisonCardsProps) {
  const { current, optimized, benefits } = optimizationImpact

  const entrySizeDelta = current.entrySize - optimized.entrySize
  const dupDelta = current.duplicatePackages - optimized.duplicatePackages
  const asyncDelta = optimized.asyncChunks - current.asyncChunks
  const scoreDelta = benefits.scoreDelta

  return (
    <Card className="relative overflow-hidden">
      <div className="absolute top-0 right-0 w-40 h-40 bg-primary/5 rounded-full blur-3xl pointer-events-none" />

      <CardHeader>
        <CardTitle>优化前后对比</CardTitle>
        <CardDescription>应用所有建议后的预期收益</CardDescription>
      </CardHeader>

      <CardContent>
        {/* 列标题 */}
        <div className="grid grid-cols-[1fr_1fr_auto_1fr_1fr] items-center gap-x-4 pb-2 border-b text-xs text-muted-foreground/60 font-medium">
          <span>指标</span>
          <span className="text-right">当前状态</span>
          <span className="opacity-0 text-base">→</span>
          <span className="text-right">优化后预期</span>
          <span className="text-right">变化</span>
        </div>

        {/* Entry 块体积 */}
        <MetricRow
          label="Entry 块体积"
          current={<span className="font-medium text-destructive">{formatBytes(current.entrySize)}</span>}
          optimized={
            <span className="font-medium text-green-600 dark:text-green-400">{formatBytes(optimized.entrySize)}</span>
          }
          delta={
            entrySizeDelta > 0 ? (
              <Badge variant="default" className="text-xs bg-green-600 hover:bg-green-600 text-white">
                -{formatBytes(entrySizeDelta)}
              </Badge>
            ) : (
              <span className="text-xs text-muted-foreground/40">—</span>
            )
          }
        />

        {/* 重复依赖包 */}
        <MetricRow
          label="重复依赖包"
          current={<span className="font-medium text-destructive">{current.duplicatePackages} 个</span>}
          optimized={
            <span className="font-medium text-green-600 dark:text-green-400">{optimized.duplicatePackages} 个</span>
          }
          delta={
            dupDelta > 0 ? (
              <div className="text-right">
                <Badge variant="default" className="text-xs bg-green-600 hover:bg-green-600 text-white">
                  -{dupDelta} 个
                </Badge>
                {benefits.duplicateSavings > 0 && (
                  <div className="text-xs text-green-600 dark:text-green-400 mt-0.5">
                    -{formatBytes(benefits.duplicateSavings)}
                  </div>
                )}
              </div>
            ) : (
              <span className="text-xs text-muted-foreground/40">—</span>
            )
          }
        />

        {/* 异步代码块 */}
        <MetricRow
          label="异步代码块"
          current={<span className="font-medium text-destructive">{current.asyncChunks} 个</span>}
          optimized={<span className="font-medium text-green-600 dark:text-green-400">{optimized.asyncChunks} 个</span>}
          delta={
            asyncDelta > 0 ? (
              <Badge variant="default" className="text-xs bg-green-600 hover:bg-green-600 text-white">
                +{asyncDelta} 个
              </Badge>
            ) : (
              <span className="text-xs text-muted-foreground/40">—</span>
            )
          }
        />

        {/* 优化评分 */}
        <MetricRow
          label="优化评分"
          isLast
          current={<span className="text-xl font-bold text-destructive">{current.score} 分</span>}
          optimized={<span className="text-xl font-bold text-green-600 dark:text-green-400">{optimized.score} 分</span>}
          delta={
            scoreDelta > 0 ? (
              <Badge variant="default" className="text-xs bg-green-600 hover:bg-green-600 text-white">
                +{scoreDelta.toFixed(0)} 分
              </Badge>
            ) : (
              <span className="text-xs text-muted-foreground/40">—</span>
            )
          }
        />
      </CardContent>
    </Card>
  )
}
