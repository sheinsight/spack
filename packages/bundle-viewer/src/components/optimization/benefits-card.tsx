/**
 * 收益量化卡片 - 按网络环境估算性能改善
 */

import { useState } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { cn } from '@/lib/utils'

// ─── 网络配置 ────────────────────────────────────────────────────────────────
// 带宽参考 Chrome DevTools 网络节流标准（bytes/ms）
// JS 解析速度参考 Addy Osmani《The Cost of JavaScript》移动端中档机基准：~1ms/KB

const NETWORK_OPTIONS = [
  { key: '2g', label: '2G', bandwidth: (250 * 1024) / 8 / 1000 }, // 250 Kbps → bytes/ms ≈ 31
  { key: 'slow3g', label: '慢速 3G', bandwidth: (400 * 1024) / 8 / 1000 }, // 400 Kbps → ≈ 50
  { key: 'fast3g', label: '快速 3G', bandwidth: (1500 * 1024) / 8 / 1000 }, // 1.5 Mbps → ≈ 187
  { key: '4g', label: '4G', bandwidth: (9 * 1024 * 1024) / 8 / 1000 }, // 9 Mbps → ≈ 1125
  { key: '5g', label: '5G', bandwidth: (100 * 1024 * 1024) / 8 / 1000 }, // 100 Mbps → ≈ 12500
] as const

type NetworkKey = (typeof NETWORK_OPTIONS)[number]['key']

// JS 解析速度：移动端中档机约 1ms/KB（V8 基准数据）
const JS_PARSE_MS_PER_BYTE = 1 / 1024

/**
 * 计算各指标改善量
 *
 * FCP：首次内容绘制，主要受 Entry chunk 下载时间影响（阻塞渲染）
 * LCP：最大内容绘制，受下载 + 部分 JS 渲染路径影响，系数 0.7/0.3 加权
 * TTI：可交互时间，需等待下载完成 + JS 解析 + 执行（解析时间 × 1.5 估算执行开销）
 */
function calcMetrics(sizeSaving: number, bandwidthBytesPerMs: number) {
  const downloadMs = sizeSaving / bandwidthBytesPerMs
  const parseMs = sizeSaving * JS_PARSE_MS_PER_BYTE
  const execMs = parseMs * 1.5

  return {
    fcp: Math.round(downloadMs),
    lcp: Math.round(downloadMs * 0.7 + parseMs * 0.3),
    tti: Math.round(downloadMs + parseMs + execMs),
  }
}

// ─── 组件 ────────────────────────────────────────────────────────────────────

interface BenefitsCardProps {
  sizeSaving: number
}

export function BenefitsCard({ sizeSaving }: BenefitsCardProps) {
  const [network, setNetwork] = useState<NetworkKey>('fast3g')

  if (sizeSaving <= 0) return null

  const profile = NETWORK_OPTIONS.find((o) => o.key === network)!
  const { fcp, lcp, tti } = calcMetrics(sizeSaving, profile.bandwidth)

  const metrics = [
    {
      value: fcp,
      unit: 'ms',
      label: 'FCP 预期改善',
      sub: '首次内容绘制加速',
      tooltip:
        '首次内容绘制（First Contentful Paint）：浏览器渲染第一个 DOM 内容的时刻。Entry chunk 体积减少可直接缩短阻塞时间。',
    },
    {
      value: lcp,
      unit: 'ms',
      label: 'LCP 预期改善',
      sub: '最大内容绘制加速',
      tooltip:
        '最大内容绘制（Largest Contentful Paint）：页面主体内容完成渲染的时刻。受下载时间（权重 70%）和 JS 渲染路径（权重 30%）共同影响。',
    },
    {
      value: tti,
      unit: 'ms',
      label: 'TTI 预期改善',
      sub: '可交互时间加速',
      tooltip:
        '可交互时间（Time to Interactive）：页面可稳定响应用户操作的时刻。= 下载时间 + JS 解析时间 + JS 执行时间（估算为解析的 1.5 倍）。',
    },
  ]

  return (
    <Card>
      <CardHeader>
        <div className="flex items-start justify-between gap-4">
          <div>
            <CardTitle>预期收益</CardTitle>
            <CardDescription>按网络环境估算性能改善</CardDescription>
          </div>

          {/* 网络环境选择器 */}
          <div className="flex gap-0.5 p-1 bg-muted rounded-lg shrink-0">
            {NETWORK_OPTIONS.map((option) => (
              <button
                key={option.key}
                onClick={() => setNetwork(option.key)}
                className={cn(
                  'px-2.5 py-1 text-xs rounded-md transition-colors whitespace-nowrap',
                  network === option.key
                    ? 'bg-background text-foreground shadow-sm font-medium'
                    : 'text-muted-foreground hover:text-foreground',
                )}
              >
                {option.label}
              </button>
            ))}
          </div>
        </div>
      </CardHeader>

      <CardContent>
        <div className="grid md:grid-cols-3 gap-4">
          {metrics.map((m) => (
            <div key={m.label} className="text-center p-4 rounded-lg bg-primary/10">
              <div className="text-3xl font-bold text-foreground flex items-baseline justify-center gap-1">
                <span className="text-xl text-muted-foreground">~</span>
                {m.value.toLocaleString()}
                <span className="text-base font-normal text-muted-foreground">{m.unit}</span>
              </div>
              <p className="text-sm text-muted-foreground mt-1 flex items-center justify-center gap-1">
                {m.label}
                <span className="text-xs cursor-help" title={m.tooltip}>
                  ⓘ
                </span>
              </p>
              <p className="text-xs text-muted-foreground mt-1">{m.sub}</p>
            </div>
          ))}
        </div>

        {/* 算法说明 */}
        <div className="mt-4 text-xs text-muted-foreground p-3 bg-muted/50 rounded space-y-1">
          <p className="font-medium text-foreground/70">计算依据</p>
          <p>• 带宽参考 Chrome DevTools 标准网络节流配置；FCP 以 Entry chunk 下载时间为主要变量</p>
          <p>• JS 解析速度基于 V8 引擎移动端中档机基准（~1 ms/KB），执行时间估算为解析的 1.5 倍</p>
          <p>
            • 以上为 <strong>增量估算</strong>，表示优化前后的差值，非绝对加载时长；实际结果受设备性能、缓存策略、CDN
            等因素影响
          </p>
        </div>
      </CardContent>
    </Card>
  )
}
