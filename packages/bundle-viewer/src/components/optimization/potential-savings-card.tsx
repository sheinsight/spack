/**
 * 潜在节省空间卡片 - 分来源展示优化机会
 */

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { formatBytes } from '@/lib/utils/format'

interface PotentialSavingsCardProps {
  duplicateSavings: number
  duplicatePackagesCount: number
  entryChunkSavings: number
  totalSize: number
}

export function PotentialSavingsCard({
  duplicateSavings,
  duplicatePackagesCount,
  entryChunkSavings,
  totalSize,
}: PotentialSavingsCardProps) {
  const totalSavings = duplicateSavings + entryChunkSavings
  const savingsPercent = totalSize > 0 ? Math.min((totalSavings / totalSize) * 100, 100) : 0

  return (
    <Card className="relative overflow-hidden flex flex-col">
      <div className="absolute top-0 right-0 w-40 h-40 bg-primary/5 rounded-full blur-3xl pointer-events-none" />

      <CardHeader>
        <CardTitle>潜在节省空间</CardTitle>
        <CardDescription>预估体积优化机会</CardDescription>
      </CardHeader>

      <CardContent className="flex-1 flex flex-col justify-center gap-6">
        {/* 总节省大数字 */}
        <div className="text-center">
          {totalSavings > 0 ? (
            <>
              <div className="text-4xl font-bold text-green-600 dark:text-green-400">{formatBytes(totalSavings)}</div>
              <p className="text-sm text-muted-foreground mt-1">预计可节省总体积</p>
            </>
          ) : (
            <>
              <div className="text-2xl font-semibold text-foreground">构建状态良好</div>
              <p className="text-sm text-muted-foreground mt-1">暂无明显优化空间</p>
            </>
          )}
        </div>

        {/* 进度条 */}
        {totalSavings > 0 && (
          <div>
            <div className="flex justify-between text-xs text-muted-foreground mb-1.5">
              <span>优化潜力</span>
              <span>{savingsPercent.toFixed(1)}% 总体积</span>
            </div>
            <div className="h-2 rounded-full bg-muted overflow-hidden">
              <div
                className="h-full bg-primary transition-all duration-500"
                style={{ width: `${savingsPercent}%`, opacity: 0.8 }}
              />
            </div>
          </div>
        )}

        {/* 来源分解 */}
        <div className="grid grid-cols-2 gap-3">
          {/* 重复依赖 */}
          <div className="rounded-lg bg-muted/50 p-3 text-center space-y-0.5">
            <div className="text-lg font-semibold text-foreground">
              {duplicateSavings > 0 ? formatBytes(duplicateSavings) : '—'}
            </div>
            <div className="text-xs font-medium text-foreground/70">重复依赖去重</div>
            <div className="text-xs text-muted-foreground">
              {duplicatePackagesCount > 0 ? `${duplicatePackagesCount} 个包受影响` : '无重复版本'}
            </div>
          </div>

          {/* Entry chunk */}
          <div className="rounded-lg bg-muted/50 p-3 text-center space-y-0.5">
            <div className="text-lg font-semibold text-foreground">
              {entryChunkSavings > 0 ? formatBytes(entryChunkSavings) : '—'}
            </div>
            <div className="text-xs font-medium text-foreground/70">Entry 体积压缩</div>
            <div className="text-xs text-muted-foreground">
              {entryChunkSavings > 0 ? '超出 244KB 部分' : 'Entry 体积达标'}
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}
