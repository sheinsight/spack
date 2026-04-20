/**
 * 优化行动建议 - 按优先级排序的优化建议及实施指南
 */

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
import { formatBytes } from '@/lib/utils/format'
import type { DuplicatePackage } from '@/lib/optimization-utils'
import { SIZE_THRESHOLDS } from '@/lib/constants'
import { cn } from '@/lib/utils'

interface OptimizationActionsProps {
  duplicatePackages: DuplicatePackage[]
  totalPotentialSavings: number
  chunkQuality: {
    largeEntryChunks: Array<{ id: string; size: number; names: string[] }>
    asyncChunkCount: number
  }
  largePackages: Array<{ name: string; size: number }>
}

interface ActionItemProps {
  priority: 'high' | 'medium'
  title: string
  description: string
  saving?: string
  steps: Array<{
    label: string
    content: React.ReactNode
  }>
}

function ActionItem({ priority, title, description, saving, steps }: ActionItemProps) {
  return (
    <Collapsible>
      <div className={cn('rounded-lg border p-4', priority === 'high' ? 'border-destructive/30' : 'border-border')}>
        {/* 头部 */}
        <div className="flex items-start justify-between gap-3">
          <div className="flex items-start gap-3 flex-1 min-w-0">
            <Badge variant={priority === 'high' ? 'destructive' : 'secondary'} className="text-xs shrink-0 mt-0.5">
              {priority === 'high' ? '高优先级' : '中等优先级'}
            </Badge>
            <div className="flex-1 min-w-0">
              <p className="font-medium text-sm">{title}</p>
              <p className="text-xs text-muted-foreground mt-0.5">{description}</p>
            </div>
          </div>
          {saving && (
            <Badge variant="outline" className="text-xs shrink-0 font-mono">
              可节省 {saving}
            </Badge>
          )}
        </div>

        {/* 展开触发器 */}
        <CollapsibleTrigger className="flex items-center gap-2 text-xs text-muted-foreground hover:text-foreground transition-colors mt-3">
          <span>▶</span>
          <span>查看实施步骤</span>
        </CollapsibleTrigger>

        {/* 展开内容 */}
        <CollapsibleContent>
          <div className="mt-3 space-y-3">
            {steps.map((step, idx) => (
              <div key={idx} className="pl-4 border-l-2 border-primary/30">
                <p className="text-xs font-semibold text-foreground mb-1.5">
                  Step {idx + 1}：{step.label}
                </p>
                {step.content}
              </div>
            ))}
          </div>
        </CollapsibleContent>
      </div>
    </Collapsible>
  )
}

function CodeBlock({ children }: { children: string }) {
  return (
    <div className="bg-muted/50 rounded-md p-3 border">
      <code className="text-xs whitespace-pre block">{children}</code>
    </div>
  )
}

function Note({ children }: { children: React.ReactNode }) {
  return <div className="bg-muted/30 rounded-md p-3 text-xs text-muted-foreground">{children}</div>
}

// ─────────────────────────────────────────────────────────────────────────────

export function OptimizationActions({
  duplicatePackages,
  totalPotentialSavings,
  chunkQuality,
  largePackages,
}: OptimizationActionsProps) {
  const hasDuplicates = duplicatePackages.length > 0
  const hasLargeEntryChunks = chunkQuality.largeEntryChunks.length > 0
  const hasLowAsyncChunks = chunkQuality.asyncChunkCount < 3
  const hasLargePackage = largePackages.some((p) => p.size > SIZE_THRESHOLDS.ENTRY_CHUNK_CRITICAL)

  const hasAnyIssue = hasDuplicates || hasLargeEntryChunks || hasLowAsyncChunks || hasLargePackage

  return (
    <Card>
      <CardHeader>
        <CardTitle>优化行动建议</CardTitle>
        <CardDescription>按优先级排序的优化建议及实施指南</CardDescription>
      </CardHeader>
      <CardContent>
        <div className="space-y-3">
          {/* 高优先级：解决重复依赖 */}
          {hasDuplicates && (
            <ActionItem
              priority="high"
              title="解决重复依赖"
              description={`检测到 ${duplicatePackages.length} 个包存在多个版本，影响总体积与维护性`}
              saving={formatBytes(totalPotentialSavings)}
              steps={[
                {
                  label: '运行依赖去重',
                  content: (
                    <div className="space-y-2">
                      <CodeBlock>npm dedupe</CodeBlock>
                      <CodeBlock>pnpm dedupe</CodeBlock>
                    </div>
                  ),
                },
                {
                  label: '重新安装依赖',
                  content: <CodeBlock>rm -rf node_modules && npm install</CodeBlock>,
                },
                {
                  label: '验证效果',
                  content: <Note>重新构建并对比体积变化，确认优化效果。</Note>,
                },
              ]}
            />
          )}

          {/* 高优先级：优化入口代码块 */}
          {hasLargeEntryChunks && (
            <ActionItem
              priority="high"
              title="优化入口代码块"
              description={`${chunkQuality.largeEntryChunks.length} 个入口代码块超过 244KB，严重影响 FCP`}
              steps={[
                {
                  label: '分割第三方库',
                  content: (
                    <Note>
                      配置代码分割（chunks: &apos;all&apos;），将第三方库提取为独立 vendors
                      chunk。具体方式请参考你使用的构建工具文档。
                    </Note>
                  ),
                },
                {
                  label: '路由使用动态导入',
                  content: (
                    <CodeBlock>{`const Dashboard = lazy(() => import('./pages/Dashboard'))
const Settings  = lazy(() => import('./pages/Settings'))`}</CodeBlock>
                  ),
                },
              ]}
            />
          )}

          {/* 中等优先级：增加懒加载 */}
          {hasLowAsyncChunks && (
            <ActionItem
              priority="medium"
              title="增加懒加载"
              description={`仅检测到 ${chunkQuality.asyncChunkCount} 个异步代码块，增加懒加载可降低初始加载体积`}
              steps={[
                {
                  label: 'React 路由懒加载',
                  content: (
                    <CodeBlock>{`import { lazy, Suspense } from 'react'

const Dashboard = lazy(() => import('./pages/Dashboard'))`}</CodeBlock>
                  ),
                },
              ]}
            />
          )}

          {/* 中等优先级：检查大型依赖 */}
          {hasLargePackage && (
            <ActionItem
              priority="medium"
              title="替换超大依赖"
              description={`${largePackages
                .filter((p) => p.size > SIZE_THRESHOLDS.ENTRY_CHUNK_CRITICAL)
                .map((p) => p.name)
                .join('、')} 体积超过 500KB，建议寻找轻量替代`}
              steps={[
                {
                  label: '常见轻量替代方案',
                  content: (
                    <CodeBlock>{`moment.js  (289 KB) → dayjs        (7 KB)
lodash     (531 KB) → lodash-es  (按需引入)
axios       (31 KB) → 原生 fetch API`}</CodeBlock>
                  ),
                },
              ]}
            />
          )}

          {/* 无问题状态 */}
          {!hasAnyIssue && (
            <div className="flex items-start gap-3 p-4 rounded-lg bg-muted/50">
              <Badge className="bg-green-600 hover:bg-green-600 text-white shrink-0 mt-0.5">优化良好</Badge>
              <div>
                <p className="text-sm font-medium">构建配置优化良好</p>
                <p className="text-xs text-muted-foreground mt-0.5">
                  暂无明显优化问题，请持续监控构建体积，防止性能退化。
                </p>
              </div>
            </div>
          )}
        </div>
      </CardContent>
    </Card>
  )
}
