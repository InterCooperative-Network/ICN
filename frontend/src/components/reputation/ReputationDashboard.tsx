import React, { useState, useEffect } from 'react';
import { Card, CardHeader, CardContent, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer, AreaChart, Area } from 'recharts';
import { HeatMapGrid } from 'react-grid-heatmap';
import { Loader2 } from 'lucide-react';
import { Alert, AlertDescription } from '@/components/ui/alert';

interface ReputationData {
  timestamp: number;
  governance: number;
  resourceSharing: number;
  technicalContribution: number;
  communityEngagement: number;
  total: number;
  trend?: 'up' | 'down' | 'stable';
  milestones?: string[];
  performanceIndex?: number;
  ranking?: number;
  totalParticipants?: number;
}

interface Contribution {
  id: string;
  type: string;
  description: string;
  impact: number;
  timestamp: number;
}

const ReputationDashboard = () => {
  const [reputationHistory, setReputationHistory] = useState<ReputationData[]>([]);
  const [activityHeatmap, setActivityHeatmap] = useState<number[][]>([]);
  const [keyContributions, setKeyContributions] = useState<Contribution[]>([]);
  const [selectedPeriod, setSelectedPeriod] = useState('month');
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showMilestones, setShowMilestones] = useState(false);

  useEffect(() => {
    const fetchData = async () => {
      setIsLoading(true);
      setError(null);
      try {
        // Mock data - replace with actual API calls
        const mockReputationData = generateMockReputationData();
        const mockHeatmap = generateMockHeatmapData();
        const mockContributions = generateMockContributions();

        setReputationHistory(mockReputationData);
        setActivityHeatmap(mockHeatmap);
        setKeyContributions(mockContributions);
      } catch (err) {
        setError('Failed to load reputation data');
      } finally {
        setIsLoading(false);
      }
    };

    fetchData();
  }, [selectedPeriod]);

  const calculatePerformanceMetrics = (data: ReputationData[]) => {
    const latestData = data[data.length - 1];
    latestData.performanceIndex = Math.round((latestData.total / 1000) * 100);
    latestData.ranking = Math.floor(Math.random() * 100) + 1; // Mock ranking
    latestData.totalParticipants = 500; // Mock total participants
    latestData.milestones = [
      'First Proposal',
      'Resource Contribution',
      'Federation Member',
      'Active Voter'
    ];
    return data;
  };

  const getContributionColor = (impact: number) => {
    if (impact >= 8) return 'text-green-600';
    if (impact >= 5) return 'text-blue-600';
    return 'text-gray-600';
  };

  const getReputationLevel = (total: number) => {
    if (total >= 800) return { label: 'Elite', color: 'bg-purple-500' };
    if (total >= 500) return { label: 'Expert', color: 'bg-blue-500' };
    if (total >= 200) return { label: 'Advanced', color: 'bg-green-500' };
    return { label: 'Beginner', color: 'bg-gray-500' };
  };

  const renderPerformanceCard = () => (
    <Card className="col-span-1">
      <CardHeader>
        <CardTitle>Performance Overview</CardTitle>
      </CardHeader>
      <CardContent>
        {reputationHistory.length > 0 && (
          <div className="space-y-4">
            <div className="flex justify-between items-center">
              <span>Performance Index</span>
              <span className="font-bold">{reputationHistory[reputationHistory.length - 1].performanceIndex}%</span>
            </div>
            <div className="flex justify-between items-center">
              <span>Ranking</span>
              <span className="font-bold">#{reputationHistory[reputationHistory.length - 1].ranking}/{reputationHistory[reputationHistory.length - 1].totalParticipants}</span>
            </div>
            <div className="mt-4">
              <h4 className="font-medium mb-2">Milestones</h4>
              <div className="flex flex-wrap gap-2">
                {reputationHistory[reputationHistory.length - 1].milestones?.map((milestone, index) => (
                  <Badge key={index} variant="secondary">{milestone}</Badge>
                ))}
              </div>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-96">
        <Loader2 className="h-8 w-8 animate-spin" />
      </div>
    );
  }

  if (error) {
    return (
      <Alert variant="destructive">
        <AlertDescription>{error}</AlertDescription>
      </Alert>
    );
  }

  return (
    <div className="container mx-auto p-4 space-y-6">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold">Reputation Dashboard</h1>
        <div className="space-x-2">
          <Badge variant="outline" className="cursor-pointer" onClick={() => setSelectedPeriod('week')}>
            Week
          </Badge>
          <Badge variant="outline" className="cursor-pointer" onClick={() => setSelectedPeriod('month')}>
            Month
          </Badge>
          <Badge variant="outline" className="cursor-pointer" onClick={() => setSelectedPeriod('year')}>
            Year
          </Badge>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        {renderPerformanceCard()}
        <Card className="col-span-2">
          <CardHeader>
            <CardTitle>Reputation Over Time</CardTitle>
            {reputationHistory.length > 0 && (
              <Badge 
                className={`${getReputationLevel(reputationHistory[reputationHistory.length - 1].total).color} text-white`}
              >
                {getReputationLevel(reputationHistory[reputationHistory.length - 1].total).label}
              </Badge>
            )}
          </CardHeader>
          <CardContent>
            <div className="h-80">
              <ResponsiveContainer width="100%" height="100%">
                <AreaChart data={reputationHistory}>
                  <CartesianGrid strokeDasharray="3 3" />
                  <XAxis dataKey="timestamp" tickFormatter={(value) => new Date(value).toLocaleDateString()} />
                  <YAxis />
                  <Tooltip 
                    content={({ active, payload, label }) => {
                      if (active && payload && payload.length) {
                        return (
                          <div className="bg-white p-4 rounded-lg shadow-lg border">
                            <p className="font-bold">{new Date(label).toLocaleDateString()}</p>
                            {payload.map((entry) => (
                              <div key={entry.name} className="flex justify-between gap-4">
                                <span style={{ color: entry.color }}>{entry.name}:</span>
                                <span className="font-medium">{entry.value}</span>
                              </div>
                            ))}
                          </div>
                        );
                      }
                      return null;
                    }}
                  />
                  <Legend />
                  <Area type="monotone" dataKey="governance" stackId="1" stroke="#8884d8" fill="#8884d8" />
                  <Area type="monotone" dataKey="resourceSharing" stackId="1" stroke="#82ca9d" fill="#82ca9d" />
                  <Area type="monotone" dataKey="technicalContribution" stackId="1" stroke="#ffc658" fill="#ffc658" />
                  <Area type="monotone" dataKey="communityEngagement" stackId="1" stroke="#ff7300" fill="#ff7300" />
                </AreaChart>
              </ResponsiveContainer>
            </div>
          </CardContent>
        </Card>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <Card>
          <CardHeader>
            <CardTitle>Activity Heatmap</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="h-80">
              <HeatMapGrid
                data={activityHeatmap}
                xLabels={Array.from({ length: 7 }, (_, i) => ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'][i])}
                yLabels={Array.from({ length: 24 }, (_, i) => `${i}:00`)}
                cellHeight="20px"
                cellRender={(x, y, value) => (
                  <div title={`${value} activities`} className="w-full h-full" />
                )}
                cellStyle={(x, y, ratio) => ({
                  background: `rgb(0, 128, 0, ${ratio})`,
                  fontSize: '11px',
                })}
              />
            </div>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Key Contributions</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {keyContributions.map((contribution) => (
              <div 
                key={contribution.id}
                className="flex justify-between items-center p-4 bg-gray-50 rounded-lg"
              >
                <div>
                  <h3 className="font-medium">{contribution.type}</h3>
                  <p className="text-sm text-gray-600">{contribution.description}</p>
                </div>
                <div className="text-right">
                  <span className={`font-bold ${getContributionColor(contribution.impact)}`}>
                    +{contribution.impact}
                  </span>
                  <p className="text-sm text-gray-500">
                    {new Date(contribution.timestamp).toLocaleDateString()}
                  </p>
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  );
};

// Helper functions to generate mock data
function generateMockReputationData(): ReputationData[] {
  const data: ReputationData[] = [];
  const now = Date.now();
  const dayInMs = 86400000;

  for (let i = 30; i >= 0; i--) {
    data.push({
      timestamp: now - (i * dayInMs),
      governance: Math.floor(50 + Math.random() * 50),
      resourceSharing: Math.floor(30 + Math.random() * 40),
      technicalContribution: Math.floor(40 + Math.random() * 45),
      communityEngagement: Math.floor(35 + Math.random() * 35),
      total: Math.floor(200 + Math.random() * 100)
    });
  }
  return calculatePerformanceMetrics(data);
}

function generateMockHeatmapData(): number[][] {
  return Array.from({ length: 24 }, () =>
    Array.from({ length: 7 }, () => Math.random())
  );
}

function generateMockContributions(): Contribution[] {
  return [
    {
      id: '1',
      type: 'Governance Proposal',
      description: 'Proposed new resource allocation strategy',
      impact: 8,
      timestamp: Date.now() - 86400000
    },
    {
      id: '2',
      type: 'Technical Contribution',
      description: 'Implemented new consensus algorithm',
      impact: 10,
      timestamp: Date.now() - 172800000
    },
    {
      id: '3',
      type: 'Community Engagement',
      description: 'Organized virtual meetup',
      impact: 5,
      timestamp: Date.now() - 259200000
    }
  ];
}

export default ReputationDashboard;
