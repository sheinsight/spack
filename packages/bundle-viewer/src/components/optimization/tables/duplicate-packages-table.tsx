/**
 * 重复依赖表格
 */

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { formatBytes } from '@/lib/utils/format'
import { navigateTo } from '@/lib/navigation'
import type { DuplicatePackage } from '@/lib/optimization-utils'

interface DuplicatePackagesTableProps {
  duplicatePackages: DuplicatePackage[]
  totalPotentialSavings: number
  highlightNames?: string[]
}

export function DuplicatePackagesTable({
  duplicatePackages,
  totalPotentialSavings,
  highlightNames = [],
}: DuplicatePackagesTableProps) {
  if (duplicatePackages.length === 0) return null

  return (
    <Card id="duplicate-packages-section">
      <CardHeader>
        <div className="space-y-3">
          <div>
            <CardTitle className="flex items-center gap-2">
              重复依赖分析
              <Badge variant="destructive">{duplicatePackages.length}</Badge>
            </CardTitle>
            <CardDescription>
              存在多个版本的包 - 自动 dedupe 可节省约 {formatBytes(totalPotentialSavings)} （需确认兼容性）
            </CardDescription>
          </div>

          {/* 计算说明 */}
          <div className="bg-muted/30 p-3 rounded-lg space-y-2 text-xs">
            <p className="font-semibold text-foreground flex items-center gap-2">
              <InfoIcon className="h-4 w-4 text-muted-foreground" />
              <span>潜在节省计算规则</span>
            </p>
            <ul className="space-y-1 text-muted-foreground">
              <li className="flex items-start gap-2">
                <CheckIcon className="mt-0.5 h-3.5 w-3.5 text-emerald-600 shrink-0" />
                <span>
                  <strong className="text-foreground">可自动优化</strong>：相同主版本（如 1.2.0 和 1.3.0）
                  <br />
                  <span className="text-xs">
                    → 可通过 <code className="bg-muted px-1 rounded">npm dedupe</code> 自动合并，保留最大版本
                  </span>
                </span>
              </li>
              <li className="flex items-start gap-2">
                <AlertIcon className="mt-0.5 h-3.5 w-3.5 text-amber-500 shrink-0" />
                <span>
                  <strong className="text-foreground">需人工评估</strong>：不同主版本（如 1.x 和 2.x）
                  <br />
                  <span className="text-xs">→ 可能存在 breaking changes，需检查 CHANGELOG 和依赖链</span>
                </span>
              </li>
            </ul>
            <p className="text-xs text-muted-foreground pt-1 border-t border-border/50">
              节省空间 = 所有版本总和 - 保留的最大版本
            </p>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        <Table maxHeight="h-108">
          <TableHeader>
            <TableRow>
              <TableHead>包名</TableHead>
              <TableHead>版本分布</TableHead>
              <TableHead>总体积</TableHead>
              <TableHead>可节省</TableHead>
              <TableHead>状态</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {duplicatePackages.map((pkg) => {
              const isHighlighted = highlightNames.includes(pkg.name)
              return (
                <TableRow
                  key={pkg.name}
                  className={isHighlighted ? 'bg-primary/10 ring-1 ring-primary/30' : undefined}
                >
                  <TableCell
                    className="font-mono text-xs max-w-xs truncate cursor-pointer hover:text-primary transition-colors"
                    title={`${pkg.name}\n点击查看依赖包详情`}
                    onClick={() => navigateTo('packages', { search: pkg.name })}
                  >
                    {pkg.name}
                  </TableCell>
                  <TableCell>
                    <div className="flex flex-wrap gap-1">
                      {pkg.versions.map((v, idx) => (
                        <Badge
                          key={idx}
                          variant="outline"
                          className="text-xs font-mono"
                          title={`体积: ${formatBytes(v.size)}`}
                        >
                          {v.version} ({formatBytes(v.size)})
                        </Badge>
                      ))}
                    </div>
                  </TableCell>
                  <TableCell className="font-medium">{formatBytes(pkg.totalSize)}</TableCell>
                  <TableCell>
                    {pkg.potentialSavings > 0 ? (
                      <span className="text-green-600 dark:text-green-400 font-medium">
                        -{formatBytes(pkg.potentialSavings)}
                      </span>
                    ) : (
                      <span className="text-muted-foreground text-xs">需人工评估</span>
                    )}
                  </TableCell>
                  <TableCell>
                    {pkg.canDedupe ? (
                      <Badge variant="default" className="text-xs">
                        可 dedupe
                      </Badge>
                    ) : pkg.hasDifferentMajors ? (
                      <Badge variant="secondary" className="text-xs" title={`${pkg.majorVersionCount} 个主版本`}>
                        需评估
                      </Badge>
                    ) : (
                      <Badge variant="outline" className="text-xs">
                        -
                      </Badge>
                    )}
                  </TableCell>
                </TableRow>
              )
            })}
          </TableBody>
        </Table>
        <div className="mt-4 p-3 rounded-lg bg-muted/50 text-xs text-muted-foreground">
          <p className="font-medium mb-1">说明：</p>
          <ul className="space-y-1 ml-4 list-disc">
            <li>
              <strong>可 dedupe</strong>：相同主版本的不同次版本，可通过{' '}
              <code className="bg-background px-1 rounded">npm/yarn/pnpm dedupe</code> 自动合并
            </li>
            <li>
              <strong>需评估</strong>：存在不同主版本，可能有破坏性更新（breaking changes），需人工确认是否可升级统一
            </li>
          </ul>
        </div>
      </CardContent>
    </Card>
  )
}

function InfoIcon({ className = '' }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 16 16" fill="none" aria-hidden="true">
      <circle cx="8" cy="8" r="6.5" stroke="currentColor" strokeWidth="1.2" />
      <path d="M8 7V11" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" />
      <circle cx="8" cy="5" r="0.8" fill="currentColor" />
    </svg>
  )
}

function CheckIcon({ className = '' }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 16 16" fill="none" aria-hidden="true">
      <path d="M3.5 8.5L6.5 11.5L12.5 4.5" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" />
    </svg>
  )
}

function AlertIcon({ className = '' }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 16 16" fill="none" aria-hidden="true">
      <path
        d="M8 2.8L14 13.2H2L8 2.8Z"
        stroke="currentColor"
        strokeWidth="1.2"
        strokeLinejoin="round"
      />
      <path d="M8 6.5V9.5" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" />
      <circle cx="8" cy="11.6" r="0.7" fill="currentColor" />
    </svg>
  )
}
