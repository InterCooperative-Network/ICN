import React from 'react';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { Progress } from '@/components/ui/progress';

interface MetricProps {
  label: string;
  value: number;
  maxValue: number;
  color: string;
}

const ReputationMetric = ({ label, value, maxValue, color }: MetricProps) => (
  <div className="space-y-2">
    <div className="flex justify-between">
      <span className="text-sm font-medium">{label}</span>
      <span className="text-sm text-muted-foreground">{value}/{maxValue}</span>
    </div>
    <Progress
      value={(value / maxValue) * 100}
      className={`h-2 ${color}`}
    />
  </div>
);

interface ReputationMetricsProps {
  governance: number;
  resourceSharing: number;
  technicalContribution: number;
  communityEngagement: number;
}

const ReputationMetrics = ({
  governance,
  resourceSharing,
  technicalContribution,
  communityEngagement,
}: ReputationMetricsProps) => {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Reputation Metrics</CardTitle>
        <CardDescription>Detailed breakdown of your reputation components</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <ReputationMetric
          label="Governance"
          value={governance}
          maxValue={100}
          color="bg-blue-500"
        />
        <ReputationMetric
          label="Resource Sharing"
          value={resourceSharing}
          maxValue={100}
          color="bg-green-500"
        />
        <ReputationMetric
          label="Technical Contribution"
          value={technicalContribution}
          maxValue={100}
          color="bg-purple-500"
        />
        <ReputationMetric
          label="Community Engagement"
          value={communityEngagement}
          maxValue={100}
          color="bg-orange-500"
        />
      </CardContent>
    </Card>
  );
};

export default ReputationMetrics;
