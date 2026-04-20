import { useMemo, useState } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { PercentageBar } from '@/components/ui/percentage-bar'
import { analyzeCodeDistribution } from '@/lib/package-analyzer'
import { formatTime, formatNumber, safePercentage, safeToFixed } from '@/lib/utils/format'
import { formatBytes, getSizeBadgeVariant } from '@/lib/utils/format'
import { getModuleName } from '@/lib/utils/path-utils'
import { getFileExtension, getAssetType, getAssetTypeName } from '@/lib/utils/file-utils'
import { BADGE_STYLES } from '@/lib/constants'
import { navigateTo, type NavigationParams } from '@/lib/navigation'

import type { StandardBundleData } from '@/types/bundle-data-standard'

interface SummaryProps {
  data: StandardBundleData
  navigationParams?: NavigationParams
}

export function Summary({ data }: SummaryProps) {
  const { summary, timestamp } = data
  const buildDate = new Date(timestamp).toLocaleString()

  // Collapsible 状态
  const [chunksExpanded, setChunksExpanded] = useState(false)
  const [assetsExpanded, setAssetsExpanded] = useState(false)
  const [modulesExpanded, setModulesExpanded] = useState(false)

  // 分析 Chunks - 互斥分类，避免重复计算
  const chunkStats = useMemo(() => {
    // 互斥分类：按优先级分配到唯一类别
    const entryChunks = data.chunks.filter((c) => c.type === 'entry')
    const runtimeChunks = data.chunks.filter((c) => c.type === 'runtime')
    const initialChunks = data.chunks.filter((c) => c.type === 'initial')
    const asyncChunks = data.chunks.filter((c) => c.type === 'async')

    const entrySize = entryChunks.reduce((sum, c) => sum + c.size, 0)
    const runtimeSize = runtimeChunks.reduce((sum, c) => sum + c.size, 0)
    const initialSize = initialChunks.reduce((sum, c) => sum + c.size, 0)
    const asyncSize = asyncChunks.reduce((sum, c) => sum + c.size, 0)

    // 使用实际分类的 chunks 总和，而不是 summary.totalSize
    const totalSize = entrySize + runtimeSize + initialSize + asyncSize

    return {
      entry: entryChunks.length,
      initial: initialChunks.length,
      runtime: runtimeChunks.length,
      async: asyncChunks.length,
      entrySize,
      initialSize,
      runtimeSize,
      asyncSize,
      entryPercent: safePercentage(entrySize, totalSize),
      initialPercent: safePercentage(initialSize, totalSize),
      runtimePercent: safePercentage(runtimeSize, totalSize),
      asyncPercent: safePercentage(asyncSize, totalSize),
    }
  }, [data.chunks])

  const { topChunks, topAssets, topModules, assetTypeStats, packageStats, codeDistribution } = useMemo(() => {
    const topChunks = data.chunks.toSorted((a, b) => b.size - a.size).slice(0, 10)
    const topAssets = data.assets.toSorted((a, b) => b.size - a.size).slice(0, 10)
    const topModules = data.modules.toSorted((a, b) => b.size - a.size).slice(0, 10)

    const stats = new Map<ReturnType<typeof getAssetType>, { count: number; size: number }>()
    for (const asset of data.assets) {
      const type = getAssetType(asset.name)
      const existing = stats.get(type) || { count: 0, size: 0 }
      stats.set(type, {
        count: existing.count + 1,
        size: existing.size + asset.size,
      })
    }
    const sorted = Array.from(stats.entries()).toSorted((a, b) => b[1].size - a[1].size)
    const totalSize = sorted.reduce((sum, [, s]) => sum + s.size, 0)
    const assetTypeStats = sorted.map(([type, s]) => ({
      type,
      count: s.count,
      size: s.size,
      percent: safePercentage(s.size, totalSize),
    }))

    const grouped = new Map<string, Set<string>>()
    for (const pkg of data.packages) {
      if (!grouped.has(pkg.name)) {
        grouped.set(pkg.name, new Set())
      }
      grouped.get(pkg.name)!.add(pkg.version)
    }
    const multiVersion = Array.from(grouped.entries()).filter(([, versions]) => versions.size > 1)
    const packageStats = {
      total: grouped.size,
      multiVersion: multiVersion.length,
      totalPackages: data.packages.length,
    }

    const codeDistribution = analyzeCodeDistribution(data)

    return { topChunks, topAssets, topModules, assetTypeStats, packageStats, codeDistribution }
  }, [data])

  const moduleTotalSize = codeDistribution.thirdPartySize + codeDistribution.projectSize
  const moduleThirdPartyPercent = safePercentage(codeDistribution.thirdPartySize, moduleTotalSize)
  const moduleProjectPercent = safePercentage(codeDistribution.projectSize, moduleTotalSize)

  // 构建耗时统计
  const timingStats = useMemo(() => {
    const { timings } = summary
    const stages = [
      { name: '收集资源', time: timings.collectAssetsMs },
      { name: '收集模块', time: timings.collectModulesMs },
      { name: '收集代码块', time: timings.collectChunksMs },
      { name: '分析依赖包', time: timings.analyzePackagesMs },
    ]
    const totalTime = timings.totalMs
    return stages.map((stage) => ({
      name: stage.name,
      time: stage.time,
      percent: safePercentage(stage.time, totalTime),
    }))
  }, [summary])

  return (
    <div className="space-y-6 animate-fade-in">
      {/* 基础统计 */}
      <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-4">
        <Card className="relative overflow-hidden">
          <div className="absolute top-0 right-0 w-24 h-24 bg-primary/5 blur-3xl rounded-full transform translate-x-8 -translate-y-8" />
          <CardHeader className="pb-3">
            <CardTitle className="text-sm font-medium text-muted-foreground">总体积</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              <div className="flex items-baseline gap-2">
                <span className="text-3xl font-bold">{formatBytes(summary.totalSize)}</span>
                {summary.totalSize > 5 * 1024 * 1024 && (
                  <Badge variant="destructive" className="text-xs">
                    偏大
                  </Badge>
                )}
                {summary.totalSize <= 1 * 1024 * 1024 && (
                  <Badge variant="default" className="text-xs">
                    优秀
                  </Badge>
                )}
              </div>
              {summary.totalGzipSize > 0 && (
                <div className="space-y-1">
                  <div className="flex items-center justify-between text-xs">
                    <span className="text-muted-foreground">Gzip 压缩后</span>
                    <span className="font-medium text-green-600 dark:text-green-400">
                      {formatBytes(summary.totalGzipSize)}
                    </span>
                  </div>
                  <div className="flex items-center gap-2">
                    <div className="h-1.5 flex-1 rounded-full bg-muted overflow-hidden">
                      <div
                        className="h-full bg-primary transition-all"
                        style={{
                          width: `${safePercentage(summary.totalGzipSize, summary.totalSize)}%`,
                          opacity: 0.8,
                        }}
                      />
                    </div>
                    <span className="text-xs text-muted-foreground">
                      {safeToFixed(safePercentage(summary.totalGzipSize, summary.totalSize), 0)}%
                    </span>
                  </div>
                </div>
              )}
            </div>
          </CardContent>
        </Card>

        <Card className="relative overflow-hidden">
          <div className="absolute top-0 right-0 w-24 h-24 bg-primary/5 blur-3xl rounded-full transform translate-x-8 -translate-y-8" />
          <CardHeader className="pb-3">
            <CardTitle className="text-sm font-medium text-muted-foreground">构建时间</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              <div className="flex items-baseline gap-2">
                <span className="text-3xl font-bold">{formatTime(summary.buildTime)}</span>
                {summary.buildTime < 5000 && (
                  <Badge variant="default" className="text-xs">
                    快速
                  </Badge>
                )}
                {summary.buildTime > 30000 && (
                  <Badge variant="secondary" className="text-xs">
                    较慢
                  </Badge>
                )}
              </div>
              <p className="text-xs text-muted-foreground">{buildDate}</p>
            </div>
          </CardContent>
        </Card>

        <Card
          className="relative overflow-hidden cursor-pointer hover:bg-muted/50 transition-colors"
          onClick={() => navigateTo('modules')}
          title="点击查看模块详情"
        >
          <div className="absolute top-0 right-0 w-24 h-24 bg-primary/5 blur-3xl rounded-full transform translate-x-8 -translate-y-8" />
          <CardHeader className="pb-3">
            <CardTitle className="text-sm font-medium text-muted-foreground">模块总数</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              <div className="text-3xl font-bold">{formatNumber(summary.totalModules)}</div>
              <div className="flex items-center gap-2 text-xs">
                <Badge
                  variant="outline"
                  className="text-xs cursor-pointer hover:bg-muted"
                  onClick={(e) => {
                    e.stopPropagation()
                    navigateTo('chunks')
                  }}
                  title="点击查看代码块详情"
                >
                  {summary.totalChunks} 个代码块
                </Badge>
                {summary.totalChunks > 0 && (
                  <span className="text-muted-foreground">
                    {safeToFixed(summary.totalModules / summary.totalChunks, 0)} 模块/块
                  </span>
                )}
              </div>
            </div>
          </CardContent>
        </Card>

        <Card className="relative overflow-hidden">
          <div className="absolute top-0 right-0 w-24 h-24 bg-primary/5 blur-3xl rounded-full transform translate-x-8 -translate-y-8" />
          <CardHeader className="pb-3">
            <CardTitle className="text-sm font-medium text-muted-foreground">资源文件</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              <div className="text-3xl font-bold">{formatNumber(summary.totalAssets)}</div>
              <p className="text-xs text-muted-foreground">{assetTypeStats.length} 种类型</p>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Chunks & Packages 概览 */}
      <div className="grid gap-6 md:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>代码块概览</CardTitle>
            <CardDescription>代码块类型和体积分布</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <div className="flex items-center justify-between text-sm">
                <span className="font-medium">类型分布</span>
                <span className="text-xs text-muted-foreground">{summary.totalChunks} 个代码块</span>
              </div>

              {/* 合并的进度条 */}
              <PercentageBar
                items={[
                  {
                    label: `入口 (${chunkStats.entry} 个)`,
                    value: `${formatBytes(chunkStats.entrySize)} (${chunkStats.entryPercent.toFixed(1)}%)`,
                    percent: chunkStats.entryPercent,
                    variant: 'viz-1' as const,
                  },
                  {
                    label: `初始 (${chunkStats.initial} 个)`,
                    value: `${formatBytes(chunkStats.initialSize)} (${chunkStats.initialPercent.toFixed(1)}%)`,
                    percent: chunkStats.initialPercent,
                    variant: 'viz-2' as const,
                  },
                  {
                    label: `异步 (${chunkStats.async} 个)`,
                    value: `${formatBytes(chunkStats.asyncSize)} (${chunkStats.asyncPercent.toFixed(1)}%)`,
                    percent: chunkStats.asyncPercent,
                    variant: 'viz-3' as const,
                  },
                  {
                    label: `运行时 (${chunkStats.runtime} 个)`,
                    value: `${formatBytes(chunkStats.runtimeSize)} (${chunkStats.runtimePercent.toFixed(1)}%)`,
                    percent: chunkStats.runtimePercent,
                    variant: 'viz-4' as const,
                  },
                ]}
                height="md"
                showLegend={true}
                legendPosition="bottom"
                legendLayout="grid"
                legendColumns={2}
              />
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>依赖包概览</CardTitle>
            <CardDescription>npm 包统计信息</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-1 p-3 rounded-lg bg-muted/30">
                <p className="text-xs text-muted-foreground">独立包数量</p>
                <p className="text-2xl font-bold">{formatNumber(packageStats.total)}</p>
              </div>
              <div className="space-y-1 p-3 rounded-lg bg-muted/30">
                <p className="text-xs text-muted-foreground">包实例总数</p>
                <p className="text-2xl font-bold">{formatNumber(packageStats.totalPackages)}</p>
              </div>
            </div>

            <div className="space-y-2 pt-2 border-t">
              <div className="flex items-center justify-between">
                <span className="text-sm font-medium">多版本依赖</span>
                {packageStats.multiVersion > 0 ? (
                  <Badge variant="destructive" className="text-sm">
                    {packageStats.multiVersion} 个
                  </Badge>
                ) : (
                  <Badge variant="default" className="text-sm">
                    ✓ 无重复
                  </Badge>
                )}
              </div>
              {packageStats.multiVersion > 0 && (
                <p className="text-xs text-muted-foreground">
                  ⚠️ 发现 {packageStats.multiVersion} 个包安装了多个版本，建议统一版本以减小体积
                </p>
              )}
            </div>
          </CardContent>
        </Card>
      </div>

      {/* 代码分布 */}
      <Card>
        <CardHeader>
          <CardTitle>代码分布</CardTitle>
          <CardDescription>第三方依赖包与项目源码占比</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <div className="flex items-center justify-between text-sm">
              <span className="font-medium">整体分布</span>
              <span className="text-xs text-muted-foreground">{formatBytes(moduleTotalSize)} 总计</span>
            </div>
            <PercentageBar
              items={[
                {
                  label: `第三方依赖 (${formatNumber(codeDistribution.thirdPartyModules)} 个模块)`,
                  value: `${formatBytes(codeDistribution.thirdPartySize)} (${moduleThirdPartyPercent.toFixed(1)}%)`,
                  percent: moduleThirdPartyPercent,
                  variant: 'viz-2',
                },
                {
                  label: `项目源码 (${formatNumber(codeDistribution.projectModules)} 个模块)`,
                  value: `${formatBytes(codeDistribution.projectSize)} (${moduleProjectPercent.toFixed(1)}%)`,
                  percent: moduleProjectPercent,
                  variant: 'viz-1',
                },
              ]}
              height="md"
              showLegend={true}
              legendPosition="bottom"
              legendLayout="grid"
              legendColumns={2}
            />
          </div>
        </CardContent>
      </Card>

      {/* Asset Types */}
      <Card>
        <CardHeader>
          <CardTitle>资源类型</CardTitle>
          <CardDescription>资源文件类型分布</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <div className="flex items-center justify-between text-sm">
              <span className="font-medium">类型分布</span>
              <span className="text-xs text-muted-foreground">{summary.totalAssets} 个文件</span>
            </div>

            <PercentageBar
              items={assetTypeStats.map((stat, index) => {
                // 为每个资源类型明确分配颜色变体
                const variants = [
                  'viz-1',
                  'viz-2',
                  'viz-3',
                  'viz-4',
                  'viz-5',
                  'viz-6',
                  'viz-7',
                  'viz-8',
                  'viz-9',
                  'viz-10',
                ] as const
                return {
                  label: `${getAssetTypeName(stat.type)} (${stat.count} 个)`,
                  value: `${formatBytes(stat.size)} (${stat.percent.toFixed(1)}%)`,
                  percent: stat.percent,
                  variant: variants[index % variants.length],
                }
              })}
              height="md"
              colorMode="distinct"
              showLegend={true}
              legendPosition="bottom"
              legendLayout="grid"
            />
          </div>
        </CardContent>
      </Card>

      {/* Build Timings */}
      <Card>
        <CardHeader>
          <CardTitle>构建耗时</CardTitle>
          <CardDescription>构建过程各阶段耗时详情</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <div className="flex items-center justify-between text-sm">
              <span className="font-medium">阶段分布</span>
              <span className="text-xs text-muted-foreground">总计 {formatTime(summary.timings.totalMs)}</span>
            </div>

            <PercentageBar
              items={timingStats.map((stat, index) => {
                // 为每个构建阶段明确分配颜色变体
                const variants = [
                  'viz-1',
                  'viz-2',
                  'viz-3',
                  'viz-4',
                  'viz-5',
                  'viz-6',
                  'viz-7',
                  'viz-8',
                  'viz-9',
                  'viz-10',
                ] as const
                return {
                  label: stat.name,
                  value: `${formatTime(stat.time)} (${stat.percent.toFixed(1)}%)`,
                  percent: stat.percent,
                  variant: variants[index % variants.length],
                }
              })}
              height="md"
              colorMode="distinct"
              showLegend={true}
              legendPosition="bottom"
              legendLayout="grid"
              legendColumns={4}
            />
          </div>
        </CardContent>
      </Card>

      {/* Top Chunks */}
      <Collapsible open={chunksExpanded} onOpenChange={setChunksExpanded}>
        <Card className="py-0!">
          <CollapsibleTrigger asChild>
            <CardHeader className="cursor-pointer py-3 hover:bg-muted/50 transition-colors grid-cols-1! grid-rows-1!">
              <div className="flex items-center justify-between w-full h-full">
                <div className="text-base font-medium">最大的 10 个代码块</div>
                <div className="flex items-center gap-2">
                  <Badge variant="outline">{topChunks.length} 个代码块</Badge>
                  <span className="text-muted-foreground text-sm">{chunksExpanded ? '▲' : '▼'}</span>
                </div>
              </div>
            </CardHeader>
          </CollapsibleTrigger>
          <CollapsibleContent>
            <CardContent className="pt-0">
              <Table maxHeight="h-108">
                <TableHeader>
                  <TableRow>
                    <TableHead>名称</TableHead>
                    <TableHead>体积</TableHead>
                    <TableHead>模块</TableHead>
                    <TableHead>类型</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {topChunks.map((chunk) => (
                    <TableRow
                      key={chunk.id}
                      className="cursor-pointer hover:bg-muted/50 transition-colors"
                      onClick={() => navigateTo('chunks', { chunkId: chunk.id })}
                      title="点击查看代码块详情"
                    >
                      <TableCell className="font-mono text-xs">{chunk.names.join(', ') || chunk.id}</TableCell>
                      <TableCell>
                        <Badge variant={getSizeBadgeVariant(chunk.size)} className={BADGE_STYLES.SIZE}>
                          {formatBytes(chunk.size)}
                        </Badge>
                      </TableCell>
                      <TableCell>
                        <Badge variant="outline" className="text-xs">
                          {chunk.moduleIds.length} 个模块
                        </Badge>
                      </TableCell>
                      <TableCell>
                        <div className="flex gap-1 flex-wrap">
                          {chunk.type === 'entry' && (
                            <Badge variant="default" className="text-xs">
                              入口
                            </Badge>
                          )}
                          {chunk.type === 'initial' && (
                            <Badge variant="default" className="text-xs">
                              初始
                            </Badge>
                          )}
                          {chunk.type === 'runtime' && (
                            <Badge variant="outline" className="text-xs">
                              运行时
                            </Badge>
                          )}
                          {chunk.type === 'async' && (
                            <Badge variant="outline" className="text-xs">
                              异步
                            </Badge>
                          )}
                        </div>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </CardContent>
          </CollapsibleContent>
        </Card>
      </Collapsible>

      {/* Top Assets */}
      <Collapsible open={assetsExpanded} onOpenChange={setAssetsExpanded}>
        <Card className="py-0!">
          <CollapsibleTrigger asChild>
            <CardHeader className="cursor-pointer py-3 hover:bg-muted/50 transition-colors grid-cols-1! grid-rows-1!">
              <div className="flex items-center justify-between w-full h-full">
                <div className="text-base font-medium">最大的 10 个资源文件</div>
                <div className="flex items-center gap-2">
                  <Badge variant="outline">{topAssets.length} 个资源</Badge>
                  <span className="text-muted-foreground text-sm">{assetsExpanded ? '▲' : '▼'}</span>
                </div>
              </div>
            </CardHeader>
          </CollapsibleTrigger>
          <CollapsibleContent>
            <CardContent className="pt-0">
              <Table maxHeight="h-108">
                <TableHeader>
                  <TableRow>
                    <TableHead>名称</TableHead>
                    <TableHead>体积</TableHead>
                    <TableHead>类型</TableHead>
                    <TableHead>代码块</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {topAssets.map((asset) => (
                    <TableRow key={asset.name}>
                      <TableCell className="font-mono text-xs max-w-md truncate" title={asset.name}>
                        {asset.name}
                      </TableCell>
                      <TableCell>
                        <Badge variant={getSizeBadgeVariant(asset.size)} className={BADGE_STYLES.SIZE}>
                          {formatBytes(asset.size)}
                        </Badge>
                      </TableCell>
                      <TableCell>
                        <Badge variant="outline" className="text-xs">
                          {getAssetTypeName(getAssetType(asset.name))}
                        </Badge>
                      </TableCell>
                      <TableCell>
                        <Badge variant="outline" className="text-xs">
                          {asset.chunkIds.length} 个块
                        </Badge>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </CardContent>
          </CollapsibleContent>
        </Card>
      </Collapsible>

      {/* Top Modules */}
      <Collapsible open={modulesExpanded} onOpenChange={setModulesExpanded}>
        <Card className="py-0!">
          <CollapsibleTrigger asChild>
            <CardHeader className="cursor-pointer py-3 hover:bg-muted/50 transition-colors grid-cols-1! grid-rows-1!">
              <div className="flex items-center justify-between w-full h-full">
                <div className="text-base font-medium">最大的 10 个模块</div>
                <div className="flex items-center gap-2">
                  <Badge variant="outline">{topModules.length} 个模块</Badge>
                  <span className="text-muted-foreground text-sm">{modulesExpanded ? '▲' : '▼'}</span>
                </div>
              </div>
            </CardHeader>
          </CollapsibleTrigger>
          <CollapsibleContent>
            <CardContent className="pt-0">
              <Table maxHeight="h-108">
                <TableHeader>
                  <TableRow>
                    <TableHead>名称</TableHead>
                    <TableHead>体积</TableHead>
                    <TableHead>类型</TableHead>
                    <TableHead>代码块</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {topModules.map((module) => {
                    const [moduleName, fullPathWithLoader] = getModuleName(module.name, data.modules)
                    const fileExt = getFileExtension(moduleName)
                    return (
                      <TableRow
                        key={module.name}
                        className="cursor-pointer hover:bg-muted/50 transition-colors"
                        onClick={() => navigateTo('modules', { moduleName: module.name })}
                        title="点击查看模块详情"
                      >
                        <TableCell className="font-mono text-xs max-w-md truncate" title={fullPathWithLoader}>
                          {moduleName}
                        </TableCell>
                        <TableCell>
                          <Badge variant={getSizeBadgeVariant(module.size)} className={BADGE_STYLES.SIZE}>
                            {formatBytes(module.size)}
                          </Badge>
                        </TableCell>
                        <TableCell>
                          <div className="flex gap-1 flex-wrap">
                            {fileExt && (
                              <Badge variant="outline" className="text-xs">
                                .{fileExt}
                              </Badge>
                            )}
                            <Badge variant="outline" className="text-xs">
                              {module.type}
                            </Badge>
                            {module.isNodeModule && (
                              <Badge variant="secondary" className="text-xs">
                                pkg
                              </Badge>
                            )}
                          </div>
                        </TableCell>
                        <TableCell>
                          <Badge variant="outline" className="text-xs">
                            {module.chunkIds.length} 个块
                          </Badge>
                        </TableCell>
                      </TableRow>
                    )
                  })}
                </TableBody>
              </Table>
            </CardContent>
          </CollapsibleContent>
        </Card>
      </Collapsible>
    </div>
  )
}
