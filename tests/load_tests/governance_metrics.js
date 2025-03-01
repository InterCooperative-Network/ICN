import { Trend, Counter, Rate } from 'k6/metrics';

// Define custom metrics for governance operations
export const metrics = {
    // Proposal operations
    proposalCreationDuration: new Trend('governance_proposal_creation_duration'),
    proposalCreationFailRate: new Rate('governance_proposal_creation_fail_rate'),
    proposalCreationCount: new Counter('governance_proposal_creation_count'),
    
    // Voting operations
    voteCastDuration: new Trend('governance_vote_cast_duration'),
    voteCastFailRate: new Rate('governance_vote_cast_fail_rate'),
    voteCastCount: new Counter('governance_vote_cast_count'),
    
    // Proposal execution operations
    proposalExecutionDuration: new Trend('governance_proposal_execution_duration'),
    proposalExecutionFailRate: new Rate('governance_proposal_execution_fail_rate'),
    proposalExecutionCount: new Counter('governance_proposal_execution_count')
};

// Helper function to record proposal creation metrics
export function recordProposalCreation(response, startTime) {
    const duration = Date.now() - startTime;
    metrics.proposalCreationDuration.add(duration);
    metrics.proposalCreationCount.add(1);
    
    if (response.status !== 200) {
        metrics.proposalCreationFailRate.add(1);
    } else {
        metrics.proposalCreationFailRate.add(0);
    }
}

// Helper function to record vote casting metrics
export function recordVoteCast(response, startTime) {
    const duration = Date.now() - startTime;
    metrics.voteCastDuration.add(duration);
    metrics.voteCastCount.add(1);
    
    if (response.status !== 200) {
        metrics.voteCastFailRate.add(1);
    } else {
        metrics.voteCastFailRate.add(0);
    }
}

// Helper function to record proposal execution metrics
export function recordProposalExecution(response, startTime) {
    const duration = Date.now() - startTime;
    metrics.proposalExecutionDuration.add(duration);
    metrics.proposalExecutionCount.add(1);
    
    if (response.status !== 200) {
        metrics.proposalExecutionFailRate.add(1);
    } else {
        metrics.proposalExecutionFailRate.add(0);
    }
}
