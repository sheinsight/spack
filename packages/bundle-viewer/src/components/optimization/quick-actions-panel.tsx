/**
 * 行动建议面板
 */

import { useMemo, useState } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
import { cn } from '@/lib/utils'
import { navigateTo, scrollToElement, type TabId } from '@/lib/navigation'
import type { PackageManager } from '@/lib/optimization-utils'

interface QuickAction {
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
}

interface QuickActionsPanelProps {
  actions: QuickAction[]
}

function ActionItem({ action }: { action: QuickAction }) {
  const [open, setOpen] = useState(false)

  return (
    <div className="p-4 rounded-lg bg-background border border-border">
      {/* 头部：优先级 + 标题 + 预估 */}
      <div className="flex items-start gap-3">
        <Badge
          variant={action.priority === 'P0' ? 'destructive' : action.priority === 'P1' ? 'default' : 'secondary'}
          className="shrink-0 mt-0.5"
        >
          {action.priority}
        </Badge>
        <div className="flex-1 min-w-0">
          <div className="flex items-center justify-between gap-4">
            <h4 className="font-semibold text-sm">{action.title}</h4>
            <div className="flex items-center gap-2 text-xs text-muted-foreground shrink-0">
              <span>耗时 {action.estimatedTime}</span>
              {action.estimatedSaving && (
                <>
                  <span>•</span>
                  <span className="text-green-600 dark:text-green-400">节省 {action.estimatedSaving}</span>
                </>
              )}
            </div>
          </div>
          <div className="grid md:grid-cols-2 gap-x-4 gap-y-0.5 mt-1 text-xs text-muted-foreground">
            <span>
              <span className="text-foreground/50">问题：</span>
              {action.problem}
            </span>
            <span>
              <span className="text-foreground/50">影响：</span>
              {action.impact}
            </span>
          </div>
        </div>
      </div>

      {/* 优化建议说明 + 折叠步骤 */}
      {action.steps.length > 0 && (
        <div className="mt-3 pt-3 border-t">
          {/* 简要优化说明 */}
          <p className="text-xs text-muted-foreground mb-2.5">{action.suggestion}</p>

          {action.target && (
            <button
              className="mb-2 inline-flex items-center gap-1.5 text-xs text-primary hover:text-primary/80 transition-colors"
              onClick={() => {
                navigateTo(action.target!.tab, { highlight: action.target!.highlight })
                scrollToElement(action.target!.sectionId, { behavior: 'smooth' })
              }}
            >
              <TargetIcon className="h-3.5 w-3.5" />
              <span>{action.ctaLabel || '定位问题'}</span>
            </button>
          )}

          {/* 折叠步骤 */}
          <Collapsible open={open} onOpenChange={setOpen}>
            <CollapsibleTrigger className="flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors">
              <ChevronIcon className={cn('h-3 w-3 transition-transform duration-200', open && 'rotate-90')} />
              <span>{open ? '收起步骤' : `查看实施步骤（${action.steps.length} 步）`}</span>
            </CollapsibleTrigger>

            <CollapsibleContent>
              <div className="mt-2.5 space-y-2.5">
                {action.steps.map((step, idx) => (
                  <div key={idx} className="pl-3 border-l-2 border-primary/30">
                    <p className="text-xs font-semibold text-foreground mb-1">
                      Step {idx + 1}：{step.label}
                    </p>
                    {step.code && (
                      <div className="bg-muted/50 rounded-md p-2.5 border">
                        <code className="text-xs whitespace-pre block">{step.code}</code>
                      </div>
                    )}
                    {step.note && (
                      <p className="text-xs text-muted-foreground bg-muted/30 rounded-md p-2.5">{step.note}</p>
                    )}
                  </div>
                ))}
              </div>
            </CollapsibleContent>
          </Collapsible>
        </div>
      )}
    </div>
  )
}

export function QuickActionsPanel({ actions }: QuickActionsPanelProps) {
  if (actions.length === 0) return null

  const grouped = useMemo(() => {
    const byPriority: Record<'P0' | 'P1' | 'P2', QuickAction[]> = { P0: [], P1: [], P2: [] }
    for (const action of actions) {
      byPriority[action.priority].push(action)
    }
    return byPriority
  }, [actions])

  const [openGroups, setOpenGroups] = useState({
    P0: true,
    P1: true,
    P2: false,
  })

  return (
    <Card className="border-primary/30 bg-primary/5">
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          行动建议
          <Badge variant="destructive" className="text-xs">
            {actions.length} 项待优化
          </Badge>
        </CardTitle>
        <CardDescription>按优先级排序，建议依次执行</CardDescription>
      </CardHeader>

      <CardContent className="space-y-3">
        {(['P0', 'P1', 'P2'] as const).map((priority) => {
          const items = grouped[priority]
          if (items.length === 0) return null
          return (
            <Collapsible
              key={priority}
              open={openGroups[priority]}
              onOpenChange={(open) => setOpenGroups((prev) => ({ ...prev, [priority]: open }))}
            >
              <div className="flex items-center justify-between gap-2 border rounded-md px-3 py-2 bg-background">
                <div className="flex items-center gap-2">
                  <Badge
                    variant={priority === 'P0' ? 'destructive' : priority === 'P1' ? 'default' : 'secondary'}
                    className="text-xs"
                  >
                    {priority}
                  </Badge>
                  <span className="text-sm font-medium">优先级 {priority}</span>
                  <Badge variant="outline" className="text-xs">
                    {items.length} 项
                  </Badge>
                </div>
                <CollapsibleTrigger className="text-xs text-muted-foreground hover:text-foreground transition-colors">
                  {openGroups[priority] ? '收起' : '展开'}
                </CollapsibleTrigger>
              </div>
              <CollapsibleContent>
                <div className="mt-3 space-y-3">
                  {items.map((action, index) => (
                    <ActionItem key={`${priority}-${index}`} action={action} />
                  ))}
                </div>
              </CollapsibleContent>
            </Collapsible>
          )
        })}

        <div className="pt-2 border-t text-xs text-muted-foreground">完成后重新构建，对比前后体积变化以确认效果。</div>
      </CardContent>
    </Card>
  )
}

function ChevronIcon({ className = '' }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 16 16" fill="none" aria-hidden="true">
      <path d="M6 4L10 8L6 12" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  )
}

function TargetIcon({ className = '' }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 16 16" fill="none" aria-hidden="true">
      <circle cx="8" cy="8" r="6" stroke="currentColor" strokeWidth="1.5" />
      <circle cx="8" cy="8" r="2.5" stroke="currentColor" strokeWidth="1.5" />
    </svg>
  )
}
