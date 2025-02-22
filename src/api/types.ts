export interface Proposal {
  id: string;
  title: string;
  description: string;
  status: 'pending' | 'active' | 'completed';
  votes: Vote[];
}

export interface Vote {
  userId: string;
  proposalId: string;
  choice: 'yes' | 'no' | 'abstain';
  timestamp: Date;
}

export interface APIResponse<T> {
  success: boolean;
  data?: T;
  error?: {
    code: string;
    message: string;
  };
}
