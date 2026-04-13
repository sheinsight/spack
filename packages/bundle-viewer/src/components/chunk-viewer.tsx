import { useMemo } from 'react'
import type { StandardBundleData } from '@/types/bundle-data-standard'
import { ChunkAnalysis } from './chunks/chunk-analysis'
import { ChunkTreeView } from './chunks/chunk-tree-view'
import { Tabs, TabsContent, TabsList, TabsTrigger } from './ui/tabs'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from './ui/card'
import { Badge } from './ui/badge'
import { formatBytes } from '@/lib/utils/format'
import { analyzeChunkComposition, getCategoryName } from '@/lib/chunk-composition-analyzer'
import { PercentageBar, type PercentageItem } from './ui/percentage-bar'
import type { NavigationParams } from '@/lib/navigation'

interface ChunkViewerProps {
  data: StandardBundleData
  navigationParams?: NavigationParams
}

// SVG 图标组件
function TreeIcon({ className = '' }: { className?: string }) {
  return (
    <svg
      className={className}
      width="14"
      height="14"
      viewBox="0 0 14 14"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <path
        d="M2 2.5L5 5.5L2 8.5M6.5 7H11.5M2 11H11.5"
        stroke="currentColor"
        strokeWidth="1.2"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  )
}

function GridIcon({ className = '' }: { className?: string }) {
  return (
    <svg
      className={className}
      width="14"
      height="14"
      viewBox="0 0 14 14"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <rect x="1.5" y="1.5" width="4.5" height="4.5" rx="0.75" stroke="currentColor" strokeWidth="1.2" />
      <rect x="8" y="1.5" width="4.5" height="4.5" rx="0.75" stroke="currentColor" strokeWidth="1.2" />
      <rect x="1.5" y="8" width="4.5" height="4.5" rx="0.75" stroke="currentColor" strokeWidth="1.2" />
      <rect x="8" y="8" width="4.5" height="4.5" rx="0.75" stroke="currentColor" strokeWidth="1.2" />
    </svg>
  )
}

export function ChunkViewer({ data, navigationParams }: ChunkViewerProps) {
  const stats = useMemo(() => {
    const totalSize = data.chunks.reduce((sum, chunk) => sum + chunk.size, 0)
    const entryChunks = data.chunks.filter((c) => c.type === 'entry' || c.type === 'initial')
    const asyncChunks = data.chunks.filter((c) => c.type === 'async')

    return {
      totalChunks: data.chunks.length,
      totalSize,
      entryChunks: entryChunks.length,
      asyncChunks: asyncChunks.length,
    }
  }, [data.chunks])

  // Chunk 组成分析
  const compositionAnalysis = useMemo(() => analyzeChunkComposition(data), [data])

  // 准备 PercentageBar 的数据
  const percentageItems = useMemo(() => {
    return compositionAnalysis.distribution.map((dist) => {
      const percent = (dist.totalSize / compositionAnalysis.stats.totalSize) * 100
      // 根据分类分配颜色，保持语义一致性
      // pure-lib (纯第三方) → viz-2 碧绿色
      // lib-dominant (第三方主导) → viz-3
      // balanced (均衡) → viz-4
      // source-dominant (源码主导) → viz-5
      // pure-source (纯源码) → viz-1 钴蓝色
      const colorMap: Record<string, PercentageItem['variant']> = {
        'pure-lib': 'viz-2', // 纯第三方 - 碧绿色
        'lib-dominant': 'viz-3',
        balanced: 'viz-4',
        'source-dominant': 'viz-5',
        'pure-source': 'viz-1', // 纯源码 - 钴蓝色
      }
      return {
        label: `${getCategoryName(dist.category)} (${dist.count} 个)`,
        value: formatBytes(dist.totalSize),
        percent,
        variant: colorMap[dist.category] || 'viz-4',
      } as PercentageItem
    })
  }, [compositionAnalysis])

  return (
    <Tabs defaultValue="tree" className="w-full space-y-4 animate-fade-in">
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-xs font-medium text-muted-foreground">代码块总数</CardTitle>
          </CardHeader>
          <CardContent className="pb-3">
            <p className="text-2xl font-bold">{stats.totalChunks}</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-xs font-medium text-muted-foreground">总体积</CardTitle>
          </CardHeader>
          <CardContent className="pb-3">
            <p className="text-2xl font-bold">{formatBytes(stats.totalSize)}</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-xs font-medium text-muted-foreground">入口块</CardTitle>
          </CardHeader>
          <CardContent className="pb-3">
            <div className="flex items-baseline gap-2">
              <p className="text-2xl font-bold">{stats.entryChunks}</p>
              <Badge variant="secondary" className="text-xs">
                初始加载
              </Badge>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-xs font-medium text-muted-foreground">异步块</CardTitle>
          </CardHeader>
          <CardContent className="pb-3">
            <div className="flex items-baseline gap-2">
              <p className="text-2xl font-bold">{stats.asyncChunks}</p>
              <Badge variant="outline" className="text-xs">
                按需加载
              </Badge>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* 代码块组成分布 */}
      <Card>
        <CardHeader>
          <CardTitle>代码块组成分布</CardTitle>
          <CardDescription>按第三方库/源码比例分类的代码块分布</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {/* 总体统计 */}
            <div className="grid grid-cols-2 md:grid-cols-3 gap-3 p-3 rounded-lg bg-muted/50">
              <div className="text-sm">
                <p className="text-muted-foreground mb-1">第三方库总体积</p>
                <p className="text-lg font-bold">{formatBytes(compositionAnalysis.stats.totalLibSize)}</p>
                <p className="text-xs text-muted-foreground">
                  {compositionAnalysis.stats.avgLibPercentage.toFixed(1)}% 占比
                </p>
              </div>
              <div className="text-sm">
                <p className="text-muted-foreground mb-1">源码总体积</p>
                <p className="text-lg font-bold">{formatBytes(compositionAnalysis.stats.totalSourceSize)}</p>
                <p className="text-xs text-muted-foreground">
                  {(100 - compositionAnalysis.stats.avgLibPercentage).toFixed(1)}% 占比
                </p>
              </div>
              <div className="text-sm">
                <p className="text-muted-foreground mb-1">代码块总数</p>
                <p className="text-lg font-bold">{compositionAnalysis.stats.totalChunks}</p>
              </div>
            </div>

            {/* 使用 PercentageBar 组件显示分类分布和图例 */}
            <div className="space-y-2">
              <div className="flex items-center justify-between text-sm">
                <span className="font-medium">分类分布</span>
                <span className="text-xs text-muted-foreground">
                  {formatBytes(compositionAnalysis.stats.totalSize)} 总计
                </span>
              </div>
              <PercentageBar
                items={percentageItems}
                height="md"
                colorMode="distinct"
                showAnimation={true}
                showLegend={true}
                legendPosition="bottom"
                legendLayout="grid"
                legendColumns={5}
              />
            </div>
          </div>
        </CardContent>
      </Card>

      {/* 视图切换器 - 紧挨着视图内容上方，左对齐显示 */}
      <div className="flex justify-start">
        <TabsList>
          <TabsTrigger value="tree" className="gap-1.5">
            <TreeIcon />
            <span>树形</span>
          </TabsTrigger>
          <TabsTrigger value="list" className="gap-1.5">
            <GridIcon />
            <span>列表</span>
          </TabsTrigger>
        </TabsList>
      </div>

      {/* 视图内容 - 根据 TabsList 切换 */}
      <TabsContent value="tree" className="mt-0">
        <ChunkTreeView data={data} navigationParams={navigationParams} />
      </TabsContent>

      <TabsContent value="list" className="mt-0">
        <ChunkAnalysis data={data} navigationParams={navigationParams} />
      </TabsContent>
    </Tabs>
  )
}
