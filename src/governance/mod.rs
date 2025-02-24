mod rollback;

use rollback::{DisputeInfo, DisputeStatus, RollbackError, RollbackConfig};
use std::collections::HashMap;
use chrono::Utc;
use crate::icn_types::Proposal;

pub struct GovernanceService {
    // ...existing code...
    disputes: HashMap<String, DisputeInfo>,
    rollback_config: RollbackConfig,
    proposals: HashMap<String, Proposal>,
}

impl GovernanceService {
    pub async fn initiate_rollback(
        &mut self,
        proposal_id: &str,
        initiator: String,
        reason: String,
        evidence: Option<String>,
    ) -> Result<(), RollbackError> {
        let proposal = self.proposals
            .get(proposal_id)
            .ok_or(RollbackError::ProposalNotFound)?;

        if !self.is_within_rollback_window(&proposal) {
            return Err(RollbackError::TimeframePassed);
        }

        let dispute = DisputeInfo {
            initiator,
            reason,
            timestamp: Utc::now(),
            evidence,
            status: DisputeStatus::Pending,
            votes: HashMap::new(),
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

        if dispute.status != DisputeStatus::Pending {
            return Err(RollbackError::InvalidRollbackState);
        }

        if self.get_rollback_approvals(proposal_id) < self.rollback_config.required_approvals {
            return Err(RollbackError::UnauthorizedRollback);
        }

        self.proposals.remove(proposal_id)
            .ok_or(RollbackError::ProposalNotFound)?;
        self.disputes.remove(proposal_id);
        Ok(())
    }

    fn is_within_rollback_window(&self, proposal: &Proposal) -> bool {
        let now = Utc::now();
        let window = chrono::Duration::seconds(self.rollback_config.rollback_window.as_secs() as i64);
        now.signed_duration_since(proposal.created_at) <= window
    }

    fn freeze_proposal(&mut self, _proposal_id: &str) -> Result<(), RollbackError> {
        // Implementation to freeze proposal during dispute resolution
        Ok(())
    }

    fn get_rollback_approvals(&self, _proposal_id: &str) -> u32 {
        // Implementation to count governance members' approvals for rollback
        0
    }
}
