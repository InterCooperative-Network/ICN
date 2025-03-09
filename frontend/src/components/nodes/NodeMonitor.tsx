import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import { useNodes } from '@/hooks/useNodes';
import { Loader2 } from 'lucide-react';

export const NodeMonitor = () => {
  const { nodes, isLoading, error } = useNodes();

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'online': return 'bg-green-500';
      case 'offline': return 'bg-red-500';
      default: return 'bg-yellow-500';
    }
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <Loader2 className="h-8 w-8 animate-spin" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="text-red-500 text-center p-4">
        Error loading node data: {error}
      </div>
    );
  }

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      {nodes.map(node => (
        <Card key={node.id}>
          <CardHeader>
            <div className="flex justify-between items-center">
              <CardTitle>{node.id}</CardTitle>
              <Badge className={getStatusColor(node.metrics.status)}>
                {node.metrics.status}
              </Badge>
            </div>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              <div>
                <p className="text-sm font-medium mb-2">CPU Usage</p>
                <Progress value={node.metrics.cpuUsage} className="h-2" />
                <p className="text-sm text-gray-500 mt-1">{node.metrics.cpuUsage}%</p>
              </div>
              
              <div>
                <p className="text-sm font-medium mb-2">Memory Usage</p>
                <Progress value={node.metrics.memoryUsage.percentage} className="h-2" />
                <p className="text-sm text-gray-500 mt-1">
                  {Math.round(node.metrics.memoryUsage.used / 1024 / 1024 / 1024)}GB / 
                  {Math.round(node.metrics.memoryUsage.total / 1024 / 1024 / 1024)}GB
                </p>
              </div>

              <div>
                <p className="text-sm font-medium mb-2">Workloads</p>
                <div className="grid grid-cols-3 gap-2 text-center">
                  <div>
                    <p className="text-sm font-medium">{node.metrics.workloads.active}</p>
                    <p className="text-xs text-gray-500">Active</p>
                  </div>
                  <div>
                    <p className="text-sm font-medium">{node.metrics.workloads.completed}</p>
                    <p className="text-xs text-gray-500">Completed</p>
                  </div>
                  <div>
                    <p className="text-sm font-medium">{node.metrics.workloads.failed}</p>
                    <p className="text-xs text-gray-500">Failed</p>
                  </div>
                </div>
              </div>

              <div>
                <p className="text-sm font-medium mb-2">Connected Peers</p>
                <p className="text-2xl font-bold">{node.metrics.peersCount}</p>
              </div>
            </div>
          </CardContent>
        </Card>
      ))}
    </div>
  );
};