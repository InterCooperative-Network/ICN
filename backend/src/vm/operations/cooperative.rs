use std::collections::HashMap;
use super::{Operation, VMState, VMResult, ensure_permissions, ensure_reputation, emit_event};

/// Types of operations that can be performed on cooperatives
pub enum CooperativeOperation {
    /// Create a new cooperative
    CreateCooperative {
        name: String,
        description: String,
        resource_policies: HashMap<String, ResourcePolicy>,
        membership_requirements: Vec<MembershipRequirement>,
    },
    
    /// Join an existing cooperative
    JoinCooperative {
        cooperative_id: String,
        role: String,
        qualifications: Vec<String>,
    },
    
    /// Leave a cooperative
    LeaveCooperative {
        cooperative_id: String,
        exit_reason: String,
    },
    
    /// Allocate resources within a cooperative
    AllocateResource {
        resource_type: String,
        amount: u64,
        recipient: String,
        purpose: String,
    },
    
    /// Transfer resources between cooperatives
    TransferResource {
        from_cooperative: String,
        to_cooperative: String,
        resource_type: String,
        amount: u64,
        terms: Vec<String>,
    },
    
    /// Create resource sharing agreement
    CreateSharingAgreement {
        partner_cooperative: String,
        resources: Vec<ResourceDefinition>,
        duration: u64,
        terms: Vec<String>,
    },
    
    /// Update cooperative metadata
    UpdateMetadata {
        cooperative_id: String,
        updates: HashMap<String, String>,
    },
    
    /// Add role to cooperative
    AddRole {
        role_name: String,
        permissions: Vec<String>,
        requirements: Vec<MembershipRequirement>,
    },
    
    /// Initiate federation with another cooperative
    InitiateFederation {
        partner_cooperative: String,
        federation_type: FederationType,
        terms: Vec<String>,
    },
}

#[derive(Debug, Clone)]
pub struct ResourcePolicy {
    pub resource_type: String,
    pub allocation_limit: u64,
    pub replenishment_rate: u64,
    pub required_reputation: i64,
    pub sharing_permissions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct MembershipRequirement {
    pub requirement_type: RequirementType,
    pub threshold: i64,
    pub verification_method: String,
}

#[derive(Debug, Clone)]
pub enum RequirementType {
    Reputation,
    ContributionCount,
    SkillLevel,
    TimeCommitment,
    ResourceStake,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct ResourceDefinition {
    pub resource_type: String,
    pub quantity: u64,
    pub access_level: AccessLevel,
    pub conditions: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum AccessLevel {
    ReadOnly,
    ReadWrite,
    FullAccess,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum FederationType {
    ResourceSharing,
    JointProjects,
    MutualAid,
    FullIntegration,
    Custom(String),
}

impl Operation for CooperativeOperation {
    fn execute(&self, state: &mut VMState) -> VMResult<()> {
        match self {
            CooperativeOperation::CreateCooperative { 
                name, 
                description, 
                resource_policies,
                membership_requirements 
            } => {
                ensure_permissions(&["cooperative.create".to_string()], &state.permissions)?;
                
                let reputation = state.reputation_context
                    .get(&state.caller_did)
                    .copied()
                    .unwrap_or(0);
                
                ensure_reputation(200, reputation)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("name".to_string(), name.clone());
                event_data.insert("description".to_string(), description.clone());
                event_data.insert("creator".to_string(), state.caller_did.clone());
                event_data.insert("policy_count".to_string(), resource_policies.len().to_string());
                event_data.insert("requirement_count".to_string(), membership_requirements.len().to_string());
                
                emit_event(state, "CooperativeCreated".to_string(), event_data);
                Ok(())
            },
            
            CooperativeOperation::JoinCooperative { cooperative_id, role, qualifications } => {
                ensure_permissions(&["cooperative.join".to_string()], &state.permissions)?;
                
                let reputation = state.reputation_context
                    .get(&state.caller_did)
                    .copied()
                    .unwrap_or(0);
                
                ensure_reputation(50, reputation)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("cooperative_id".to_string(), cooperative_id.clone());
                event_data.insert("role".to_string(), role.clone());
                event_data.insert("member_did".to_string(), state.caller_did.clone());
                event_data.insert("qualifications".to_string(), qualifications.join(","));
                
                emit_event(state, "CooperativeMemberAdded".to_string(), event_data);
                Ok(())
            },
            
            CooperativeOperation::LeaveCooperative { cooperative_id, exit_reason } => {
                let mut event_data = HashMap::new();
                event_data.insert("cooperative_id".to_string(), cooperative_id.clone());
                event_data.insert("member_did".to_string(), state.caller_did.clone());
                event_data.insert("exit_reason".to_string(), exit_reason.clone());
                
                emit_event(state, "CooperativeMemberLeft".to_string(), event_data);
                Ok(())
            },
            
            CooperativeOperation::AllocateResource { 
                resource_type, 
                amount, 
                recipient, 
                purpose 
            } => {
                ensure_permissions(&["resource.allocate".to_string()], &state.permissions)?;
                
                let reputation = state.reputation_context
                    .get(&state.caller_did)
                    .copied()
                    .unwrap_or(0);
                
                ensure_reputation(100, reputation)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("resource_type".to_string(), resource_type.clone());
                event_data.insert("amount".to_string(), amount.to_string());
                event_data.insert("recipient".to_string(), recipient.clone());
                event_data.insert("purpose".to_string(), purpose.clone());
                
                emit_event(state, "ResourceAllocated".to_string(), event_data);
                Ok(())
            },
            
            CooperativeOperation::TransferResource { 
                from_cooperative, 
                to_cooperative, 
                resource_type, 
                amount, 
                terms 
            } => {
                ensure_permissions(&["resource.transfer".to_string()], &state.permissions)?;
                
                let reputation = state.reputation_context
                    .get(&state.caller_did)
                    .copied()
                    .unwrap_or(0);
                
                ensure_reputation(150, reputation)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("from_cooperative".to_string(), from_cooperative.clone());
                event_data.insert("to_cooperative".to_string(), to_cooperative.clone());
                event_data.insert("resource_type".to_string(), resource_type.clone());
                event_data.insert("amount".to_string(), amount.to_string());
                event_data.insert("terms".to_string(), terms.join(","));
                
                emit_event(state, "ResourceTransferred".to_string(), event_data);
                Ok(())
            },
            
            CooperativeOperation::CreateSharingAgreement { 
                partner_cooperative, 
                resources, 
                duration, 
                terms 
            } => {
                ensure_permissions(&["agreement.create".to_string()], &state.permissions)?;
                
                let reputation = state.reputation_context
                    .get(&state.caller_did)
                    .copied()
                    .unwrap_or(0);
                
                ensure_reputation(200, reputation)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("partner_cooperative".to_string(), partner_cooperative.clone());
                event_data.insert("resource_count".to_string(), resources.len().to_string());
                event_data.insert("duration".to_string(), duration.to_string());
                event_data.insert("terms".to_string(), terms.join(","));
                
                emit_event(state, "SharingAgreementCreated".to_string(), event_data);
                Ok(())
            },
            
            CooperativeOperation::UpdateMetadata { cooperative_id, updates } => {
                ensure_permissions(&["cooperative.update".to_string()], &state.permissions)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("cooperative_id".to_string(), cooperative_id.clone());
                for (key, value) in updates {
                    event_data.insert(format!("update_{}", key), value.clone());
                }
                
                emit_event(state, "CooperativeMetadataUpdated".to_string(), event_data);
                Ok(())
            },
            
            CooperativeOperation::AddRole { 
                role_name, 
                permissions, 
                requirements 
            } => {
                ensure_permissions(&["role.create".to_string()], &state.permissions)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("role_name".to_string(), role_name.clone());
                event_data.insert("permissions".to_string(), permissions.join(","));
                event_data.insert("requirement_count".to_string(), requirements.len().to_string());
                
                emit_event(state, "RoleCreated".to_string(), event_data);
                Ok(())
            },
            
            CooperativeOperation::InitiateFederation { 
                partner_cooperative, 
                federation_type, 
                terms 
            } => {
                ensure_permissions(&["federation.initiate".to_string()], &state.permissions)?;
                
                let reputation = state.reputation_context
                    .get(&state.caller_did)
                    .copied()
                    .unwrap_or(0);
                
                ensure_reputation(300, reputation)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("partner_cooperative".to_string(), partner_cooperative.clone());
                event_data.insert("federation_type".to_string(), format!("{:?}", federation_type));
                event_data.insert("terms".to_string(), terms.join(","));
                
                emit_event(state, "FederationInitiated".to_string(), event_data);
                Ok(())
            },
        }
    }

    fn resource_cost(&self) -> u64 {
        match self {
            CooperativeOperation::CreateCooperative { .. } => 500,
            CooperativeOperation::JoinCooperative { .. } => 100,
            CooperativeOperation::LeaveCooperative { .. } => 50,
            CooperativeOperation::AllocateResource { .. } => 200,
            CooperativeOperation::TransferResource { .. } => 300,
            CooperativeOperation::CreateSharingAgreement { .. } => 400,
            CooperativeOperation::UpdateMetadata { .. } => 100,
            CooperativeOperation::AddRole { .. } => 150,
            CooperativeOperation::InitiateFederation { .. } => 1000,
        }
    }

    fn required_permissions(&self) -> Vec<String> {
        match self {
            CooperativeOperation::CreateCooperative { .. } => vec!["cooperative.create".to_string()],
            CooperativeOperation::JoinCooperative { .. } => vec!["cooperative.join".to_string()],
            CooperativeOperation::AllocateResource { .. } => vec!["resource.allocate".to_string()],
            CooperativeOperation::TransferResource { .. } => vec!["resource.transfer".to_string()],
            CooperativeOperation::CreateSharingAgreement { .. } => vec!["agreement.create".to_string()],
            CooperativeOperation::UpdateMetadata { .. } => vec!["cooperative.update".to_string()],
            CooperativeOperation::AddRole { .. } => vec!["role.create".to_string()],
            CooperativeOperation::InitiateFederation { .. } => vec!["federation.initiate".to_string()],
            _ => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_state() -> VMState {
        VMState {
            stack: Vec::new(),
            memory: HashMap::new(),
            events: Vec::new(),
            instruction_pointer: 0,
            reputation_context: HashMap::new(),
            caller_did: "test_caller".to_string(),
            block_number: 1,
            timestamp: 1000,
            permissions: vec![
                "cooperative.create".to_string(),
                "cooperative.join".to_string(),
                "resource.allocate".to_string(),
                "agreement.create".to_string(),
            ],
            memory_limit: 1024 * 1024, // 1MB default limit
            memory_address_counter: AtomicU64::new(0),
        }
    }

    #[test]
    fn test_create_cooperative() {
        let mut state = setup_test_state();
        let resource_policies = HashMap::new();
        let membership_requirements = vec![];
        
        let op = CooperativeOperation::CreateCooperative {
            name: "Test Coop".to_string(),
            description: "Test Description".to_string(),
            resource_policies,
            membership_requirements,
        };
        
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events[0].event_type, "CooperativeCreated");
    }

    #[test]
    fn test_allocate_resource() {
        let mut state = setup_test_state();
        let op = CooperativeOperation::AllocateResource {
            resource_type: "computing".to_string(),
            amount: 100,
            recipient: "test_recipient".to_string(),
            purpose: "testing".to_string(),
        };
        
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events[0].event_type, "ResourceAllocated");
    }

    #[test]
    fn test_insufficient_reputation() {
        let mut state = setup_test_state();
        state.reputation_context.insert(state.caller_did.clone(), 100);
        
        let op = CooperativeOperation::InitiateFederation {
            partner_cooperative: "partner".to_string(),
            federation_type: FederationType::ResourceSharing,
            terms: vec!["term1".to_string()],
        };
        
        assert!(matches!(op.execute(&mut state), Err(_)));
    }
}
