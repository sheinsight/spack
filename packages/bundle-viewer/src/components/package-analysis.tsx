import { useEffect, useMemo, useState } from 'react'
import type { StandardBundleData } from '@/types/bundle-data-standard'
import { analyzeCodeDistribution, filterPackageGroups, groupPackagesByName } from '@/lib/package-analyzer'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
import { PercentageBar } from '@/components/ui/percentage-bar'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { formatBytes, formatNumber } from '@/lib/utils/format'
import { BADGE_STYLES } from '@/lib/constants'
import { PackageCard } from './package-card'
import type { NavigationParams } from '@/lib/navigation'

interface PackageAnalysisProps {
  data: StandardBundleData
  navigationParams?: NavigationParams
}

export function PackageAnalysis({ data, navigationParams }: PackageAnalysisProps) {
  const [searchQuery, setSearchQuery] = useState(navigationParams?.search || '')
  const [singleVersionExpanded, setSingleVersionExpanded] = useState(true)

  // 响应 URL 参数变化
  useEffect(() => {
    if (navigationParams?.search !== undefined) {
      setSearchQuery(navigationParams.search)
    }
  }, [navigationParams])

  // 分析包分组
  const packageGroups = useMemo(() => groupPackagesByName(data.packages), [data])

  // 分析代码占比
  const codeDistribution = useMemo(() => analyzeCodeDistribution(data), [data])

  // 分析 chunk 占比 (for future use)
  // const chunkDistribution = useMemo(() => analyzeChunkDistribution(data), [data])

  // 过滤包列表
  const filteredGroups = useMemo(() => filterPackageGroups(packageGroups, searchQuery), [packageGroups, searchQuery])

  // 分离多版本和单版本包
  const multiVersionPackages = filteredGroups.filter((g) => g.versionCount > 1)
  const singleVersionPackages = filteredGroups.filter((g) => g.versionCount === 1)

  // 计算 Module 百分比
  const moduleTotalSize = codeDistribution.thirdPartySize + codeDistribution.projectSize
  const moduleThirdPartyPercent = moduleTotalSize > 0 ? (codeDistribution.thirdPartySize / moduleTotalSize) * 100 : 0
  const moduleProjectPercent = moduleTotalSize > 0 ? (codeDistribution.projectSize / moduleTotalSize) * 100 : 0

  const packageStats = useMemo(() => {
    const allVersions = packageGroups.flatMap((group) => group.versions)
    const largePackageCount = allVersions.filter((v) => v.size >= 1024 * 1024).length
    const multiVersionCount = packageGroups.filter((g) => g.versionCount > 1).length
    const singleVersionCount = packageGroups.filter((g) => g.versionCount === 1).length

    return {
      totalPackages: packageGroups.length,
      largePackageCount,
      multiVersionCount,
      singleVersionCount,
    }
  }, [packageGroups])

  // 大型依赖包（按大小排序，前 20 个）
  const largePackages = useMemo(() => {
    const allPackages = packageGroups
      .flatMap((group) =>
        group.versions.map((v) => ({
          name: group.name,
          version: v.version,
          size: v.size,
          moduleCount: v.moduleCount,
          packageJsonPath: v.packageJsonPath,
          percentage: moduleTotalSize > 0 ? (v.size / moduleTotalSize) * 100 : 0,
        })),
      )
      .toSorted((a, b) => b.size - a.size)
      .slice(0, 20)
    return allPackages
  }, [packageGroups, moduleTotalSize])

  // 计算 Chunk 百分比 (for future use)
  // const chunkTotalSize = chunkDistribution.thirdPartySize + chunkDistribution.projectSize
  // const chunkThirdPartyPercent =
  //   chunkTotalSize > 0 ? (chunkDistribution.thirdPartySize / chunkTotalSize) * 100 : 0
  // const chunkProjectPercent =
  //   chunkTotalSize > 0 ? (chunkDistribution.projectSize / chunkTotalSize) * 100 : 0

  return (
    <div className="space-y-6 animate-fade-in">
      {/* 依赖包概览 */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-xs font-medium text-muted-foreground">依赖包总数</CardTitle>
          </CardHeader>
          <CardContent className="pb-3">
            <div className="flex items-baseline gap-2">
              <p className="text-2xl font-bold">{formatNumber(packageStats.totalPackages)}</p>
              <div className="flex items-center gap-1">
                <Badge variant="outline" className="text-xs">
                  {formatNumber(packageStats.largePackageCount)} 个大依赖（超过 1MB）
                </Badge>
              </div>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-xs font-medium text-muted-foreground">多版本依赖数</CardTitle>
          </CardHeader>
          <CardContent className="pb-3">
            <div className="flex items-baseline gap-2">
              <p className="text-2xl font-bold">{formatNumber(packageStats.multiVersionCount)}</p>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-xs font-medium text-muted-foreground">单版本依赖数</CardTitle>
          </CardHeader>
          <CardContent className="pb-3">
            <p className="text-2xl font-bold">{formatNumber(packageStats.singleVersionCount)}</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-xs font-medium text-muted-foreground">依赖包总体积</CardTitle>
          </CardHeader>
          <CardContent className="pb-3">
            <div className="flex items-baseline gap-2">
              <p className="text-2xl font-bold">{formatBytes(codeDistribution.thirdPartySize)}</p>
              <Badge variant="outline" className="text-xs">
                {moduleThirdPartyPercent.toFixed(1)}%
              </Badge>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* 代码占比概览 */}
      <Card>
        <CardHeader>
          <CardTitle>代码分布</CardTitle>
          <CardDescription>第三方依赖包与项目源码占比（点击图例可切换显示）</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <PercentageBar
            items={[
              {
                label: '第三方依赖',
                value: `${formatBytes(codeDistribution.thirdPartySize)} · ${formatNumber(codeDistribution.thirdPartyModules)} 个模块`,
                percent: moduleThirdPartyPercent,
                variant: 'viz-2', // 碧绿色
              },
              {
                label: '项目源码',
                value: `${formatBytes(codeDistribution.projectSize)} · ${formatNumber(codeDistribution.projectModules)} 个模块`,
                percent: moduleProjectPercent,
                variant: 'viz-1', // 钴蓝色
              },
            ]}
            height="md"
            colorMode="distinct"
            showAnimation={true}
            showLegend={true}
            legendPosition="bottom"
            legendLayout="grid"
            legendColumns={2}
          />
        </CardContent>
      </Card>

      {/* 大型依赖包 */}
      <Card>
        <CardHeader>
          <CardTitle>大型依赖包</CardTitle>
          <CardDescription>最大的 20 个依赖包实例 - 考虑使用替代方案或优化引入方式</CardDescription>
        </CardHeader>
        <CardContent>
          <Table maxHeight="h-62.5">
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
              {largePackages.map((pkg, idx) => (
                <TableRow key={`${pkg.name}-${pkg.version}-${idx}`}>
                  <TableCell className="font-mono text-xs max-w-xs truncate" title={pkg.name}>
                    {pkg.name}
                  </TableCell>
                  <TableCell className="font-mono text-xs">{pkg.version}</TableCell>
                  <TableCell>
                    <Badge
                      variant={pkg.size > 1024 * 1024 ? 'destructive' : pkg.size > 500 * 1024 ? 'default' : 'secondary'}
                      className={BADGE_STYLES.SIZE}
                    >
                      {formatBytes(pkg.size)}
                    </Badge>
                  </TableCell>
                  <TableCell className="text-muted-foreground text-xs font-mono">
                    {pkg.percentage.toFixed(1)}%
                  </TableCell>
                  <TableCell className="text-muted-foreground text-xs font-mono">
                    {formatNumber(pkg.moduleCount)}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </CardContent>
      </Card>

      {/* 搜索框 */}
      <div className="flex items-center gap-4">
        <div className="relative max-w-md flex-1">
          <Input
            type="search"
            placeholder="搜索依赖包..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="pr-24"
          />
          {searchQuery && (
            <div className="absolute right-2 top-1/2 -translate-y-1/2 flex items-center gap-1">
              <span className="text-xs text-muted-foreground">
                {filteredGroups.length} / {packageGroups.length}
              </span>
              <button
                onClick={() => setSearchQuery('')}
                className="text-muted-foreground hover:text-foreground transition-colors p-1"
                title="清除搜索"
              >
                ✕
              </button>
            </div>
          )}
        </div>
        {!searchQuery && <span className="text-sm text-muted-foreground">{filteredGroups.length} 个包</span>}
      </div>

      {/* 多版本包列表 */}
      {multiVersionPackages.length > 0 && (
        <div className="space-y-3">
          <div className="flex items-center gap-1.5">
            <h2 className="text-lg font-semibold">多版本依赖</h2>
            <Badge variant="destructive" className={BADGE_STYLES.NUMERIC}>
              {multiVersionPackages.length}
            </Badge>
          </div>
          <div className="grid gap-3">
            {multiVersionPackages.map((group) => (
              <PackageCard
                key={group.name}
                group={group}
                allPackages={data.packages}
                allModules={data.modules}
                data={data}
              />
            ))}
          </div>
        </div>
      )}

      {/* 单版本包列表（折叠） */}
      {singleVersionPackages.length > 0 && (
        <Collapsible open={singleVersionExpanded} onOpenChange={setSingleVersionExpanded}>
          <Card>
            <CardHeader className="pb-3">
              <div className="flex items-center justify-between">
                <CardTitle className="text-base">单版本依赖包</CardTitle>
                <div className="flex items-center gap-1.5">
                  <Badge variant="outline" className={BADGE_STYLES.NUMERIC}>
                    {singleVersionPackages.length} 个包
                  </Badge>
                  <CollapsibleTrigger asChild>
                    <Button variant="ghost" size="sm">
                      {singleVersionExpanded ? '▼ 收起' : '▶ 展开'}
                    </Button>
                  </CollapsibleTrigger>
                </div>
              </div>
            </CardHeader>
            <CollapsibleContent>
              <CardContent className="space-y-3 pt-0">
                {singleVersionPackages.map((group) => (
                  <PackageCard
                    key={group.name}
                    group={group}
                    allPackages={data.packages}
                    allModules={data.modules}
                    data={data}
                  />
                ))}
              </CardContent>
            </CollapsibleContent>
          </Card>
        </Collapsible>
      )}

      {/* 无结果提示 */}
      {filteredGroups.length === 0 && (
        <Card>
          <CardContent className="py-12 text-center">
            <p className="text-muted-foreground">未找到匹配 "{searchQuery}" 的依赖包</p>
          </CardContent>
        </Card>
      )}
    </div>
  )
}
