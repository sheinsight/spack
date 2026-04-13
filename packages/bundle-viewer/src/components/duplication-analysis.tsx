import { memo, useMemo, useState } from 'react'
import type { StandardBundleData } from '@/types/bundle-data-standard'
import { navigateTo } from '@/lib/navigation'
import {
  analyzeModuleDuplication,
  type ModuleDuplicationStat,
  type DuplicationAnalysisResult,
} from '@/lib/duplication-analyzer'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
import { PercentageBar } from '@/components/ui/percentage-bar'
import { formatBytes } from '@/lib/utils/format'
import { beautifyModulePath } from '@/lib/utils/path-utils'
import { BADGE_STYLES } from '@/lib/constants'

interface DuplicationAnalysisProps {
  data: StandardBundleData
}

// 统计卡片组件
function StatsCards({ result }: { result: DuplicationAnalysisResult }) {
  const maxChunkCount = Math.max(...result.modules.map((m) => m.chunkCount), 0)

  return (
    <div className="grid gap-4 md:grid-cols-4">
      {/* 重复模块数 */}
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-xs font-medium text-muted-foreground">重复模块数</CardTitle>
        </CardHeader>
        <CardContent className="pb-3">
          <div className="flex items-baseline gap-2">
            <span className="text-3xl font-bold font-mono">{result.duplicatedModules}</span>
            <span className="text-xs text-muted-foreground font-mono">/ {result.totalModules}</span>
          </div>
          <p className="text-xs text-muted-foreground mt-1 font-mono">
            {result.totalModules > 0 ? ((result.duplicatedModules / result.totalModules) * 100).toFixed(1) : 0}%
            存在重复
          </p>
        </CardContent>
      </Card>

      {/* 浪费体积 */}
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-xs font-medium text-muted-foreground">浪费体积</CardTitle>
        </CardHeader>
        <CardContent className="pb-3">
          <div className="text-3xl font-bold text-destructive">{formatBytes(result.totalWastedSize)}</div>
          <p className="text-xs text-muted-foreground mt-1">可通过优化节省</p>
        </CardContent>
      </Card>

      {/* 浪费占比 */}
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-xs font-medium text-muted-foreground">浪费占比</CardTitle>
        </CardHeader>
        <CardContent className="pb-3">
          <div className="text-3xl font-bold font-mono">{result.wastedPercentage.toFixed(1)}%</div>
          <div className="mt-2 h-2 rounded-full bg-muted overflow-hidden">
            <div
              className="h-full bg-primary transition-all"
              style={{
                width: `${Math.min(result.wastedPercentage, 100)}%`,
                opacity: 0.8,
              }}
            />
          </div>
        </CardContent>
      </Card>

      {/* 最大重复次数 */}
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-xs font-medium text-muted-foreground">最大重复次数</CardTitle>
        </CardHeader>
        <CardContent className="pb-3">
          <div className="text-3xl font-bold font-mono">{maxChunkCount} 次</div>
          <p className="text-xs text-muted-foreground mt-1">单个模块最多出现次数</p>
        </CardContent>
      </Card>
    </div>
  )
}

// 重复度分布图组件
function DistributionChart({ distribution }: { distribution: DuplicationAnalysisResult['duplicationDistribution'] }) {
  const totalCount = distribution.count2 + distribution.count3 + distribution.count4 + distribution.count5Plus

  const items = [
    {
      label: '2x',
      value: `${distribution.count2} 个模块`,
      percent: totalCount > 0 ? (distribution.count2 / totalCount) * 100 : 0,
      variant: 'viz-1' as const,
    },
    {
      label: '3x',
      value: `${distribution.count3} 个模块`,
      percent: totalCount > 0 ? (distribution.count3 / totalCount) * 100 : 0,
      variant: 'viz-2' as const,
    },
    {
      label: '4x',
      value: `${distribution.count4} 个模块`,
      percent: totalCount > 0 ? (distribution.count4 / totalCount) * 100 : 0,
      variant: 'viz-3' as const,
    },
    {
      label: '5+',
      value: `${distribution.count5Plus} 个模块`,
      percent: totalCount > 0 ? (distribution.count5Plus / totalCount) * 100 : 0,
      variant: 'viz-4' as const,
    },
  ].filter((item) => item.percent > 0) // 只显示有数据的项

  return (
    <Card>
      <CardHeader>
        <CardTitle>重复度分布</CardTitle>
        <CardDescription>共 {totalCount} 个模块存在重复引用（点击图例可切换显示）</CardDescription>
      </CardHeader>
      <CardContent>
        <PercentageBar
          items={items}
          height="md"
          colorMode="distinct"
          showAnimation={true}
          showLegend={true}
          legendPosition="bottom"
          legendLayout="grid"
          legendColumns={4}
        />
      </CardContent>
    </Card>
  )
}

// 模块列表项组件
interface ModuleItemProps {
  stat: ModuleDuplicationStat
  isExpanded: boolean
  onToggle: () => void
}

const getBadgeVariant = (count: number): 'destructive' | 'default' | 'secondary' | 'outline' => {
  if (count >= 5) return 'destructive'
  if (count >= 3) return 'default'
  if (count === 2) return 'secondary'
  return 'outline'
}

const ModuleDuplicationItem = memo(({ stat, isExpanded, onToggle }: ModuleItemProps) => {
  return (
    <Collapsible open={isExpanded} onOpenChange={onToggle}>
      <div className="rounded-lg border hover:bg-muted/50 transition-colors">
        {/* 头部 - 分离展开按钮和内容 */}
        <CollapsibleTrigger asChild className="hover:bg-muted/50 cursor-pointer">
          <div className="p-3 flex items-start gap-2">
            {/* 模块内容区域 */}
            <div className="min-w-0">
              {/* 模块名 + 徽章 */}
              <div className="flex items-start justify-between gap-2">
                <div
                  onClick={() => navigateTo('modules', { moduleName: stat.moduleName })}
                  className="flex-1 text-left font-mono text-sm truncate cursor-pointer hover:text-primary transition-colors"
                  title={`${stat.moduleName}\n点击查看模块详情`}
                >
                  {beautifyModulePath(stat.moduleName)}
                </div>
                <div className="flex gap-1.5 shrink-0">
                  {stat.moduleType === 'third-party' && (
                    <Badge variant="secondary" className={BADGE_STYLES.COMPACT}>
                      pkg
                    </Badge>
                  )}
                  <Badge variant={getBadgeVariant(stat.chunkCount)} className={BADGE_STYLES.NUMERIC}>
                    {stat.chunkCount}x
                  </Badge>
                </div>
              </div>

              {/* 指标行 */}
              <div className="flex items-center gap-4 text-xs text-muted-foreground mt-2 font-mono">
                <span title="影响评分">评分 {stat.impactScore.toFixed(1)}</span>
                <span title="浪费体积" className="text-destructive font-medium">
                  浪费 {formatBytes(stat.wastedSize)}
                </span>
                <span title="模块体积">体积 {formatBytes(stat.moduleSize)}</span>
              </div>
            </div>
          </div>
        </CollapsibleTrigger>

        {/* 展开内容 */}
        <CollapsibleContent className="mt-3 ml-7 space-y-3">
          {/* Chunk列表 */}
          <div>
            <p className="text-xs text-muted-foreground mb-2">出现在以下 {stat.chunkCount} 个代码块：</p>
            <div className="space-y-1">
              {stat.chunks.map((chunk) => (
                <div
                  key={chunk.id}
                  className="w-full flex items-center gap-2 text-xs bg-muted/30 px-2 py-1.5 rounded cursor-pointer hover:bg-muted/50 transition-colors"
                  onClick={() => navigateTo('chunks', { chunkId: chunk.id, moduleName: stat.moduleName })}
                  title="点击查看代码块详情"
                >
                  <span className="flex-1 font-mono truncate text-left" title={chunk.names.join(', ')}>
                    {chunk.names.join(', ') || `#${chunk.id}`}
                  </span>
                  {chunk.type === 'entry' && (
                    <Badge variant="default" className={BADGE_STYLES.COMPACT}>
                      入口
                    </Badge>
                  )}
                  {(chunk.type === 'initial' || chunk.type === 'entry') && (
                    <Badge variant="outline" className={BADGE_STYLES.COMPACT}>
                      初始
                    </Badge>
                  )}
                  <span className="text-muted-foreground shrink-0 font-mono">{formatBytes(chunk.size)}</span>
                </div>
              ))}
            </div>
          </div>

          {/* concatenatedModules */}
          {stat.concatenatedModules && stat.concatenatedModules.length > 0 && (
            <div className="pt-2 border-t">
              <p className="text-xs text-muted-foreground mb-1">包含 {stat.concatenatedModules.length} 个合并模块：</p>
              <div className="space-y-0.5">
                {stat.concatenatedModules.slice(0, 5).map((cm) => (
                  <div key={cm.id} className="text-xs font-mono text-muted-foreground pl-2">
                    • {beautifyModulePath(cm.name)} ({formatBytes(cm.size)})
                  </div>
                ))}
                {stat.concatenatedModules.length > 5 && (
                  <p className="text-xs text-muted-foreground pl-2">
                    ... 还有 {stat.concatenatedModules.length - 5} 个
                  </p>
                )}
              </div>
            </div>
          )}
        </CollapsibleContent>
      </div>
    </Collapsible>
  )
})

ModuleDuplicationItem.displayName = 'ModuleDuplicationItem'

// 主组件
export function DuplicationAnalysis({ data }: DuplicationAnalysisProps) {
  // 1. 计算分析结果
  const result = useMemo(() => analyzeModuleDuplication(data), [data])

  // 2. UI状态
  const [searchQuery, setSearchQuery] = useState('')
  const [filterType, setFilterType] = useState<'all' | 'thirdParty' | 'project'>('all')
  const [sortBy, setSortBy] = useState<'impact' | 'count' | 'size'>('impact')
  const [expandedModule, setExpandedModule] = useState<string | null>(null)
  const [displayLimit, setDisplayLimit] = useState(20)

  const duplicatedTotal = useMemo(() => result.modules.filter((m) => m.chunkCount > 1).length, [result.modules])
  const duplicatedThirdPartyTotal = useMemo(
    () => result.modules.filter((m) => m.chunkCount > 1 && m.moduleType === 'third-party').length,
    [result.modules],
  )
  const duplicatedProjectTotal = useMemo(
    () => result.modules.filter((m) => m.chunkCount > 1 && m.moduleType !== 'third-party').length,
    [result.modules],
  )

  // 3. 过滤和排序
  const filteredModules = useMemo(() => {
    let filtered = result.modules.filter((m) => m.chunkCount > 1) // 只显示重复的

    // 搜索过滤
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase()
      filtered = filtered.filter(
        (m) =>
          m.moduleName.toLowerCase().includes(query) ||
          m.concatenatedModules?.some((cm) => cm.name.toLowerCase().includes(query)),
      )
    }

    // 类型过滤
    if (filterType === 'thirdParty') {
      filtered = filtered.filter((m) => m.moduleType === 'third-party')
    } else if (filterType === 'project') {
      filtered = filtered.filter((m) => m.moduleType !== 'third-party')
    }

    // 排序
    if (sortBy === 'count') {
      filtered = filtered.toSorted((a, b) => b.chunkCount - a.chunkCount)
    } else if (sortBy === 'size') {
      filtered = filtered.toSorted((a, b) => b.wastedSize - a.wastedSize)
    } // 默认按 impactScore 排序（已排好）

    return filtered
  }, [result.modules, searchQuery, filterType, sortBy])

  const displayedModules = filteredModules.slice(0, displayLimit)
  const hasMore = filteredModules.length > displayLimit

  return (
    <div className="space-y-6 animate-fade-in">
      {/* 1. 统计卡片 */}
      <StatsCards result={result} />

      {/* 2. 重复度分布图 */}
      <DistributionChart distribution={result.duplicationDistribution} />

      {/* 3. 搜索和过滤 */}
      <div className="flex items-center gap-4 flex-wrap">
        <Input
          type="search"
          placeholder="搜索模块..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="max-w-md"
        />

        <div className="flex items-center gap-3 flex-1 flex-wrap justify-between">
          <div className="flex items-center gap-2 flex-wrap">
            <button
              onClick={() => setFilterType('all')}
              className={`px-3 py-1 text-sm rounded-md transition-colors ${
                filterType === 'all' ? 'bg-primary text-primary-foreground' : 'bg-muted hover:bg-muted/80'
              }`}
            >
              全部 ({duplicatedTotal})
            </button>
            <button
              onClick={() => setFilterType('thirdParty')}
              className={`px-3 py-1 text-sm rounded-md transition-colors ${
                filterType === 'thirdParty' ? 'bg-primary text-primary-foreground' : 'bg-muted hover:bg-muted/80'
              }`}
            >
              仅第三方 ({duplicatedThirdPartyTotal})
            </button>
            <button
              onClick={() => setFilterType('project')}
              className={`px-3 py-1 text-sm rounded-md transition-colors ${
                filterType === 'project' ? 'bg-primary text-primary-foreground' : 'bg-muted hover:bg-muted/80'
              }`}
            >
              仅项目代码 ({duplicatedProjectTotal})
            </button>
          </div>

          <div className="flex items-center gap-2">
            <select
              value={sortBy}
              onChange={(e) => setSortBy(e.target.value as typeof sortBy)}
              className="px-3 py-1 text-sm rounded-md bg-muted border-0"
            >
              <option value="impact">按影响评分排序</option>
              <option value="count">按重复次数排序</option>
              <option value="size">按浪费体积排序</option>
            </select>
          </div>
        </div>
      </div>

      {/* 4. 模块列表 */}
      <Card>
        <CardHeader className="pb-3">
          <CardTitle>重复模块列表</CardTitle>
          <CardDescription>
            按{sortBy === 'impact' ? '影响评分' : sortBy === 'count' ? '重复次数' : '浪费体积'}排序
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-2">
            {displayedModules.map((stat) => (
              <ModuleDuplicationItem
                key={stat.moduleName}
                stat={stat}
                isExpanded={expandedModule === stat.moduleName}
                onToggle={() => setExpandedModule(expandedModule === stat.moduleName ? null : stat.moduleName)}
              />
            ))}
          </div>

          {hasMore && (
            <div className="mt-4 text-center">
              <button
                onClick={() => setDisplayLimit((prev) => prev + 20)}
                className="px-4 py-2 text-sm bg-primary text-primary-foreground rounded-md hover:bg-primary/90 transition-colors"
              >
                加载更多 (还有 {filteredModules.length - displayLimit} 个)
              </button>
            </div>
          )}

          {displayedModules.length === 0 && (
            <div className="py-12 text-center">
              <p className="text-muted-foreground">
                {searchQuery ? `未找到匹配 "${searchQuery}" 的模块` : '无重复模块'}
              </p>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
