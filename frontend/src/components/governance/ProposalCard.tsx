import React from 'react';

interface ProposalCardProps {
  proposal: {
    id: string;
    title: string;
    description: string;
    status: 'active' | 'passed' | 'rejected';
    votesFor: number;
    votesAgainst: number;
    quorum: number;
    createdBy: string;
    endsAt: string;
    totalVoters: number;
    delegatedVotes: number;
  };
}

const ProposalCard: React.FC<ProposalCardProps> = ({ proposal }) => {
  return (
    <div className="card mb-4">
      <div className="card-body">
        <h5 className="card-title">{proposal.title}</h5>
        <h6 className="card-subtitle mb-2 text-muted">Status: {proposal.status}</h6>
        <p className="card-text">{proposal.description}</p>
        <p className="card-text">Votes For: {proposal.votesFor}</p>
        <p className="card-text">Votes Against: {proposal.votesAgainst}</p>
        <p className="card-text">Quorum: {proposal.quorum}</p>
        <p className="card-text">Created By: {proposal.createdBy}</p>
        <p className="card-text">Ends At: {new Date(proposal.endsAt).toLocaleDateString()}</p>
        <p className="card-text">Total Voters: {proposal.totalVoters}</p>
        <p className="card-text">Delegated Votes: {proposal.delegatedVotes}</p>
      </div>
    </div>
  );
};

export default ProposalCard;
