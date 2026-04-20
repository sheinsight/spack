import * as React from 'react'
import { cva, type VariantProps } from 'class-variance-authority'
import { Slot } from 'radix-ui'

import { cn } from '@/lib/utils'

const buttonVariants = cva(
  'rounded-md text-xs/relaxed font-medium transition-all disabled:pointer-events-none disabled:opacity-50 inline-flex items-center justify-center whitespace-nowrap outline-none select-none shrink-0 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/50',
  {
    variants: {
      variant: {
        default: 'bg-primary text-primary-foreground hover:bg-primary/90',
        outline: 'border border-primary/30 bg-background hover:bg-primary/10 text-foreground',
        ghost: 'hover:bg-muted hover:text-foreground text-muted-foreground',
      },
      size: {
        default: 'h-7 gap-1.5 px-3',
        xs: 'h-5 gap-1 px-2 text-[0.625rem]',
        sm: 'h-6 gap-1 px-2.5 text-xs',
        lg: 'h-9 gap-2 px-4',
      },
    },
    defaultVariants: {
      variant: 'default',
      size: 'default',
    },
  },
)

function Button({
  className,
  variant = 'default',
  size = 'default',
  asChild = false,
  ...props
}: React.ComponentProps<'button'> &
  VariantProps<typeof buttonVariants> & {
    asChild?: boolean
  }) {
  const Comp = asChild ? Slot.Root : 'button'

  return (
    <Comp
      data-slot="button"
      data-variant={variant}
      data-size={size}
      className={cn(buttonVariants({ variant, size, className }))}
      {...props}
    />
  )
}

export { Button, buttonVariants }
