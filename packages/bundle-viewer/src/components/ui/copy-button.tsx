import { useState, useCallback } from 'react'
import { Button } from './button'
import type { VariantProps } from 'class-variance-authority'
import { buttonVariants } from './button'

interface CopyButtonProps extends VariantProps<typeof buttonVariants> {
  /** 要复制的文本内容 */
  text: string
  /** 自定义类名 */
  className?: string
  /** 复制前的文本 */
  children?: React.ReactNode
  /** 复制成功后的文本 */
  successText?: string
  /** 成功状态持续时间（毫秒） */
  successDuration?: number
  /** 是否显示图标 */
  showIcon?: boolean
}

// SVG 复制图标
function CopyIcon({ className = '' }: { className?: string }) {
  return (
    <svg
      className={className}
      width="14"
      height="14"
      viewBox="0 0 14 14"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <rect x="4.5" y="4.5" width="7.5" height="7.5" rx="1" stroke="currentColor" strokeWidth="1.2" />
      <path
        d="M2.5 9.5V2.5C2.5 2.22386 2.72386 2 3 2H10"
        stroke="currentColor"
        strokeWidth="1.2"
        strokeLinecap="round"
      />
    </svg>
  )
}

// SVG 成功图标
function CheckIcon({ className = '' }: { className?: string }) {
  return (
    <svg
      className={className}
      width="14"
      height="14"
      viewBox="0 0 14 14"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <path
        d="M2.5 7L5.5 10L11.5 4"
        stroke="currentColor"
        strokeWidth="1.5"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  )
}

export function CopyButton({
  text,
  variant = 'default',
  size = 'sm',
  className,
  children = '复制',
  successText = '已复制',
  successDuration = 2000,
  showIcon = true,
}: CopyButtonProps) {
  const [copied, setCopied] = useState(false)

  const handleCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(text)
      setCopied(true)

      // 重置状态
      setTimeout(() => {
        setCopied(false)
      }, successDuration)
    } catch (error) {
      console.error('复制失败:', error)
    }
  }, [text, successDuration])

  return (
    <Button
      variant={variant}
      size={size}
      onClick={handleCopy}
      disabled={copied}
      className={className}
      title={copied ? '已复制到剪贴板' : '点击复制'}
    >
      {showIcon && (copied ? <CheckIcon className="animate-in zoom-in-50 duration-200" /> : <CopyIcon />)}
      {copied ? successText : children}
    </Button>
  )
}
