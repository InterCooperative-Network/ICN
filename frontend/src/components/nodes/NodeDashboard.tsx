import React, { useState } from 'react';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { NodeMonitor } from './NodeMonitor';
import { NetworkGraph } from './NetworkGraph';

interface WorkloadSubmission {
  type: string;
  command: string;
  cpuCores: number;
  memoryMB: number;
  targetNode?: string;
}

export const NodeDashboard = () => {
  const [workload, setWorkload] = useState<WorkloadSubmission>({
    type: 'container',
    command: '',
    cpuCores: 1,
    memoryMB: 256
  });

  const handleSubmitWorkload = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      const response = await fetch('http://localhost:8080/api/workloads', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(workload),
      });

      if (response.ok) {
        // Clear form and show success message
        setWorkload({
          type: 'container',
          command: '',
          cpuCores: 1,
          memoryMB: 256
        });
        // You could add a toast notification here
      }
    } catch (error) {
      console.error('Error submitting workload:', error);
      // You could add an error toast notification here
    }
  };

  return (
    <div className="container mx-auto p-6">
      <h1 className="text-3xl font-bold mb-6">ICN Network Dashboard</h1>
      
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
            <CardHeader>
              <CardTitle>Network Topology</CardTitle>
            </CardHeader>
            <CardContent className="h-[600px]">
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
                      {/* Node options will be populated dynamically */}
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