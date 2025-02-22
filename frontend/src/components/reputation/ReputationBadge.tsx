import React from 'react';
import { Badge } from '@/components/ui/badge';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';

interface ReputationBadgeProps {
  score: number;
  showTooltip?: boolean;
}

const ReputationBadge: React.FC<ReputationBadgeProps> = ({ score, showTooltip = true }) => {
  const getLevelInfo = (score: number) => {
    if (score >= 800) return { label: 'Elite', color: 'bg-purple-500', description: 'Top contributor with exceptional impact' };
    if (score >= 500) return { label: 'Expert', color: 'bg-blue-500', description: 'Highly respected community member' };
    if (score >= 200) return { label: 'Advanced', color: 'bg-green-500', description: 'Active and valued participant' };
    return { label: 'Beginner', color: 'bg-gray-500', description: 'Starting the journey' };
  };

  const levelInfo = getLevelInfo(score);

  if (!showTooltip) {
    return (
      <Badge className={`${levelInfo.color} text-white`}>
        {levelInfo.label}
      </Badge>
    );
  }

  return (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger>
          <Badge className={`${levelInfo.color} text-white`}>
            {levelInfo.label}
          </Badge>
        </TooltipTrigger>
        <TooltipContent>
          <p>{levelInfo.description}</p>
          <p className="text-sm text-gray-500">Score: {score}</p>
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
};

export default ReputationBadge;
