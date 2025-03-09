import * as React from "react";

interface TabsProps extends React.HTMLAttributes<HTMLDivElement> {
  defaultValue?: string;
  value?: string;
  onValueChange?: (value: string) => void;
}

const Tabs = React.forwardRef<HTMLDivElement, TabsProps>(
  ({ className, defaultValue, value, onValueChange, ...props }, ref) => {
    const [selectedValue, setSelectedValue] = React.useState(value || defaultValue || "");
    
    const handleValueChange = (newValue: string) => {
      setSelectedValue(newValue);
      onValueChange?.(newValue);
    };
    
    return (
      <div
        ref={ref}
        className={`data-[state=active]:bg-background ${className || ""}`}
        {...props}
        data-state={selectedValue ? "active" : "inactive"}
        data-value={selectedValue}
      />
    );
  }
);
Tabs.displayName = "Tabs";

interface TabsListProps extends React.HTMLAttributes<HTMLDivElement> {}

const TabsList = React.forwardRef<HTMLDivElement, TabsListProps>(
  ({ className, ...props }, ref) => (
    <div
      ref={ref}
      className={`inline-flex h-10 items-center justify-center rounded-md bg-muted p-1 text-muted-foreground ${className || ""}`}
      role="tablist"
      {...props}
    />
  )
);
TabsList.displayName = "TabsList";

interface TabsTriggerProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  value: string;
}

const TabsTrigger = React.forwardRef<HTMLButtonElement, TabsTriggerProps>(
  ({ className, value, ...props }, ref) => {
    const parent = React.useContext(TabsContext);
    const isSelected = parent?.value === value;
    
    return (
      <button
        ref={ref}
        role="tab"
        className={`inline-flex items-center justify-center whitespace-nowrap rounded-sm px-3 py-1.5 text-sm font-medium ring-offset-background transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 data-[state=active]:bg-background data-[state=active]:text-foreground data-[state=active]:shadow-sm ${className || ""}`}
        data-state={isSelected ? "active" : "inactive"}
        onClick={() => parent?.onValueChange?.(value)}
        {...props}
      />
    );
  }
);
TabsTrigger.displayName = "TabsTrigger";

interface TabsContentProps extends React.HTMLAttributes<HTMLDivElement> {
  value: string;
}

const TabsContent = React.forwardRef<HTMLDivElement, TabsContentProps>(
  ({ className, value, ...props }, ref) => {
    const parent = React.useContext(TabsContext);
    const isSelected = parent?.value === value;
    
    if (!isSelected) return null;
    
    return (
      <div
        ref={ref}
        role="tabpanel"
        className={`mt-2 ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 ${className || ""}`}
        {...props}
      />
    );
  }
);
TabsContent.displayName = "TabsContent";

// Context for managing tab state
interface TabsContextType {
  value?: string;
  onValueChange?: (value: string) => void;
}

const TabsContext = React.createContext<TabsContextType | undefined>(undefined);

// Provide a context wrapper component
const TabsProvider: React.FC<TabsProps & { children: React.ReactNode }> = ({
  value,
  defaultValue,
  onValueChange,
  children,
}) => {
  const [selectedValue, setSelectedValue] = React.useState(value || defaultValue || "");
  
  React.useEffect(() => {
    if (value !== undefined) setSelectedValue(value);
  }, [value]);
  
  const handleValueChange = React.useCallback((newValue: string) => {
    setSelectedValue(newValue);
    onValueChange?.(newValue);
  }, [onValueChange]);
  
  return (
    <TabsContext.Provider value={{ value: selectedValue, onValueChange: handleValueChange }}>
      {children}
    </TabsContext.Provider>
  );
};

// Exported component with provider
const TabsRoot: React.FC<TabsProps & { children: React.ReactNode }> = (props) => {
  return (
    <TabsProvider {...props}>
      <Tabs {...props} />
    </TabsProvider>
  );
};

export { TabsRoot as Tabs, TabsList, TabsTrigger, TabsContent };