import React, { useEffect, useState, useRef } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';

interface NodeMetrics {
  cpuUsage: number;
  memoryUsage: {
    total: number;
    used: number;
    percentage: number;
  };
  peersCount: number;
  status: 'online' | 'offline' | 'error';
  workloads: {
    active: number;
    completed: number;
    failed: number;
  };
}

interface Node {
  id: string;
  type: string;
  apiPort: number;
  metrics: NodeMetrics;
}

export const NodeMonitor = () => {
  const [nodes, setNodes] = useState<Node[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const wsRef = useRef<WebSocket | null>(null);

  useEffect(() => {
    const connectWebSocket = () => {
      if (!wsRef.current) {
        try {
          wsRef.current = new WebSocket('ws://localhost:8080/api/nodes/ws');

          wsRef.current.onopen = () => {
            console.log('Node monitoring WebSocket connected');
          };

          wsRef.current.onmessage = (event) => {
            try {
              const data = JSON.parse(event.data);
              updateNodeMetrics(data);
            } catch (error) {
              console.error('Error parsing node metrics:', error);
            }
          };

          wsRef.current.onclose = () => {
            console.log('Node monitoring WebSocket disconnected, attempting to reconnect...');
            wsRef.current = null;
            setTimeout(connectWebSocket, 5000);
          };
        } catch (error) {
          console.error('Failed to establish WebSocket connection:', error);
          setTimeout(connectWebSocket, 5000);
        }
      }
    };

    const fetchInitialNodes = async () => {
      try {
        const response = await fetch('http://localhost:8080/api/nodes');
        if (response.ok) {
          const data = await response.json();
          setNodes(data);
        }
      } catch (error) {
        console.error('Error fetching nodes:', error);
      } finally {
        setIsLoading(false);
      }
    };

    fetchInitialNodes();
    connectWebSocket();

    return () => {
      if (wsRef.current) {
        wsRef.current.close();
        wsRef.current = null;
      }
    };
  }, []);

  const updateNodeMetrics = (data: any) => {
    setNodes(prev => prev.map(node => {
      if (node.id === data.nodeId) {
        return {
          ...node,
          metrics: {
            ...node.metrics,
            ...data.metrics
          }
        };
      }
      return node;
    }));
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'online': return 'bg-green-500';
      case 'offline': return 'bg-red-500';
      default: return 'bg-yellow-500';
    }
  };

  if (isLoading) {
    return <div>Loading node data...</div>;
  }

  return (
    <div className="container mx-auto p-4 space-y-6">
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
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
    </div>
  );
};