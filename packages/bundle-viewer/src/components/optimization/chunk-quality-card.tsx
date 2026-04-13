/**
 * 代码分割质量卡片
 */

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { formatBytes } from '@/lib/utils/format'
import { navigateTo } from '@/lib/navigation'
import { cn } from '@/lib/utils'

interface ChunkQualityCardProps {
  chunkQuality: {
    entrySize: number
    asyncSize: number
    totalSize: number
    entryPercentage: number
    asyncPercentage: number
    largeEntryChunks: Array<{ id: string; size: number; names: string[] }>
    tooManyModulesChunks: Array<{ id: string; moduleIds: number[] }>
    asyncChunkCount: number
  }
}

export function ChunkQualityCard({ chunkQuality }: ChunkQualityCardProps) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>代码分割质量</CardTitle>
        <CardDescription>代码块分布分析</CardDescription>
      </CardHeader>
      <CardContent>
        <div className="space-y-4">
          <div className="grid gap-4 md:grid-cols-3">
            <div className="space-y-2">
              <p className="text-sm text-muted-foreground">入口代码块</p>
              <p className="text-2xl font-bold">{formatBytes(chunkQuality.entrySize)}</p>
              <div className="h-2 rounded-full bg-muted overflow-hidden">
                <div
                  className={cn('h-full', chunkQuality.entryPercentage > 50 ? 'bg-destructive' : 'bg-primary')}
                  style={{ width: `${chunkQuality.entryPercentage}%` }}
                />
              </div>
              <p className="text-xs text-muted-foreground">占总体积 {chunkQuality.entryPercentage.toFixed(1)}%</p>
            </div>

            <div className="space-y-2">
              <p className="text-sm text-muted-foreground">异步代码块</p>
              <p className="text-2xl font-bold">{formatBytes(chunkQuality.asyncSize)}</p>
              <div className="h-2 rounded-full bg-muted overflow-hidden">
                <div
                  className="h-full bg-primary"
                  style={{ width: `${chunkQuality.asyncPercentage}%`, opacity: 0.6 }}
                />
              </div>
              <p className="text-xs text-muted-foreground">占总体积 {chunkQuality.asyncPercentage.toFixed(1)}%</p>
            </div>

            <div className="space-y-2">
              <p className="text-sm text-muted-foreground">懒加载</p>
              <p className="text-2xl font-bold">{chunkQuality.asyncChunkCount}</p>
              <p className="text-xs text-muted-foreground">个异步代码块</p>
              {chunkQuality.asyncChunkCount < 3 && (
                <Badge variant="destructive" className="text-xs">
                  建议增加懒加载
                </Badge>
              )}
            </div>
          </div>

          {chunkQuality.largeEntryChunks.length > 0 && (
            <div className="pt-4 border-t">
              <p className="text-sm font-medium mb-2 text-destructive">
                {chunkQuality.largeEntryChunks.length} 个入口代码块超过 244KB：
              </p>
              <ul className="space-y-1">
                {chunkQuality.largeEntryChunks.slice(0, 5).map((chunk) => (
                  <li
                    key={chunk.id}
                    className="text-sm text-muted-foreground flex items-center gap-2 cursor-pointer hover:text-primary transition-colors"
                    onClick={() => navigateTo('chunks', { chunkId: chunk.id })}
                    title="点击查看代码块详情"
                  >
                    <span className="font-mono text-xs">{chunk.names.join(', ') || chunk.id}</span>
                    <span className="font-medium">{formatBytes(chunk.size)}</span>
                  </li>
                ))}
              </ul>
            </div>
          )}
        </div>
      </CardContent>
    </Card>
  )
}
