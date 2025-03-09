import React, { useState } from 'react';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { NodeMonitor } from './NodeMonitor';
import { NetworkGraph } from './NetworkGraph';
import { useNodes } from '@/hooks/useNodes';

interface WorkloadSubmission {
  type: string;
  command: string;
  cpuCores: number;
  memoryMB: number;
  targetNode?: string;
}

export const NodeDashboard = () => {
  const { nodes } = useNodes();
  const [workload, setWorkload] = useState<WorkloadSubmission>({
    type: 'container',
    command: '',
    cpuCores: 1,
    memoryMB: 256
  });

  const handleSubmitWorkload = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      const response = await fetch('http://localhost:8081/api/workloads', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          type: workload.type,
          command: workload.command,
          requirements: {
            cpu: { cores: workload.cpuCores },
            memory: { required: `${workload.memoryMB}MB` }
          },
          targetNode: workload.targetNode
        }),
      });

      if (response.ok) {
        setWorkload({
          type: 'container',
          command: '',
          cpuCores: 1,
          memoryMB: 256
        });
        // Could add a success notification here
      }
    } catch (error) {
      console.error('Error submitting workload:', error);
      // Could add an error notification here
    }
  };

  return (
    <div className="container mx-auto p-6 space-y-6">
      <h1 className="text-3xl font-bold">ICN Network Dashboard</h1>
      
      <Tabs defaultValue="monitor" className="space-y-4">
        <TabsList>
          <TabsTrigger value="monitor">Node Monitor</TabsTrigger>
          <TabsTrigger value="network">Network View</TabsTrigger>
          <TabsTrigger value="workload">Workload Management</TabsTrigger>
        </TabsList>

        <TabsContent value="monitor">
          <NodeMonitor />
        </TabsContent>

        <TabsContent value="network">
          <Card>
            <CardContent className="h-[600px] p-0">
              <NetworkGraph />
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="workload">
          <Card>
            <CardHeader>
              <CardTitle>Submit New Workload</CardTitle>
            </CardHeader>
            <CardContent>
              <form onSubmit={handleSubmitWorkload} className="space-y-4">
                <div className="space-y-2">
                  <Label htmlFor="type">Workload Type</Label>
                  <Select
                    value={workload.type}
                    onValueChange={(value) => setWorkload({...workload, type: value})}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="Select type" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="container">Container</SelectItem>
                      <SelectItem value="function">Function</SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                <div className="space-y-2">
                  <Label htmlFor="command">Command</Label>
                  <Input
                    id="command"
                    value={workload.command}
                    onChange={(e) => setWorkload({...workload, command: e.target.value})}
                    placeholder='["echo", "Hello ICN"]'
                  />
                </div>

                <div className="grid grid-cols-2 gap-4">
                  <div className="space-y-2">
                    <Label htmlFor="cpu">CPU Cores</Label>
                    <Input
                      id="cpu"
                      type="number"
                      min="1"
                      value={workload.cpuCores}
                      onChange={(e) => setWorkload({...workload, cpuCores: parseInt(e.target.value)})}
                    />
                  </div>

                  <div className="space-y-2">
                    <Label htmlFor="memory">Memory (MB)</Label>
                    <Input
                      id="memory"
                      type="number"
                      min="64"
                      value={workload.memoryMB}
                      onChange={(e) => setWorkload({...workload, memoryMB: parseInt(e.target.value)})}
                    />
                  </div>
                </div>

                <div className="space-y-2">
                  <Label htmlFor="targetNode">Target Node (Optional)</Label>
                  <Select
                    value={workload.targetNode}
                    onValueChange={(value) => setWorkload({...workload, targetNode: value})}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="Auto-select" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="">Auto-select</SelectItem>
                      {nodes.map(node => (
                        <SelectItem key={node.id} value={node.id}>
                          {node.id} ({node.metrics.status})
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>

                <Button type="submit" className="w-full">
                  Submit Workload
                </Button>
              </form>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
};