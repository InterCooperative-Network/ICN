import React, { useState, useEffect, useRef, useCallback } from 'react';
import { Card, CardHeader, CardContent, CardTitle } from '@/components/ui/card';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';
import { AlertCircle, Users, Activity, BarChart3 } from 'lucide-react';
import { Progress } from '@/components/ui/progress';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { FixedSizeList as List } from 'react-window';

const ResourceRow = ({ index, style, data }) => {
  const resource = data[index];
  return (
    <div style={style}>
      <div className="flex justify-between text-sm mb-2">
        <span>{resource.name}</span>
        <span>{resource.utilization}%</span>
      </div>
      <Progress value={resource.utilization} className="h-2" />
    </div>
  );
};

const CommunityDashboard = () => {
  const [metrics, setMetrics] = useState({
    totalMembers: 0,
    activePolicies: 0,
    resourceUtilization: 0,
    monthlyActivity: [],
    reputationCategories: {
      governance: 0,
      resourceSharing: 0,
      technicalContributions: 0
    }
  });
  
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    // Mock data - replace with actual API calls
    const mockData = {
      totalMembers: 156,
      activePolicies: 3,
      resourceUtilization: 78,
      monthlyActivity: [
        { month: 'Jan', activity: 65 },
        { month: 'Feb', activity: 75 },
        { month: 'Mar', activity: 85 },
        { month: 'Apr', activity: 90 }
      ],
      reputationCategories: {
        governance: 120,
        resourceSharing: 80,
        technicalContributions: 95
      }
    };

    setMetrics(mockData);
    setLoading(false);
  }, []);

  const resourceAllocation = [
    { name: 'Computing Resources', utilization: 75 },
    { name: 'Storage Resources', utilization: 60 },
    { name: 'Network Resources', utilization: 85 }
  ];

  const getListHeight = useCallback(() => {
    return Math.min(window.innerHeight * 0.4, 400);
  }, []);

  return (
    <div className="container mx-auto p-4 space-y-6">
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-500">Total Members</p>
                <h3 className="text-2xl font-bold">{metrics.totalMembers}</h3>
              </div>
              <Users className="h-8 w-8 text-blue-500" />
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-500">Active Policies</p>
                <h3 className="text-2xl font-bold">{metrics.activePolicies}</h3>
              </div>
              <AlertCircle className="h-8 w-8 text-green-500" />
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-500">Resource Utilization</p>
                <h3 className="text-2xl font-bold">{metrics.resourceUtilization}%</h3>
              </div>
              <Activity className="h-8 w-8 text-purple-500" />
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-500">Monthly Growth</p>
                <h3 className="text-2xl font-bold">+12%</h3>
              </div>
              <BarChart3 className="h-8 w-8 text-orange-500" />
            </div>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Community Activity</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="h-64">
            <ResponsiveContainer width="100%" height="100%">
              <LineChart data={metrics.monthlyActivity}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="month" />
                <YAxis />
                <Tooltip />
                <Legend />
                <Line 
                  type="monotone" 
                  dataKey="activity" 
                  stroke="#8884d8"
                  strokeWidth={2}
                />
              </LineChart>
            </ResponsiveContainer>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Resource Allocation</CardTitle>
        </CardHeader>
        <CardContent>
          <List
            height={getListHeight()}
            itemCount={resourceAllocation.length}
            itemSize={60}
            width="100%"
            itemData={resourceAllocation}
          >
            {ResourceRow}
          </List>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Reputation Categories</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <div>
              <div className="flex justify-between text-sm mb-2">
                <span>Governance</span>
                <span>{metrics.reputationCategories.governance}</span>
              </div>
              <Progress value={metrics.reputationCategories.governance} className="h-2" />
            </div>
            <div>
              <div className="flex justify-between text-sm mb-2">
                <span>Resource Sharing</span>
                <span>{metrics.reputationCategories.resourceSharing}</span>
              </div>
              <Progress value={metrics.reputationCategories.resourceSharing} className="h-2" />
            </div>
            <div>
              <div className="flex justify-between text-sm mb-2">
                <span>Technical Contributions</span>
                <span>{metrics.reputationCategories.technicalContributions}</span>
              </div>
              <Progress value={metrics.reputationCategories.technicalContributions} className="h-2" />
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};

export default CommunityDashboard;
