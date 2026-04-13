/**
 * 优化评分卡片
 */

import { useState } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
import { CircularProgress } from './circular-progress'
import { getScoreLabel } from '@/lib/optimization-utils'
import { cn } from '@/lib/utils'

interface OptimizationScoreCardProps {
  score: number
}

export function OptimizationScoreCard({ score }: OptimizationScoreCardProps) {
  const [open, setOpen] = useState(false)

  return (
    <Card className="flex flex-col">
      <CardHeader>
        <div className="space-y-3">
          <div>
            <CardTitle className="flex items-center gap-2">
              优化评分
              <span
                className="text-xs text-muted-foreground cursor-help font-normal"
                title="评分基于构建体积、代码分割质量和依赖管理。分数越高表示优化越好。"
              >
                ⓘ
              </span>
            </CardTitle>
            <CardDescription>基于 Core Web Vitals 影响的整体构建质量评估</CardDescription>
          </div>

          {/* 评分依据说明 */}
          <Collapsible open={open} onOpenChange={setOpen}>
            <CollapsibleTrigger className="flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors">
              <span className={cn('transition-transform duration-200', open && 'rotate-90')}>▶</span>
              <span>{open ? '收起评分依据' : '查看评分依据'}</span>
            </CollapsibleTrigger>
            <CollapsibleContent className="mt-3">
              <div className="space-y-2 text-xs text-muted-foreground bg-muted/30 p-3 rounded-lg">
                <p className="font-semibold text-foreground mb-2">评分权重说明（总分 100）</p>
                <ul className="space-y-1.5">
                  <li className="flex justify-between">
                    <span>• Entry chunk 大小</span>
                    <span className="font-mono">最高 30 分</span>
                  </li>
                  <li className="text-xs pl-4 text-muted-foreground">直接影响 FCP/LCP，基于 244KB 基准（3G 网络）</li>

                  <li className="flex justify-between pt-1">
                    <span>• 重复依赖</span>
                    <span className="font-mono">最高 25 分</span>
                  </li>
                  <li className="text-xs pl-4 text-muted-foreground">影响总体积和维护性</li>

                  <li className="flex justify-between pt-1">
                    <span>• Entry 占比</span>
                    <span className="font-mono">最高 20 分</span>
                  </li>
                  <li className="text-xs pl-4 text-muted-foreground">反映代码分割质量，建议 &lt;50%</li>

                  <li className="flex justify-between pt-1">
                    <span>• 懒加载利用率</span>
                    <span className="font-mono">最高 15 分</span>
                  </li>
                  <li className="text-xs pl-4 text-muted-foreground">异步 chunk 数量，建议 ≥3 个</li>

                  <li className="flex justify-between pt-1">
                    <span>• 超大包检测</span>
                    <span className="font-mono">最高 10 分</span>
                  </li>
                  <li className="text-xs pl-4 text-muted-foreground">单包 &gt;1MB 会扣分</li>
                </ul>
                <p className="text-xs pt-2 border-t border-border/50 mt-2">
                  参考：
                  <a
                    href="https://web.dev/performance-budgets-101/"
                    target="_blank"
                    rel="noopener"
                    className="text-primary hover:underline"
                  >
                    Web.dev Performance Budgets
                  </a>
                </p>
              </div>
            </CollapsibleContent>
          </Collapsible>
        </div>
      </CardHeader>
      <CardContent className="flex-1 flex flex-col justify-center">
        <div className="flex flex-col items-center gap-3 py-2">
          <CircularProgress score={score} />
          <Badge
            variant={score >= 80 ? 'default' : score >= 60 ? 'secondary' : 'destructive'}
            className="text-sm px-4 py-1"
          >
            {getScoreLabel(score)}
          </Badge>
        </div>
      </CardContent>
    </Card>
  )
}
