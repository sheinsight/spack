import { useState } from 'react'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import { cn } from '@/lib/utils'

export interface PercentageItem {
  label: string
  value: string | number // 显示的值（如 "1.2 MB" 或 "150ms"）
  percent: number // 百分比 0-100
  variant?:
    | 'viz-1'
    | 'viz-2'
    | 'viz-3'
    | 'viz-4'
    | 'viz-5'
    | 'viz-6'
    | 'viz-7'
    | 'viz-8'
    | 'viz-9'
    | 'viz-10'
    | 'success'
    | 'warning'
    | 'danger' // 颜色变体
}

interface PercentageBarProps {
  items: PercentageItem[]
  height?: 'sm' | 'md' | 'lg' // 高度预设
  colorMode?: 'gradient' | 'distinct' | 'semantic' | 'auto' // 颜色模式
  showAnimation?: boolean // 是否显示入场动画（默认 true）
  showLegend?: boolean // 是否显示图例（默认 false）
  legendPosition?: 'top' | 'bottom' | 'left' | 'right' // 图例位置（默认 bottom）
  legendLayout?: 'flex' | 'grid' // 图例布局模式（默认 flex）
  legendColumns?: number // grid 模式下的列数（默认 3）
}

// 辅助函数：获取分段颜色
function getSegmentColor(
  item: PercentageItem,
  index: number,
  colorMode: 'gradient' | 'distinct' | 'semantic' | 'auto',
): string {
  // 1. 优先使用自定义 variant
  if (item.variant) {
    switch (item.variant) {
      case 'viz-1':
        return 'bg-data-viz-1' // #1 钴蓝 - 深邃专业（最常用）
      case 'viz-2':
        return 'bg-data-viz-2' // #2 碧绿 - 清新活力
      case 'viz-3':
        return 'bg-data-viz-3' // #3 蓝紫 - 优雅神秘
      case 'viz-4':
        return 'bg-data-viz-4' // #4 青蓝 - 冷静稳重
      case 'viz-5':
        return 'bg-data-viz-5' // #5 翠绿 - 生机勃勃
      case 'viz-6':
        return 'bg-data-viz-6' // #6 靛蓝 - 深沉高雅
      case 'viz-7':
        return 'bg-data-viz-7' // #7 青绿 - 清透明亮
      case 'viz-8':
        return 'bg-data-viz-8' // #8 紫色 - 创意独特
      case 'viz-9':
        return 'bg-data-viz-9' // #9 浅青 - 轻盈舒适
      case 'viz-10':
        return 'bg-data-viz-10' // #10 宝石蓝 - 精致奢华
      case 'success':
        return 'bg-green-600' // 成功：绿色
      case 'warning':
        return 'bg-amber-500' // 警告：琥珀色
      case 'danger':
        return 'bg-red-600' // 危险：红色
      default:
        return 'bg-data-viz-1'
    }
  }

  // 2. 根据 colorMode 自动分配颜色
  switch (colorMode) {
    case 'gradient': {
      // 渐变模式：选择色相接近的 4 色形成和谐渐变（钴蓝→青蓝→浅青→宝石蓝）
      const colors = ['bg-data-viz-1', 'bg-data-viz-4', 'bg-data-viz-9', 'bg-data-viz-10']
      return colors[index % colors.length]
    }

    case 'distinct': {
      // 区分模式：10 色完整方案，色相跨度大，高对比度
      const colors = [
        'bg-data-viz-1', // #1 钴蓝（200°）
        'bg-data-viz-2', // #2 碧绿（150°）
        'bg-data-viz-3', // #3 蓝紫（270°）
        'bg-data-viz-4', // #4 青蓝（180°）
        'bg-data-viz-5', // #5 翠绿（130°）
        'bg-data-viz-6', // #6 靛蓝（240°）
        'bg-data-viz-7', // #7 青绿（160°）
        'bg-data-viz-8', // #8 紫色（290°）
        'bg-data-viz-9', // #9 浅青（170°）
        'bg-data-viz-10', // #10 宝石蓝（220°）
      ]
      return colors[index % colors.length]
    }

    case 'semantic': {
      // 语义模式：根据占比自动分配颜色（高→低：钴蓝→青蓝→青绿→碧绿）
      if (item.percent > 50) return 'bg-data-viz-1' // 占比高：钴蓝
      if (item.percent > 30) return 'bg-data-viz-4' // 占比中：青蓝
      if (item.percent > 10) return 'bg-data-viz-7' // 占比低：青绿
      return 'bg-data-viz-2' // 占比极低：碧绿
    }

    case 'auto':
    default: {
      // 自动模式：智能选择（默认使用 distinct 的 10 色方案）
      const colors = [
        'bg-data-viz-1',
        'bg-data-viz-2',
        'bg-data-viz-3',
        'bg-data-viz-4',
        'bg-data-viz-5',
        'bg-data-viz-6',
        'bg-data-viz-7',
        'bg-data-viz-8',
        'bg-data-viz-9',
        'bg-data-viz-10',
      ]
      return colors[index % colors.length]
    }
  }
}

export function PercentageBar({
  items,
  height = 'md',
  colorMode = 'distinct',
  showAnimation = true,
  showLegend = false,
  legendPosition = 'bottom',
  legendLayout = 'flex',
  legendColumns = 3,
}: PercentageBarProps) {
  // 状态管理：跟踪每个项目的显示/隐藏状态（默认全部显示）
  const [visibleItems, setVisibleItems] = useState<Set<number>>(new Set(items.map((_, index) => index)))

  const heightClasses = {
    sm: 'h-2',
    md: 'h-4',
    lg: 'h-6',
  }

  // 切换项目的显示/隐藏状态
  const toggleItem = (index: number) => {
    setVisibleItems((prev) => {
      const newSet = new Set(prev)
      if (newSet.has(index)) {
        // 至少保留一个可见项
        if (newSet.size > 1) {
          newSet.delete(index)
        }
      } else {
        newSet.add(index)
      }
      return newSet
    })
  }

  // 过滤出可见的项目（同时保存原始索引）
  const visibleItemsData = items
    .map((item, index) => ({ item, originalIndex: index }))
    .filter(({ originalIndex }) => visibleItems.has(originalIndex))

  // 重新计算可见项目的百分比（归一化到 100%）
  const totalVisiblePercent = visibleItemsData.reduce((sum, { item }) => sum + item.percent, 0)
  const normalizedItems = visibleItemsData.map(({ item, originalIndex }) => ({
    ...item,
    originalIndex,
    normalizedPercent: totalVisiblePercent > 0 ? (item.percent / totalVisiblePercent) * 100 : 0,
  }))

  // 获取图例容器的样式
  const getLegendStyle = () => {
    if (legendLayout === 'grid') {
      return {
        display: 'grid',
        gap: '0.5rem',
        gridTemplateColumns: `repeat(${legendColumns}, minmax(0, 1fr))`,
      }
    }
    return undefined
  }

  const getLegendClassName = () => {
    // grid 模式不需要额外的 className，使用内联样式
    if (legendLayout === 'grid') {
      return ''
    }
    // 默认 flex 布局
    return `flex flex-wrap gap-2 ${
      legendPosition === 'top' || legendPosition === 'bottom' ? 'justify-center' : 'flex-col'
    }`
  }

  // 渲染图例
  const renderLegend = () => (
    <div className={getLegendClassName()} style={getLegendStyle()}>
      {items.map((item, index) => {
        const segmentColor = getSegmentColor(item, index, colorMode)
        const isVisible = visibleItems.has(index)
        const canHide = visibleItems.size > 1 // 至少保留一个可见项

        return (
          <button
            key={`legend-${index}`}
            type="button"
            onClick={() => void toggleItem(index)}
            disabled={!canHide && isVisible}
            className={cn(
              'flex items-center gap-1.5 p-1.5 rounded text-xs',
              'transition-all duration-200',
              isVisible
                ? 'bg-muted hover:bg-muted/80 text-foreground cursor-pointer'
                : 'bg-muted/30 text-muted-foreground hover:bg-muted/50 cursor-pointer',
              !canHide && isVisible && 'cursor-not-allowed opacity-60',
              'focus:outline-none focus:ring-2 focus:ring-primary/50',
            )}
            title={
              !canHide && isVisible
                ? '至少需要保留一个可见项'
                : isVisible
                  ? `点击隐藏 ${item.label}`
                  : `点击显示 ${item.label}`
            }
          >
            {/* 颜色指示器 */}
            <div
              className={cn(
                'w-3 h-3 rounded shrink-0',
                segmentColor,
                'transition-all duration-200',
                isVisible ? 'opacity-100' : 'opacity-40',
              )}
            />
            {/* 内容区域 */}
            <div className="flex-1 text-left min-w-0">
              <div className="flex items-center justify-between gap-1">
                <span className={cn('font-medium text-foreground truncate', !isVisible && 'line-through')}>
                  {item.label}
                </span>
              </div>
              <div className="text-muted-foreground truncate">{item.value}</div>
            </div>
          </button>
        )
      })}
    </div>
  )

  // 布局容器样式
  const getContainerClass = () => {
    if (!showLegend) return ''
    const layoutClasses = {
      top: 'flex flex-col gap-3',
      bottom: 'flex flex-col gap-3',
      left: 'flex flex-row gap-4 items-center',
      right: 'flex flex-row gap-4 items-center',
    }
    return layoutClasses[legendPosition]
  }

  const isAllLegendVisible = items.every((_, index) => visibleItems.has(index))

  return (
    <div className={getContainerClass()}>
      {showLegend && legendPosition === 'top' && renderLegend()}
      {showLegend && legendPosition === 'left' && renderLegend()}

      <TooltipProvider delayDuration={0}>
        <div
          className={cn(
            'rounded-full overflow-hidden relative bg-muted',
            heightClasses[height],
            showLegend && (legendPosition === 'left' || legendPosition === 'right') && 'flex-1',
          )}
          role="progressbar"
          aria-label="数据分布进度条"
        >
          {normalizedItems.map((item, visibleIndex) => {
            // 原始索引已经保存在 item 中
            const { originalIndex } = item
            // 计算左偏移量（基于归一化后的百分比）
            const leftOffset = normalizedItems.slice(0, visibleIndex).reduce((sum, i) => sum + i.normalizedPercent, 0)
            const segmentColor = getSegmentColor(item, originalIndex, colorMode)

            return (
              <Tooltip key={originalIndex}>
                <TooltipTrigger asChild>
                  <div
                    className={cn(
                      'absolute top-0 h-full',
                      segmentColor,
                      showAnimation && 'animate-fade-in',
                      'transition-all duration-300 ease-in-out',
                      'hover:brightness-110',
                      'motion-reduce:transition-none',
                    )}
                    style={{
                      left: `${leftOffset}%`,
                      width: `${item.normalizedPercent}%`,
                      animationDelay: showAnimation ? `${visibleIndex * 50}ms` : '0ms',
                    }}
                    aria-label={`${item.label}: ${item.value} (${item.percent.toFixed(1)}%)`}
                  />
                </TooltipTrigger>
                <TooltipContent side="top" className="max-w-xs">
                  <div className="space-y-1.5">
                    <div className="font-semibold text-sm text-foreground">{item.label}</div>
                    <div className="flex items-center justify-between gap-4 text-xs">
                      <span className="text-muted-foreground">数值</span>
                      <span className="font-medium">{item.value}</span>
                    </div>
                    <div className="flex items-center justify-between gap-4 text-xs">
                      <span className="text-muted-foreground">原始占比</span>
                      <span className="font-medium">{item.percent.toFixed(1)}%</span>
                    </div>
                    {!isAllLegendVisible && (
                      <div className="flex items-center justify-between gap-4 text-xs">
                        <span className="text-muted-foreground">当前占比</span>
                        <span className="font-medium">{item.normalizedPercent.toFixed(1)}%</span>
                      </div>
                    )}
                    {/* 进度条可视化 */}
                    <div className="pt-1">
                      <div className="h-1 rounded-full bg-muted/30 overflow-hidden">
                        <div
                          className={cn('h-full transition-all duration-300', segmentColor)}
                          style={{ width: `${item.normalizedPercent}%` }}
                        />
                      </div>
                    </div>
                  </div>
                </TooltipContent>
              </Tooltip>
            )
          })}
        </div>
      </TooltipProvider>

      {showLegend && legendPosition === 'right' && renderLegend()}
      {showLegend && legendPosition === 'bottom' && renderLegend()}
    </div>
  )
}
