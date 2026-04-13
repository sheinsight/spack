/**
 * 模块-代码块依赖关系分析（重构版）
 * 将大组件拆分为多个可维护的子组件
 */

import { memo, useMemo } from 'react'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { ModuleChunkMatrix } from './modules/module-chunk-matrix'
import { useChunkDepthAndCircular, useChunkNodes, useChunkStats } from '@/hooks/use-chunk-relations'
import { formatNumber, formatPercentage } from '@/lib/utils/format'

import type { StandardBundleData } from '@/types/bundle-data-standard'
import type { NavigationParams } from '@/lib/navigation'

interface ModuleRelationsProps {
  data: StandardBundleData
  navigationParams?: NavigationParams
}

export const ModuleRelations = memo(function ModuleRelations({ data }: ModuleRelationsProps) {
  const chunkNodes = useChunkNodes(data)
  const { nodesWithDepth, circularPaths } = useChunkDepthAndCircular(chunkNodes)
  const stats = useChunkStats(nodesWithDepth, circularPaths)

  const overview = useMemo(() => {
    const totalModules = data.modules.length
    const totalChunks = data.chunks.length
    const totalEdges = data.chunks.reduce((sum, chunk) => sum + chunk.childIds.length, 0)
    const avgChunksPerModule =
      totalModules === 0 ? 0 : data.modules.reduce((sum, m) => sum + m.chunkIds.length, 0) / totalModules
    const asyncRatio = totalChunks === 0 ? 0 : stats.asyncCount / totalChunks

    let riskLabel = '可控'
    let riskVariant: 'default' | 'secondary' | 'outline' | 'destructive' = 'default'
    if (circularPaths.length > 0) {
      riskLabel = '高风险'
      riskVariant = 'destructive'
    } else if (stats.maxDepth >= 4 || stats.avgChildren >= 3) {
      riskLabel = '需关注'
      riskVariant = 'outline'
    }

    return {
      totalModules,
      totalChunks,
      totalEdges,
      avgChunksPerModule,
      asyncRatio,
      riskLabel,
      riskVariant,
    }
  }, [data, stats, circularPaths.length])

  const relationshipInsights = useMemo(() => {
    const chunkImpact = nodesWithDepth
      .map((node) => ({
        id: node.id,
        name: node.name,
        size: node.size,
        children: node.children.length,
        parents: node.parents.length,
        moduleCount: node.moduleCount,
        impactScore: node.children.length * 3 + node.parents.length * 2 + node.moduleCount + node.size / 1024 / 50,
      }))
      .toSorted((a, b) => b.impactScore - a.impactScore)
      .slice(0, 6)

    const moduleImpact = [...data.modules]
      .toSorted((a, b) => b.chunkIds.length - a.chunkIds.length || b.size - a.size)
      .slice(0, 6)

    const signals: string[] = []
    if (circularPaths.length > 0) {
      signals.push(`存在 ${circularPaths.length} 处循环依赖，建议先拆解或隔离。`)
    }
    if (stats.maxDepth >= 4) {
      signals.push(`最大层级为 ${stats.maxDepth}，加载链路偏深。`)
    }
    if (stats.avgChildren >= 3) {
      signals.push(`平均子 chunk 数 ${stats.avgChildren.toFixed(1)}，耦合度偏高。`)
    }
    if (overview.asyncRatio < 0.3) {
      signals.push(`异步代码块占比偏低（${formatPercentage(overview.asyncRatio * 100)}）。`)
    }
    if (signals.length === 0) {
      signals.push('未发现明显风险信号，保持即可。')
    }

    return {
      chunkImpact,
      moduleImpact,
      signals,
    }
  }, [data.modules, nodesWithDepth, stats, circularPaths.length, overview.asyncRatio])

  return (
    <div className="space-y-6 animate-fade-in">
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-xs font-medium text-muted-foreground">模块总数</CardTitle>
          </CardHeader>
          <CardContent className="pb-3">
            <p className="text-2xl font-bold">{formatNumber(overview.totalModules)}</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-xs font-medium text-muted-foreground">代码块总数</CardTitle>
          </CardHeader>
          <CardContent className="pb-3">
            <p className="text-2xl font-bold">{formatNumber(overview.totalChunks)}</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-xs font-medium text-muted-foreground">依赖边数</CardTitle>
          </CardHeader>
          <CardContent className="pb-3">
            <div className="flex items-baseline gap-2">
              <p className="text-2xl font-bold">{formatNumber(overview.totalEdges)}</p>
              <Badge variant="outline" className="text-xs">
                最大层级 {overview.totalChunks ? stats.maxDepth : 0}
              </Badge>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-xs font-medium text-muted-foreground">异步占比</CardTitle>
          </CardHeader>
          <CardContent className="pb-3">
            <div className="flex items-baseline gap-2">
              <p className="text-2xl font-bold">{formatPercentage(overview.asyncRatio * 100)}</p>
              <Badge variant={overview.riskVariant} className="text-xs">
                {overview.riskLabel}
              </Badge>
            </div>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <div className="flex flex-wrap items-start justify-between gap-2">
            <div>
              <CardTitle>关系健康度</CardTitle>
              <CardDescription>从链路结构判断复杂度与风险</CardDescription>
            </div>
            <div className="flex flex-wrap items-center gap-2 text-xs">
              <Badge variant="outline">平均关联度 {overview.avgChunksPerModule.toFixed(1)}</Badge>
              <Badge variant={overview.riskVariant}>{overview.riskLabel}</Badge>
              <Badge variant="outline">最大层级 {overview.totalChunks ? stats.maxDepth : 0}</Badge>
              {circularPaths.length > 0 && <Badge variant="destructive">循环依赖 {circularPaths.length} 处</Badge>}
            </div>
          </div>
        </CardHeader>
        <CardContent className="flex flex-wrap items-center gap-2 text-xs">
          <span className="text-muted-foreground">
            {relationshipInsights.signals.join('')}注：平均关联度是指平均每模块关联的代码块数量
          </span>
        </CardContent>
      </Card>

      <div className="grid gap-4 lg:grid-cols-2">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle>代码块影响面</CardTitle>
            <CardDescription>优先处理影响面大的链路节点</CardDescription>
          </CardHeader>
          <CardContent className="space-y-2 text-xs">
            {relationshipInsights.chunkImpact.map((chunk) => (
              <div key={chunk.id} className="flex items-center justify-between gap-3 rounded-md border px-3 py-2">
                <div className="truncate font-mono" title={chunk.name}>
                  {chunk.name}
                </div>
                <div className="flex items-center gap-1">
                  <Badge variant="outline">
                    父 {chunk.parents} / 子 {chunk.children}
                  </Badge>
                  <Badge variant="outline">{chunk.moduleCount} 模块</Badge>
                  <Badge variant="outline">
                    {formatPercentage((chunk.moduleCount / (overview.totalModules || 1)) * 100)}
                  </Badge>
                </div>
              </div>
            ))}
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle>模块影响面</CardTitle>
            <CardDescription>改动范围最大的模块优先排查</CardDescription>
          </CardHeader>
          <CardContent className="space-y-2 text-xs">
            {relationshipInsights.moduleImpact.map((module) => (
              <div key={module.name} className="flex items-center justify-between gap-3 rounded-md border px-3 py-2">
                <div className="min-w-0 truncate font-mono" title={module.name}>
                  {module.name}
                </div>
                <div className="flex items-center gap-1">
                  <Badge variant="outline">{module.chunkIds.length} 块</Badge>
                  <Badge variant="outline">
                    {formatPercentage((module.chunkIds.length / (overview.totalChunks || 1)) * 100)}
                  </Badge>
                </div>
              </div>
            ))}
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader className="pb-3">
          <div className="flex flex-wrap items-start justify-between gap-3">
            <div>
              <CardTitle>模块映射</CardTitle>
              <CardDescription>从模块视角定位影响面与拆分优先级</CardDescription>
            </div>
            <div className="flex flex-wrap items-center gap-2 text-xs">
              <Badge variant="outline">模块 {formatNumber(overview.totalModules)}</Badge>
              <Badge variant="outline">代码块 {formatNumber(overview.totalChunks)}</Badge>
              <Badge variant="outline">平均关联度 {overview.avgChunksPerModule.toFixed(1)}</Badge>
            </div>
          </div>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="flex items-center flex-wrap gap-2 text-xs text-muted-foreground">
            <Badge variant="secondary">适用问题</Badge>
            <span>改动影响范围、模块拆分机会、引用集中度</span>
          </div>
          <ModuleChunkMatrix data={data} />
        </CardContent>
      </Card>
    </div>
  )
})
