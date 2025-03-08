import * as React from "react"

export interface TabsProps extends React.HTMLAttributes<HTMLDivElement> {
  value?: string
  onValueChange?: (value: string) => void
}

export const Tabs = React.forwardRef<HTMLDivElement, TabsProps>(
  ({ value, onValueChange, className, children, ...props }, ref) => {
    const [selectedTab, setSelectedTab] = React.useState(value || "")

    const handleValueChange = (newValue: string) => {
      setSelectedTab(newValue)
      onValueChange?.(newValue)
    }

    return (
      <div
        ref={ref}
        className={`${className || ""}`}
        {...props}
        data-selected-tab={selectedTab}
      >
        {React.Children.map(children, (child) => {
          if (React.isValidElement(child)) {
            return React.cloneElement(child as React.ReactElement<any>, {
              selectedTab,
              onValueChange: handleValueChange,
            })
          }
          return child
        })}
      </div>
    )
  }
)
Tabs.displayName = "Tabs"

export interface TabsListProps extends React.HTMLAttributes<HTMLDivElement> {
  selectedTab?: string
  onValueChange?: (value: string) => void
}

export const TabsList = React.forwardRef<HTMLDivElement, TabsListProps>(
  ({ className, children, ...props }, ref) => {
    return (
      <div
        ref={ref}
        className={`flex space-x-1 bg-gray-100 p-1 rounded-md ${className || ""}`}
        {...props}
      >
        {children}
      </div>
    )
  }
)
TabsList.displayName = "TabsList"

export interface TabsTriggerProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  value: string
  selectedTab?: string
  onValueChange?: (value: string) => void
}

export const TabsTrigger = React.forwardRef<HTMLButtonElement, TabsTriggerProps>(
  ({ className, value, selectedTab, onValueChange, ...props }, ref) => {
    const isSelected = selectedTab === value
    
    return (
      <button
        ref={ref}
        className={`px-3 py-1.5 text-sm font-medium transition-all rounded-md
          ${isSelected ? "bg-white shadow-sm" : "text-gray-600 hover:text-gray-900"}
          ${className || ""}`}
        onClick={() => onValueChange?.(value)}
        data-selected={isSelected}
        {...props}
      />
    )
  }
)
TabsTrigger.displayName = "TabsTrigger"

export interface TabsContentProps extends React.HTMLAttributes<HTMLDivElement> {
  value: string
  selectedTab?: string
}

export const TabsContent = React.forwardRef<HTMLDivElement, TabsContentProps>(
  ({ className, value, selectedTab, ...props }, ref) => {
    const isSelected = selectedTab === value

    if (!isSelected) return null

    return (
      <div
        ref={ref}
        className={`mt-2 ${className || ""}`}
        {...props}
      />
    )
  }
)
TabsContent.displayName = "TabsContent"
