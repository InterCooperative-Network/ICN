import { useState, useEffect, useCallback } from 'react';
import { useWebSocket } from './useWebSocket';

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

interface NetworkTopology {
  nodes: Node[];
  links: Array<{ source: string; target: string }>;
}

interface UseNodesResult {
  nodes: Node[];
  topology: NetworkTopology | null;
  isLoading: boolean;
  error: string | null;
  refreshNodes: () => Promise<void>;
}

export function useNodes(): UseNodesResult {
  const [nodes, setNodes] = useState<Node[]>([]);
  const [topology, setTopology] = useState<NetworkTopology | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const handleWebSocketMessage = useCallback((data: any) => {
    if (data.type === 'nodeUpdate') {
      setNodes(prev => prev.map(node => 
        node.id === data.nodeId ? { ...node, metrics: { ...node.metrics, ...data.metrics } } : node
      ));
    } else if (data.type === 'topologyUpdate') {
      setTopology(data.topology);
    }
  }, []);

  const { sendMessage } = useWebSocket({
    url: 'ws://localhost:8081/ws',
    onMessage: handleWebSocketMessage,
    onOpen: () => setError(null),
    onError: () => setError('WebSocket connection error'),
  });

  const fetchNodes = useCallback(async () => {
    try {
      setIsLoading(true);
      setError(null);
      
      const response = await fetch('http://localhost:8081/api/nodes');
      if (!response.ok) {
        throw new Error('Failed to fetch nodes');
      }
      
      const data = await response.json();
      setNodes(data);
      
      const topologyResponse = await fetch('http://localhost:8081/api/network/topology');
      if (topologyResponse.ok) {
        const topologyData = await topologyResponse.json();
        setTopology(topologyData);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch nodes');
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchNodes();
    const interval = setInterval(fetchNodes, 30000); // Fallback polling every 30s
    return () => clearInterval(interval);
  }, [fetchNodes]);

  return {
    nodes,
    topology,
    isLoading,
    error,
    refreshNodes: fetchNodes,
  };
}