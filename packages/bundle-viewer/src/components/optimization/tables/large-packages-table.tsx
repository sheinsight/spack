/**
 * 大型依赖包表格
 */

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { formatBytes } from '@/lib/utils/format'
import { navigateTo } from '@/lib/navigation'

interface LargePackage {
  name: string
  version: string
  size: number
  moduleCount: number
  percentage: number
}

interface LargePackagesTableProps {
  largePackages: LargePackage[]
}

export function LargePackagesTable({ largePackages }: LargePackagesTableProps) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>大型依赖包</CardTitle>
        <CardDescription>最大的 20 个包 - 考虑使用替代方案或代码分割</CardDescription>
      </CardHeader>
      <CardContent>
        <Table maxHeight="h-108">
          <TableHeader>
            <TableRow>
              <TableHead>包名</TableHead>
              <TableHead>版本</TableHead>
              <TableHead>体积</TableHead>
              <TableHead>占比</TableHead>
              <TableHead>模块数</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
              {largePackages.map((pkg) => (
                <TableRow key={`${pkg.name}@${pkg.version}`}>
                  <TableCell
                    className="font-mono text-xs max-w-xs truncate cursor-pointer hover:text-primary transition-colors"
                    title={`${pkg.name}\n点击查看依赖包详情`}
                    onClick={() => navigateTo('packages', { search: pkg.name })}
                  >
                    {pkg.name}
                  </TableCell>
                  <TableCell>
                    <Badge variant="outline" className="text-xs font-mono">
                      {pkg.version}
                    </Badge>
                  </TableCell>
                  <TableCell className="font-medium">{formatBytes(pkg.size)}</TableCell>
                  <TableCell>
                    <div className="flex items-center gap-2">
                      <div className="h-2 w-20 rounded-full bg-muted overflow-hidden">
                        <div className="h-full bg-primary" style={{ width: `${Math.min(pkg.percentage, 100)}%` }} />
                      </div>
                      <span className="text-sm text-muted-foreground">{pkg.percentage.toFixed(1)}%</span>
                    </div>
                  </TableCell>
                  <TableCell className="text-muted-foreground">{pkg.moduleCount}</TableCell>
                </TableRow>
              ))}
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  )
}
