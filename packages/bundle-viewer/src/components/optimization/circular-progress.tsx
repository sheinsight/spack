/**
 * 圆形进度条组件 - 用于显示优化评分
 */

import { cn } from '@/lib/utils'

interface CircularProgressProps {
  score: number
  size?: number
}

export function CircularProgress({ score, size = 140 }: CircularProgressProps) {
  const radius = (size - 12) / 2
  const circumference = 2 * Math.PI * radius
  const offset = circumference - (score / 100) * circumference

  const getColor = () => {
    if (score >= 80) return 'stroke-green-500' // ✅ 保留绿色（优秀）
    if (score >= 60) return 'stroke-primary' // 使用主题色（良好）
    return 'stroke-red-500' // ✅ 保留红色（需改进）
  }

  const getTextColor = () => {
    if (score >= 80) return 'text-green-600 dark:text-green-400'
    if (score >= 60) return 'text-primary'
    return 'text-red-600 dark:text-red-400'
  }

  return (
    <div className="relative inline-flex items-center justify-center">
      <svg width={size} height={size} className="transform -rotate-90">
        {/* 背景圆环 */}
        <circle
          cx={size / 2}
          cy={size / 2}
          r={radius}
          stroke="currentColor"
          strokeWidth="10"
          fill="none"
          className="text-muted/20"
        />
        {/* 进度圆环 */}
        <circle
          cx={size / 2}
          cy={size / 2}
          r={radius}
          strokeWidth="10"
          fill="none"
          strokeDasharray={circumference}
          strokeDashoffset={offset}
          className={cn(getColor(), 'transition-all duration-1000 ease-out')}
          strokeLinecap="round"
        />
      </svg>
      {/* 中心文字 */}
      <div className="absolute inset-0 flex flex-col items-center justify-center">
        <span className={cn('text-4xl font-bold', getTextColor())}>{score}</span>
        <span className="text-sm text-muted-foreground mt-0.5">/ 100</span>
      </div>
    </div>
  )
}
