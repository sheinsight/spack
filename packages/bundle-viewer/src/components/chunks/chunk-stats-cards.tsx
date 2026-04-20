/**
 * Chunk 统计卡片组
 */

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { cn } from '@/lib/utils'

interface ChunkStatsCardsProps {
  stats: {
    entryCount: number
    asyncCount: number
    maxDepth: number
    avgChildren: number
    hasCircular: boolean
  }
  circularPathsCount: number
}

export function ChunkStatsCards({ stats, circularPathsCount }: ChunkStatsCardsProps) {
  return (
    <div className="grid gap-3 md:grid-cols-5">
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-xs font-medium text-muted-foreground">Entry 代码块</CardTitle>
        </CardHeader>
        <CardContent className="pb-3">
          <div className="text-2xl font-bold text-foreground font-mono">{stats.entryCount}</div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-xs font-medium text-muted-foreground">异步代码块</CardTitle>
        </CardHeader>
        <CardContent className="pb-3">
          <div className="text-2xl font-bold text-foreground font-mono">{stats.asyncCount}</div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-xs font-medium text-muted-foreground">最大依赖深度</CardTitle>
        </CardHeader>
        <CardContent className="pb-3">
          <div className="text-2xl font-bold font-mono">{stats.maxDepth}</div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-xs font-medium text-muted-foreground">平均子代码块数</CardTitle>
        </CardHeader>
        <CardContent className="pb-3">
          <div className="text-2xl font-bold font-mono">{stats.avgChildren.toFixed(1)}</div>
        </CardContent>
      </Card>

      <Card className={stats.hasCircular ? 'border-l-4 border-l-destructive' : ''}>
        <CardHeader className="pb-2">
          <CardTitle className="text-xs font-medium text-muted-foreground">循环依赖</CardTitle>
        </CardHeader>
        <CardContent className="pb-3">
          <div className={cn('text-2xl font-bold', stats.hasCircular ? 'text-destructive' : 'text-green-600')}>
            {stats.hasCircular ? `${circularPathsCount}处` : '✓ 无'}
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
