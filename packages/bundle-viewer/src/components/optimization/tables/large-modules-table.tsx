/**
 * 大型模块表格
 */

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { formatBytes } from '@/lib/utils/format'
import { navigateTo } from '@/lib/navigation'

interface LargeModule {
  name: string
  displayName: string
  size: number
  chunkIds: string[]
  type: string
  isNodeModule: boolean
  percentage: number
}

interface LargeModulesTableProps {
  largeModules: LargeModule[]
}

export function LargeModulesTable({ largeModules }: LargeModulesTableProps) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>大型模块</CardTitle>
        <CardDescription>最大的 20 个模块 - 考虑懒加载或优化</CardDescription>
      </CardHeader>
      <CardContent>
        <Table maxHeight="h-108">
          <TableHeader>
            <TableRow>
              <TableHead>模块</TableHead>
              <TableHead>体积</TableHead>
              <TableHead>占比</TableHead>
              <TableHead>代码块</TableHead>
              <TableHead>类型</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
              {largeModules.map((module, idx) => (
                <TableRow key={idx}>
                  <TableCell
                    className="font-mono text-xs max-w-md truncate cursor-pointer hover:text-primary transition-colors"
                    title={`${module.displayName}\n点击查看模块详情`}
                    onClick={() => navigateTo('modules', { moduleName: module.name })}
                  >
                    {module.displayName}
                  </TableCell>
                  <TableCell className="font-medium">{formatBytes(module.size)}</TableCell>
                  <TableCell>
                    <div className="flex items-center gap-2">
                      <div className="h-2 w-20 rounded-full bg-muted overflow-hidden">
                        <div className="h-full bg-primary" style={{ width: `${Math.min(module.percentage, 100)}%` }} />
                      </div>
                      <span className="text-sm text-muted-foreground">{module.percentage.toFixed(1)}%</span>
                    </div>
                  </TableCell>
                  <TableCell className="text-muted-foreground">{module.chunkIds.length}</TableCell>
                  <TableCell>
                    {module.isNodeModule ? (
                      <Badge variant="secondary" className="text-xs">
                        npm
                      </Badge>
                    ) : (
                      <Badge variant="outline" className="text-xs">
                        source
                      </Badge>
                    )}
                  </TableCell>
                </TableRow>
              ))}
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  )
}
