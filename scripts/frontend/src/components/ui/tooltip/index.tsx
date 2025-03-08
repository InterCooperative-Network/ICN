import * as React from "react"

export interface TooltipProps {
  children: React.ReactNode
}

export const Tooltip = ({ children }: TooltipProps) => {
  return (
    <div className="relative inline-block">{children}</div>
  )
}

export interface TooltipTriggerProps extends React.HTMLAttributes<HTMLDivElement> {}

export const TooltipTrigger = React.forwardRef<HTMLDivElement, TooltipTriggerProps>(
  ({ className, ...props }, ref) => {
    return (
      <div
        ref={ref}
        className={`inline-block cursor-pointer ${className || ""}`}
        {...props}
      />
    )
  }
)
TooltipTrigger.displayName = "TooltipTrigger"

export interface TooltipContentProps extends React.HTMLAttributes<HTMLDivElement> {
  side?: "top" | "right" | "bottom" | "left"
}

export const TooltipContent = React.forwardRef<HTMLDivElement, TooltipContentProps>(
  ({ className, side = "top", ...props }, ref) => {
    const sidePositions = {
      top: "bottom-full mb-1",
      right: "left-full ml-1",
      bottom: "top-full mt-1",
      left: "right-full mr-1",
    }

    return (
      <div
        ref={ref}
        className={`absolute z-50 px-3 py-2 text-sm bg-black text-white rounded shadow-md ${sidePositions[side]} ${className || ""}`}
        {...props}
      />
    )
  }
)
TooltipContent.displayName = "TooltipContent"

export const TooltipProvider = ({ children }: { children: React.ReactNode }) => {
  return <>{children}</>
}
