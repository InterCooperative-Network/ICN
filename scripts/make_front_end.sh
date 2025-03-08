#!/bin/bash
# Navigate to the frontend directory
cd frontend

# Install required dependencies
npm install --save lucide-react recharts react-window react-grid-heatmap @reach/dialog

# Install shadcn/ui components
# First, make sure we have the necessary tooling
npm install --save-dev shadcn-ui

# Create directories for UI components if they don't exist
mkdir -p src/components/ui

# Create a tsconfig paths config to support @/ imports
cat > tsconfig.paths.json << EOL
{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "@/*": ["./src/*"]
    }
  }
}
EOL

# Update the main tsconfig.json to extend the paths config
cat > tsconfig.json << EOL
{
  "extends": "./tsconfig.paths.json",
  "compilerOptions": {
    "target": "es5",
    "lib": ["dom", "dom.iterable", "esnext"],
    "allowJs": true,
    "skipLibCheck": true,
    "esModuleInterop": true,
    "allowSyntheticDefaultImports": true,
    "strict": true,
    "forceConsistentCasingInFileNames": true,
    "noFallthroughCasesInSwitch": true,
    "module": "esnext",
    "moduleResolution": "node",
    "resolveJsonModule": true,
    "isolatedModules": true,
    "jsx": "react-jsx",
    "noEmit": true
  },
  "include": ["src"]
}
EOL

# Create basic UI components that are missing

# Progress component
mkdir -p src/components/ui/progress
cat > src/components/ui/progress/index.tsx << EOL
import * as React from "react"

export interface ProgressProps extends React.HTMLAttributes<HTMLDivElement> {
  value?: number
  max?: number
}

export const Progress = React.forwardRef<HTMLDivElement, ProgressProps>(
  ({ value = 0, max = 100, className, ...props }, ref) => {
    return (
      <div
        ref={ref}
        className={\`w-full h-2 bg-gray-200 rounded-full overflow-hidden \${className}\`}
        {...props}
      >
        <div
          className="h-full bg-blue-500 transition-all duration-300 ease-in-out"
          style={{ width: \`\${(value / max) * 100}%\` }}
        />
      </div>
    )
  }
)

Progress.displayName = "Progress"
EOL

# Card components
mkdir -p src/components/ui/card
cat > src/components/ui/card/index.tsx << EOL
import * as React from "react"

export interface CardProps extends React.HTMLAttributes<HTMLDivElement> {}

export const Card = React.forwardRef<HTMLDivElement, CardProps>(
  ({ className, ...props }, ref) => {
    return (
      <div
        ref={ref}
        className={\`rounded-lg border bg-white shadow-sm \${className || ""}\`}
        {...props}
      />
    )
  }
)
Card.displayName = "Card"

export interface CardHeaderProps extends React.HTMLAttributes<HTMLDivElement> {}

export const CardHeader = React.forwardRef<HTMLDivElement, CardHeaderProps>(
  ({ className, ...props }, ref) => {
    return (
      <div
        ref={ref}
        className={\`flex flex-col space-y-1.5 p-6 \${className || ""}\`}
        {...props}
      />
    )
  }
)
CardHeader.displayName = "CardHeader"

export interface CardTitleProps extends React.HTMLAttributes<HTMLHeadingElement> {}

export const CardTitle = React.forwardRef<HTMLHeadingElement, CardTitleProps>(
  ({ className, ...props }, ref) => {
    return (
      <h3
        ref={ref}
        className={\`text-lg font-semibold \${className || ""}\`}
        {...props}
      />
    )
  }
)
CardTitle.displayName = "CardTitle"

export interface CardContentProps extends React.HTMLAttributes<HTMLDivElement> {}

export const CardContent = React.forwardRef<HTMLDivElement, CardContentProps>(
  ({ className, ...props }, ref) => {
    return (
      <div
        ref={ref}
        className={\`p-6 pt-0 \${className || ""}\`}
        {...props}
      />
    )
  }
)
CardContent.displayName = "CardContent"
EOL

# Tabs components
mkdir -p src/components/ui/tabs
cat > src/components/ui/tabs/index.tsx << EOL
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
        className={\`\${className || ""}\`}
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
        className={\`flex space-x-1 bg-gray-100 p-1 rounded-md \${className || ""}\`}
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
        className={\`px-3 py-1.5 text-sm font-medium transition-all rounded-md
          \${isSelected ? "bg-white shadow-sm" : "text-gray-600 hover:text-gray-900"}
          \${className || ""}\`}
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
        className={\`mt-2 \${className || ""}\`}
        {...props}
      />
    )
  }
)
TabsContent.displayName = "TabsContent"
EOL

# Alert components
mkdir -p src/components/ui/alert
cat > src/components/ui/alert/index.tsx << EOL
import * as React from "react"

export interface AlertProps extends React.HTMLAttributes<HTMLDivElement> {}

export const Alert = React.forwardRef<HTMLDivElement, AlertProps>(
  ({ className, ...props }, ref) => {
    return (
      <div
        ref={ref}
        className={\`relative w-full rounded-lg border p-4 \${className || ""}\`}
        {...props}
      />
    )
  }
)
Alert.displayName = "Alert"

export interface AlertDescriptionProps extends React.HTMLAttributes<HTMLParagraphElement> {}

export const AlertDescription = React.forwardRef<HTMLParagraphElement, AlertDescriptionProps>(
  ({ className, ...props }, ref) => {
    return (
      <p
        ref={ref}
        className={\`text-sm \${className || ""}\`}
        {...props}
      />
    )
  }
)
AlertDescription.displayName = "AlertDescription"
EOL

# Button component
mkdir -p src/components/ui/button
cat > src/components/ui/button/index.tsx << EOL
import * as React from "react"

export interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: "default" | "outline" | "ghost"
  size?: "sm" | "md" | "lg"
}

export const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant = "default", size = "md", ...props }, ref) => {
    const variantStyles = {
      default: "bg-blue-500 text-white hover:bg-blue-600",
      outline: "border border-gray-300 hover:bg-gray-50",
      ghost: "hover:bg-gray-100",
    }

    const sizeStyles = {
      sm: "px-3 py-1 text-sm",
      md: "px-4 py-2",
      lg: "px-6 py-3 text-lg",
    }

    return (
      <button
        ref={ref}
        className={\`inline-flex items-center justify-center rounded-md font-medium transition-all focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none
        \${variantStyles[variant]} \${sizeStyles[size]} \${className || ""}\`}
        {...props}
      />
    )
  }
)
Button.displayName = "Button"
EOL

# Badge component
mkdir -p src/components/ui/badge
cat > src/components/ui/badge/index.tsx << EOL
import * as React from "react"

export interface BadgeProps extends React.HTMLAttributes<HTMLDivElement> {
  variant?: "default" | "secondary" | "success" | "warning" | "danger"
}

export const Badge = React.forwardRef<HTMLDivElement, BadgeProps>(
  ({ className, variant = "default", ...props }, ref) => {
    const variantStyles = {
      default: "bg-blue-100 text-blue-800",
      secondary: "bg-gray-100 text-gray-800",
      success: "bg-green-100 text-green-800",
      warning: "bg-yellow-100 text-yellow-800",
      danger: "bg-red-100 text-red-800",
    }

    return (
      <div
        ref={ref}
        className={\`inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium \${variantStyles[variant]} \${className || ""}\`}
        {...props}
      />
    )
  }
)
Badge.displayName = "Badge"
EOL

# Tooltip component
mkdir -p src/components/ui/tooltip
cat > src/components/ui/tooltip/index.tsx << EOL
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
        className={\`inline-block cursor-pointer \${className || ""}\`}
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
        className={\`absolute z-50 px-3 py-2 text-sm bg-black text-white rounded shadow-md \${sidePositions[side]} \${className || ""}\`}
        {...props}
      />
    )
  }
)
TooltipContent.displayName = "TooltipContent"

export const TooltipProvider = ({ children }: { children: React.ReactNode }) => {
  return <>{children}</>
}