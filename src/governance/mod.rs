mod rollback;

use rollback::{DisputeInfo, RollbackError, RollbackConfig};
use std::collections::HashMap;

pub struct GovernanceService {
    // ...existing code...
    disputes: HashMap<String, DisputeInfo>,
    rollback_config: RollbackConfig,
}

impl GovernanceService {
    pub async fn initiate_rollback(
        &mut self,
        proposal_id: &str,
        initiator: String,
        reason: String,
        evidence: Option<String>,
    ) -> Result<(), RollbackError> {
        let proposal = self.get_proposal(proposal_id)
            .map_err(|_| RollbackError::ProposalNotFound)?;

        if !self.is_within_rollback_window(&proposal) {
            return Err(RollbackError::TimeframePassed);
        }

        let dispute = DisputeInfo {
            initiator,
            reason,
            timestamp: SystemTime::now(),
            evidence,
        };

        self.disputes.insert(proposal_id.to_string(), dispute);
        self.freeze_proposal(proposal_id)?;
        Ok(())
    }

    pub async fn execute_rollback(
        &mut self,
        proposal_id: &str,
    ) -> Result<(), RollbackError> {
        let dispute = self.disputes.get(proposal_id)
            .ok_or(RollbackError::InvalidRollbackState)?;

        if self.get_rollback_approvals(proposal_id) < self.rollback_config.required_approvals {
            return Err(RollbackError::UnauthorizedRollback);
        }

        self.remove_proposal(proposal_id)?;
        self.disputes.remove(proposal_id);
        Ok(())
    }

    fn is_within_rollback_window(&self, proposal: &Proposal) -> bool {
        SystemTime::now()
            .duration_since(proposal.created_at)
            .map(|duration| duration <= self.rollback_config.rollback_window)
            .unwrap_or(false)
    }

    fn freeze_proposal(&mut self, proposal_id: &str) -> Result<(), RollbackError> {
        // Implementation to freeze proposal during dispute resolution
        Ok(())
    }

    fn get_rollback_approvals(&self, proposal_id: &str) -> u32 {
        // Implementation to count governance members' approvals for rollback
        0
    }
}
