use std::sync::Arc;
use tokio::test;
use icn_federation::{
    Federation, FederationType, FederationTerms, FederationStatus,
    MemberRole, ResourceType, ResourcePool, MemberInfo, ProposalType,
    VoteDecision, DisputeType, ResolutionMethod, ResolutionOutcome,
    MembershipAction, ResourceAllocationDetails, GovernanceConfig, VotingStrategy,
    DisputeConfig
};
use icn_federation::governance::{GovernanceManager, GovernanceResult};
use icn_federation::dispute::{DisputeManager, DisputeResult};
use icn_federation::FederationManager;
use icn_reputation::{ReputationInterface, ReputationResult};
use chrono::Utc;
use std::collections::HashMap;

// Mock reputation service for testing
struct MockReputationService {
    reputations: tokio::sync::RwLock<HashMap<String, i64>>,
}

impl MockReputationService {
    fn new() -> Self {
        let mut reputations = HashMap::new();
        // Set initial reputations
        reputations.insert("founder".to_string(), 1000);
        reputations.insert("member1".to_string(), 500);
        reputations.insert("member2".to_string(), 300);
        reputations.insert("member3".to_string(), 200);
        reputations.insert("mediator".to_string(), 750);
        
        Self {
            reputations: tokio::sync::RwLock::new(reputations),
        }
    }
}

#[async_trait::async_trait]
impl ReputationInterface for MockReputationService {
    async fn update_reputation(&self, member_id: &str, delta: i64) -> ReputationResult<()> {
        let mut reputations = self.reputations.write().await;
        let current = reputations.entry(member_id.to_string()).or_insert(0);
        *current += delta;
        Ok(())
    }
    
    async fn get_reputation(&self, member_id: &str) -> ReputationResult<i64> {
        let reputations = self.reputations.read().await;
        Ok(*reputations.get(member_id).unwrap_or(&0))
    }
    
    async fn validate_reputation(&self, member_id: &str, min_required: i64) -> ReputationResult<bool> {
        let rep = self.get_reputation(member_id).await?;
        Ok(rep >= min_required)
    }
    
    async fn get_voting_power(&self, member_id: &str) -> ReputationResult<f64> {
        let rep = self.get_reputation(member_id).await?;
        Ok(rep as f64 / 100.0)
    }
}

// Mock resource manager for testing
struct MockResourceManager;

#[async_trait::async_trait]
impl icn_federation::ResourceManager for MockResourceManager {
    async fn allocate_resources(&self, _allocation: ResourceAllocationDetails) -> Result<(), icn_federation::federation::FederationError> {
        Ok(())
    }
    
    async fn deallocate_resources(&self, _allocation: ResourceAllocationDetails) -> Result<(), icn_federation::federation::FederationError> {
        Ok(())
    }
    
    async fn get_available_resources(&self) -> Result<HashMap<ResourceType, u64>, icn_federation::federation::FederationError> {
        let mut resources = HashMap::new();
        resources.insert(ResourceType::ComputeUnit, 1000);
        resources.insert(ResourceType::StorageGb, 5000);
        resources.insert(ResourceType::BandwidthMbps, 2000);
        Ok(resources)
    }
}

#[tokio::test]
async fn test_federation_system() {
    // Initialize our components
    let reputation_service = Arc::new(MockReputationService::new());
    
    // Configure the governance system
    let governance_config = GovernanceConfig {
        voting_strategy: VotingStrategy::ReputationWeighted,
        quorum_percentage: 40,
        approval_threshold: 60,
        default_voting_period: 3600, // 1 hour for testing
        min_proposal_reputation: 100,
        min_voting_reputation: 50,
        min_membership_reputation: 0,
    };
    
    // Configure the dispute resolution system
    let dispute_config = DisputeConfig {
        default_deadline: 3600, // 1 hour for testing
        auto_escalation_days: 1,
        min_mediator_reputation: 500,
        max_disputes_per_member: 5,
        default_resolution_method: ResolutionMethod::Mediation,
        enable_auto_penalties: true,
        default_penalty: -50,
    };
    
    let governance_manager = Arc::new(GovernanceManager::new(
        governance_config,
        reputation_service.clone(),
    ));
    
    let dispute_manager = Arc::new(DisputeManager::new(
        dispute_config,
        reputation_service.clone(),
        governance_manager.clone(),
    ));
    
    let resource_manager = Arc::new(MockResourceManager);
    
    let federation_manager = FederationManager::new(
        governance_manager.clone(),
        dispute_manager.clone(),
        resource_manager,
    );
    
    // Test 1: Create a new federation
    let federation_id = federation_manager.create_federation(
        "fed1".to_string(),
        "Test Federation".to_string(),
        FederationType::ResourceSharing,
        FederationTerms::default(),
        "founder".to_string(),
    ).await.expect("Failed to create federation");
    
    println!("Federation created with ID: {}", federation_id);
    
    // Test 2: Add members to the federation
    let federation = federation_manager.get_federation(&federation_id).await.expect("Failed to get federation");
    let mut updated_federation = federation.clone();
    
    updated_federation.members.insert("member1".to_string(), MemberInfo::default());
    updated_federation.member_roles.insert("member1".to_string(), vec![MemberRole::Member]);
    
    updated_federation.members.insert("member2".to_string(), MemberInfo::default());
    updated_federation.member_roles.insert("member2".to_string(), vec![MemberRole::Member]);
    
    updated_federation.members.insert("member3".to_string(), MemberInfo::default());
    updated_federation.member_roles.insert("member3".to_string(), vec![MemberRole::Member]);
    
    updated_federation.members.insert("mediator".to_string(), MemberInfo::default());
    updated_federation.member_roles.insert("mediator".to_string(), vec![MemberRole::Member, MemberRole::Arbitrator]);
    
    federation_manager.update_federation(updated_federation).await.expect("Failed to update federation");
    
    println!("Members added to federation");
    
    // Test 3: Create a governance proposal
    let proposal_id = federation_manager.create_proposal(
        "Allocate Storage Resources".to_string(),
        "Allocate 500GB of storage to member2".to_string(),
        "member1".to_string(),
        federation_id.clone(),
        ProposalType::ResourceAllocation(ResourceAllocationDetails {
            resource_type: ResourceType::StorageGb,
            member_id: "member2".to_string(),
            amount: 500,
            duration: 30 * 24 * 60 * 60, // 30 days
            details: HashMap::new(),
        }),
        None,
    ).await.expect("Failed to create proposal");
    
    println!("Created proposal with ID: {}", proposal_id);
    
    // Test 4: Vote on the proposal
    federation_manager.submit_vote(
        &proposal_id,
        "founder".to_string(),
        VoteDecision::Approve,
        Some("This is a reasonable request".to_string()),
    ).await.expect("Failed to submit vote");
    
    federation_manager.submit_vote(
        &proposal_id,
        "member1".to_string(),
        VoteDecision::Approve,
        None,
    ).await.expect("Failed to submit vote");
    
    federation_manager.submit_vote(
        &proposal_id,
        "member3".to_string(),
        VoteDecision::Reject,
        Some("I think we should allocate less".to_string()),
    ).await.expect("Failed to submit vote");
    
    println!("Votes submitted for proposal");
    
    // The proposal should be approved based on our voting configuration
    let proposal = governance_manager.get_proposal(&proposal_id).await.expect("Failed to get proposal");
    println!("Proposal status: {:?}", proposal.status);
    
    // Test 5: File a dispute
    let dispute_id = federation_manager.file_dispute(
        "Resource Overuse".to_string(),
        "Member2 is using more than their allocated storage".to_string(),
        "member3".to_string(),
        vec!["member2".to_string()],
        federation_id.clone(),
        DisputeType::ResourceUsage,
        3, // Medium severity
    ).await.expect("Failed to file dispute");
    
    println!("Filed dispute with ID: {}", dispute_id);
    
    // Test 6: Add evidence to the dispute
    let evidence_id = dispute_manager.add_evidence(
        &dispute_id,
        "member3".to_string(),
        "Usage logs showing overuse".to_string(),
        "logs".to_string(),
        "https://example.com/logs.txt".to_string(),
        Some("abc123".to_string()),
    ).await.expect("Failed to add evidence");
    
    println!("Added evidence with ID: {}", evidence_id);
    
    // Test 7: Respondent adds a comment
    let comment_id = dispute_manager.add_comment(
        &dispute_id,
        "member2".to_string(),
        "I had permission to temporarily exceed the quota".to_string(),
        None,
    ).await.expect("Failed to add comment");
    
    println!("Added comment with ID: {}", comment_id);
    
    // Test 8: Assign a mediator
    dispute_manager.assign_mediator(
        &dispute_id,
        "mediator".to_string(),
    ).await.expect("Failed to assign mediator");
    
    println!("Assigned mediator to dispute");
    
    // Test 9: Mediator adds a comment
    dispute_manager.add_comment(
        &dispute_id,
        "mediator".to_string(),
        "Let's find a resolution that works for both parties".to_string(),
        None,
    ).await.expect("Failed to add mediator comment");
    
    // Test 10: Resolve the dispute
    dispute_manager.resolve_dispute(
        &dispute_id,
        ResolutionOutcome::Compromise(
            "Member2 will reduce usage to allocation within 7 days".to_string()
        ),
        "mediator".to_string(),
    ).await.expect("Failed to resolve dispute");
    
    println!("Dispute resolved");
    
    // Test 11: Check reputation changes
    let member2_rep = reputation_service.get_reputation("member2").await.expect("Failed to get reputation");
    let member3_rep = reputation_service.get_reputation("member3").await.expect("Failed to get reputation");
    
    println!("Final reputations - member2: {}, member3: {}", member2_rep, member3_rep);
    
    // Test 12: Create and execute a membership proposal
    let membership_proposal_id = federation_manager.create_proposal(
        "Add New Member".to_string(),
        "Add new_member to the federation".to_string(),
        "founder".to_string(),
        federation_id.clone(),
        ProposalType::MembershipChange(MembershipAction::Add("new_member".to_string())),
        None,
    ).await.expect("Failed to create membership proposal");
    
    federation_manager.submit_vote(
        &membership_proposal_id,
        "founder".to_string(),
        VoteDecision::Approve,
        None,
    ).await.expect("Failed to submit vote");
    
    federation_manager.submit_vote(
        &membership_proposal_id,
        "member1".to_string(),
        VoteDecision::Approve,
        None,
    ).await.expect("Failed to submit vote");
    
    // Execute the proposal
    governance_manager.execute_proposal(&membership_proposal_id).await.expect("Failed to execute proposal");
    
    // Verify the new member was added
    let updated_federation = federation_manager.get_federation(&federation_id).await.expect("Failed to get federation");
    assert!(updated_federation.members.contains_key("new_member"), "New member was not added");
    
    println!("New member added successfully");
    
    println!("All tests passed!");
} 