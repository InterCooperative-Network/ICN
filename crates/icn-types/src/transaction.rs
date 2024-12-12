// crates/icn-types/src/transaction.rs
//
// This module defines the transaction types for the Inter-Cooperative Network (ICN).
// ICN uses a cooperative model based on relationships and reputation rather than
// traditional blockchain economic models. Transactions represent cooperative 
// actions, relationship building, and resource sharing between network participants.

use std::time::{SystemTime, UNIX_EPOCH};
use blake3::Hash;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::identity::{DID, Signature};
use crate::relationship::RelationshipProof;
use crate::Validate;

/// A transaction in the ICN network representing a cooperative action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Transaction header containing metadata
    pub header: TransactionHeader,
    
    /// Type of cooperative action and associated data
    pub action: CooperativeAction,
    
    /// Transaction sender's DID
    pub sender: DID,
    
    /// Required relationship proof for the action
    pub relationship_proof: Option<RelationshipProof>,
    
    /// Transaction signature
    pub signature: Option<Signature>,
    
    /// Sender's reputation score at time of transaction
    pub sender_reputation: i64,
}

/// Transaction metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionHeader {
    /// Sequence number for sender's transactions
    pub sequence: u64,
    
    /// Transaction timestamp
    pub timestamp: u64,
    
    /// Optional transaction expiration
    pub expires_at: Option<u64>,
    
    /// Transaction format version
    pub version: u32,
}

/// Types of cooperative actions that can be performed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CooperativeAction {
    /// Membership management within cooperatives
    Membership(MembershipAction),
    
    /// Building and managing relationships
    Relationship(RelationshipAction),
    
    /// Sharing and exchanging resources
    Resource(ResourceAction),
    
    /// Cooperative governance actions
    Governance(GovernanceAction),
    
    /// Contribution tracking and acknowledgment
    Contribution(ContributionAction),
}

/// Actions related to cooperative membership
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MembershipAction {
    /// Join a cooperative
    Join {
        /// ID of the cooperative
        cooperative_id: String,
        /// Type of membership requested
        member_type: MemberType,
        /// Member profile and capabilities
        profile: MemberProfile,
    },
    
    /// Update membership status or profile
    Update {
        cooperative_id: String,
        profile: MemberProfile,
    },
    
    /// Leave a cooperative
    Leave {
        cooperative_id: String,
        reason: Option<String>,
    },
}

/// Actions for building relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationshipAction {
    /// Initiate a new relationship
    Initiate {
        /// Target DID to build relationship with
        target: DID,
        /// Type of relationship proposed
        relationship_type: RelationshipType,
        /// Proposed terms of the relationship
        terms: RelationshipTerms,
    },
    
    /// Accept a relationship proposal
    Accept {
        target: DID,
        relationship_type: RelationshipType,
    },
    
    /// Update an existing relationship
    Update {
        target: DID,
        terms: RelationshipTerms,
    },
    
    /// End an existing relationship
    End {
        target: DID,
        reason: Option<String>,
    },
}

/// Actions for resource sharing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceAction {
    /// Offer a resource to the network
    Offer {
        /// Resource metadata and details
        resource: ResourceOffer,
        /// Optional targeting to specific DIDs/coops
        recipients: Option<Vec<String>>,
        /// Time period of availability
        available_until: Option<u64>,
    },
    
    /// Request access to a resource
    Request {
        resource_id: String,
        purpose: String,
        duration: Option<u64>,
    },
    
    /// Grant resource access
    Grant {
        resource_id: String,
        recipient: DID,
        terms: ResourceTerms,
    },
    
    /// Report resource usage/sharing
    Report {
        resource_id: String,
        usage_metrics: ResourceMetrics,
    },
}

/// Cooperative governance actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GovernanceAction {
    /// Submit a governance proposal
    Propose {
        title: String,
        description: String,
        action_data: Vec<u8>,
        voting_period: u64,
    },
    
    /// Cast a vote on a proposal
    Vote {
        proposal_id: String,
        approve: bool,
        justification: Option<String>,
    },
    
    /// Execute an approved proposal
    Execute {
        proposal_id: String,
        execution_context: Vec<u8>,
    },
}

/// Actions for tracking contributions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContributionAction {
    /// Record a contribution to the network
    Record {
        contribution_type: String,
        description: String,
        evidence: Option<Vec<u8>>,
        beneficiaries: Vec<String>,
    },
    
    /// Validate someone else's contribution
    Validate {
        contribution_id: String,
        assessment: ContributionAssessment,
    },
    
    /// Acknowledge value received from contribution
    Acknowledge {
        contribution_id: String,
        impact_statement: String,
    },
}

/// Types of network membership
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemberType {
    /// Full cooperative member
    Full,
    /// Supporting/associate member
    Associate,
    /// Observer status
    Observer,
}

/// Member profile information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberProfile {
    /// Public name/identifier
    pub name: String,
    /// Member capabilities and offerings
    pub capabilities: Vec<String>,
    /// Contact information
    pub contact: Option<ContactInfo>,
    /// Additional profile metadata
    pub metadata: Vec<u8>,
}

/// Types of relationships that can exist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationshipType {
    /// Direct cooperation
    Cooperation,
    /// Resource sharing
    ResourceSharing,
    /// Mutual support
    Support,
    /// Knowledge exchange
    Knowledge,
}

/// Terms defining a relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipTerms {
    /// Relationship purpose
    pub purpose: String,
    /// Expected duration
    pub duration: Option<u64>,
    /// Mutual commitments
    pub commitments: Vec<String>,
    /// Additional terms
    pub metadata: Vec<u8>,
}

/// Contact information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactInfo {
    /// Contact methods
    pub methods: Vec<ContactMethod>,
    /// Preferred contact hours (UTC)
    pub hours: Option<String>,
    /// Contact preferences
    pub preferences: Vec<String>,
}

/// Contact method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactMethod {
    /// Type of contact (email, matrix, etc)
    pub method_type: String,
    /// Contact value
    pub value: String,
    /// If this is preferred method
    pub preferred: bool,
}

/// Resource being offered
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceOffer {
    /// Resource identifier
    pub id: String,
    /// Resource type/category 
    pub resource_type: String,
    /// Resource details
    pub details: String,
    /// Availability constraints
    pub constraints: Vec<String>,
    /// Required capabilities
    pub required_capabilities: Vec<String>,
}

/// Resource usage terms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceTerms {
    /// Usage period
    pub duration: Option<u64>,
    /// Usage constraints
    pub constraints: Vec<String>,
    /// Reporting requirements
    pub reporting: Vec<String>,
}

/// Resource usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    /// Time period covered
    pub period: (u64, u64),
    /// Usage measurements
    pub measurements: Vec<Measurement>,
    /// Usage patterns
    pub patterns: Vec<String>,
}

/// Resource measurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Measurement {
    /// Metric name
    pub name: String,
    /// Metric value
    pub value: f64,
    /// Metric unit
    pub unit: String,
}

/// Contribution assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributionAssessment {
    /// Assessment score (0-100)
    pub score: u8,
    /// Qualitative assessment
    pub assessment: String,
    /// Areas for improvement
    pub improvements: Vec<String>,
}

impl Transaction {
    /// Creates a new transaction
    /// 
    /// # Arguments
    /// * `sender` - DID of the transaction sender
    /// * `action` - Cooperative action being performed
    /// * `sender_reputation` - Current reputation score of sender
    pub fn new(
        sender: DID,
        action: CooperativeAction,
        sender_reputation: i64,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        let header = TransactionHeader {
            sequence: 0, // Set by sender
            timestamp,
            expires_at: None,
            version: 1,
        };
        
        Self {
            header,
            action,
            sender,
            relationship_proof: None,
            signature: None,
            sender_reputation,
        }
    }

    /// Calculates the cryptographic hash of this transaction
    pub fn hash(&self) -> Hash {
        let mut hasher = blake3::Hasher::new();
        
        // Add header fields
        hasher.update(&self.header.sequence.to_le_bytes());
        hasher.update(&self.header.timestamp.to_le_bytes());
        if let Some(expires) = self.header.expires_at {
            hasher.update(&expires.to_le_bytes());
        }
        hasher.update(&self.header.version.to_le_bytes());
        
        // Add action data
        let action_bytes = bincode::serialize(&self.action)
            .expect("Failed to serialize action");
        hasher.update(&action_bytes);
        
        // Add sender and reputation
        hasher.update(self.sender.as_bytes());
        hasher.update(&self.sender_reputation.to_le_bytes());
        
        // Add relationship proof if present
        if let Some(proof) = &self.relationship_proof {
            let proof_bytes = bincode::serialize(proof)
                .expect("Failed to serialize proof");
            hasher.update(&proof_bytes);
        }
        
        hasher.finalize()
    }

    /// Signs the transaction with the provided key pair
    pub fn sign(&mut self, key_pair: &crate::identity::KeyPair) -> Result<()> {
        let message = self.hash();
        self.signature = Some(key_pair.sign(message.as_bytes())?);
        Ok(())
    }

    /// Verifies the transaction signature
    pub fn verify_signature(&self, public_key: &crate::identity::PublicKey) -> Result<bool> {
        let signature = self.signature.as_ref()
            .ok_or_else(|| Error::Validation("Transaction not signed".into()))?;
            
        let message = self.hash();
        Ok(public_key.verify(message.as_bytes(), signature)?)
    }

    /// Checks if a relationship proof is required for this action
    pub fn requires_relationship_proof(&self) -> bool {
        matches!(
            self.action,
            CooperativeAction::Resource(_) |
            CooperativeAction::Relationship(RelationshipAction::Update { .. }) |
            CooperativeAction::Relationship(RelationshipAction::End { .. })
        )
    }

    /// Gets the encoded size in bytes
    pub fn encoded_size(&self) -> usize {
        bincode::serialize(self)
            .expect("Failed to serialize transaction")
            .len()
    }
}

impl Validate for Transaction {
    fn validate(&self) -> Result<()> {
        // Validate version
        if self.header.version == 0 {
            return Err(Error::Validation("Invalid transaction version".into()));
        }

        // Validate timestamp
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        if self.header.timestamp > now + 60 {
            return Err(Error::Validation("Transaction timestamp too far in future".into()));
        }

        if self.header.timestamp == 0 {
            return Err(Error::Validation("Transaction timestamp cannot be zero".into()));
        }

        // Check expiration
        if let Some(expires) = self.header.expires_at {
            if expires <= now {
                return Err(Error::Validation("Transaction expired".into()));
            }
        }

        // Validate sender
        if self.sender.is_empty() {
            return Err(Error::Validation("Invalid sender DID".into()));
        }

        // Validate relationship proof requirement
        if self.requires_relationship_proof() && self.relationship_proof.is_none() {
            return Err(Error::Validation(
                "Relationship proof required for this action type".into()
            ));
        }

        // Action-specific validation
        match &self.action {
            CooperativeAction::Membership(action) => {
                self.validate_membership_action(action)?;
            }
            CooperativeAction::Relationship(action) => {
                self.validate_relationship_action(action)?;
            }
            CooperativeAction::Resource(action) => {
                self.validate_resource_action(action)?;
            }
            CooperativeAction::Governance(action) => {
                self.validate_governance_action(action)?;
            }
            CooperativeAction::Contribution(action) => {
                self.validate_contribution_action(action)?;
            }
        }

        Ok(())
    }
}

// Action-specific validation implementations
impl Transaction {
    fn validate_membership_action(&self, action: &MembershipAction) -> Result<()> {
        match action {
            MembershipAction::Join { cooperative_id, profile, .. } => {
                if cooperative_id.is_empty() {
                    return Err(Error::Validation("Cooperative ID cannot be empty".into()));
                }
                if profile.name.is_empty() {
                    return Err(Error::Validation("Member profile name cannot be empty".into()));
                }
            }
            MembershipAction::Update { cooperative_id, profile } => {
                if cooperative_id.is_empty() {
                    return Err(Error::Validation("Cooperative ID cannot be empty".into()));
                }
                if profile.name.is_empty() {
                    return Err(Error::Validation("Member profile name cannot be empty".into()));
                }
            }
            MembershipAction::Leave { cooperative_id, .. } => {
                if cooperative_id.is_empty() {
                    return Err(Error::Validation("Cooperative ID cannot be empty".into()));
                }
            }
        }
        Ok(())
    }

    fn validate_relationship_action(&self, action: &RelationshipAction) -> Result<()> {
        match action {
            RelationshipAction::Initiate { target, terms, .. } => {
                if target.is_empty() {
                    return Err(Error::Validation("Target DID cannot be empty".into()));
                }
                if terms.purpose.is_empty() {
                    return Err(Error::Validation("Relationship purpose cannot be empty".into()));
                }
                if terms.commitments.is_empty() {
                    return Err(Error::Validation("Relationship must have at least one commitment".into()));
                }
            }
            RelationshipAction::Accept { target, .. } => {
                if target.is_empty() {
                    return Err(Error::Validation("Target DID cannot be empty".into()));
                }
            }
            RelationshipAction::Update { target, terms } => {
                if target.is_empty() {
                    return Err(Error::Validation("Target DID cannot be empty".into()));
                }
                if terms.purpose.is_empty() {
                    return Err(Error::Validation("Relationship purpose cannot be empty".into()));
                }
            }
            RelationshipAction::End { target, .. } => {
                if target.is_empty() {
                    return Err(Error::Validation("Target DID cannot be empty".into()));
                }
            }
        }
        Ok(())
    }

    fn validate_resource_action(&self, action: &ResourceAction) -> Result<()> {
        match action {
            ResourceAction::Offer { resource, .. } => {
                if resource.id.is_empty() {
                    return Err(Error::Validation("Resource ID cannot be empty".into()));
                }
                if resource.resource_type.is_empty() {
                    return Err(Error::Validation("Resource type cannot be empty".into()));
                }
                if resource.details.is_empty() {
                    return Err(Error::Validation("Resource details cannot be empty".into()));
                }
            }
            ResourceAction::Request { resource_id, purpose, .. } => {
                if resource_id.is_empty() {
                    return Err(Error::Validation("Resource ID cannot be empty".into()));
                }
                if purpose.is_empty() {
                    return Err(Error::Validation("Request purpose cannot be empty".into()));
                }
            }
            ResourceAction::Grant { resource_id, recipient, .. } => {
                if resource_id.is_empty() {
                    return Err(Error::Validation("Resource ID cannot be empty".into()));
                }
                if recipient.is_empty() {
                    return Err(Error::Validation("Recipient DID cannot be empty".into()));
                }
            }
            ResourceAction::Report { resource_id, usage_metrics } => {
                if resource_id.is_empty() {
                    return Err(Error::Validation("Resource ID cannot be empty".into()));
                }
                if usage_metrics.measurements.is_empty() {
                    return Err(Error::Validation("Usage report must include measurements".into()));
                }
                // Validate measurement period
                let (start, end) = usage_metrics.period;
                if end <= start {
                    return Err(Error::Validation("Invalid measurement period".into()));
                }
            }
        }
        Ok(())
    }

    fn validate_governance_action(&self, action: &GovernanceAction) -> Result<()> {
        match action {
            GovernanceAction::Propose { title, description, voting_period, .. } => {
                if title.is_empty() {
                    return Err(Error::Validation("Proposal title cannot be empty".into()));
                }
                if description.is_empty() {
                    return Err(Error::Validation("Proposal description cannot be empty".into()));
                }
                if *voting_period == 0 {
                    return Err(Error::Validation("Voting period cannot be zero".into()));
                }
            }
            GovernanceAction::Vote { proposal_id, .. } => {
                if proposal_id.is_empty() {
                    return Err(Error::Validation("Proposal ID cannot be empty".into()));
                }
            }
            GovernanceAction::Execute { proposal_id, .. } => {
                if proposal_id.is_empty() {
                    return Err(Error::Validation("Proposal ID cannot be empty".into()));
                }
            }
        }
        Ok(())
    }

    fn validate_contribution_action(&self, action: &ContributionAction) -> Result<()> {
        match action {
            ContributionAction::Record { contribution_type, description, beneficiaries, .. } => {
                if contribution_type.is_empty() {
                    return Err(Error::Validation("Contribution type cannot be empty".into()));
                }
                if description.is_empty() {
                    return Err(Error::Validation("Contribution description cannot be empty".into()));
                }
                if beneficiaries.is_empty() {
                    return Err(Error::Validation("Must specify at least one beneficiary".into()));
                }
                for beneficiary in beneficiaries {
                    if beneficiary.is_empty() {
                        return Err(Error::Validation("Invalid beneficiary identifier".into()));
                    }
                }
            }
            ContributionAction::Validate { contribution_id, assessment } => {
                if contribution_id.is_empty() {
                    return Err(Error::Validation("Contribution ID cannot be empty".into()));
                }
                if assessment.assessment.is_empty() {
                    return Err(Error::Validation("Assessment cannot be empty".into()));
                }
                if assessment.score > 100 {
                    return Err(Error::Validation("Assessment score must be between 0 and 100".into()));
                }
            }
            ContributionAction::Acknowledge { contribution_id, impact_statement } => {
                if contribution_id.is_empty() {
                    return Err(Error::Validation("Contribution ID cannot be empty".into()));
                }
                if impact_statement.is_empty() {
                    return Err(Error::Validation("Impact statement cannot be empty".into()));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
use super::*;
use crate::identity::KeyPair;

fn create_test_transaction() -> Transaction {
let sender = DID::from("did:icn:test");
let action = CooperativeAction::Membership(
MembershipAction::Join {
    cooperative_id: "test-coop".into(),
    member_type: MemberType::Full,
    profile: MemberProfile {
        name: "Test Member".into(),
        capabilities: vec!["capability1".into()],
        contact: None,
        metadata: vec![],
    },
}
);
Transaction::new(sender, action, 100)
}

#[test]
fn test_transaction_creation() {
let tx = create_test_transaction();
assert_eq!(tx.sender_reputation, 100);
assert!(tx.signature.is_none());
assert!(tx.header.expires_at.is_none());
assert_eq!(tx.header.version, 1);
}

#[test]
fn test_transaction_hash() {
let tx = create_test_transaction();
let hash1 = tx.hash();

// Same transaction should produce same hash
assert_eq!(tx.hash(), hash1);

// Different transactions should have different hashes
let mut tx2 = create_test_transaction();
tx2.header.sequence = 1;
assert_ne!(tx2.hash(), hash1);
}

#[test]
fn test_transaction_signing() {
let key_pair = KeyPair::generate();
let mut tx = create_test_transaction();

assert!(tx.sign(&key_pair).is_ok());
assert!(tx.signature.is_some());
assert!(tx.verify_signature(&key_pair.public_key()).unwrap());

// Wrong key should fail verification
let wrong_key = KeyPair::generate().public_key();
assert!(!tx.verify_signature(&wrong_key).unwrap());
}

#[test]
fn test_membership_validation() {
let mut tx = create_test_transaction();
assert!(tx.validate().is_ok());

// Test invalid cases
if let CooperativeAction::Membership(MembershipAction::Join { cooperative_id, profile, .. }) = &mut tx.action {
cooperative_id.clear();
assert!(tx.validate().is_err());

*cooperative_id = "test-coop".into();
profile.name.clear();
assert!(tx.validate().is_err());
}
}

#[test]
fn test_relationship_validation() {
let sender = DID::from("did:icn:test");
let action = CooperativeAction::Relationship(
RelationshipAction::Initiate {
    target: DID::from("did:icn:target"),
    relationship_type: RelationshipType::Cooperation,
    terms: RelationshipTerms {
        purpose: "Test purpose".into(),
        duration: None,
        commitments: vec!["commitment1".into()],
        metadata: vec![],
    },
}
);
let mut tx = Transaction::new(sender, action, 100);
assert!(tx.validate().is_ok());

// Test invalid cases
if let CooperativeAction::Relationship(RelationshipAction::Initiate { target, terms, .. }) = &mut tx.action {
target.clear();
assert!(tx.validate().is_err());

*target = DID::from("did:icn:target");
terms.purpose.clear();
assert!(tx.validate().is_err());

terms.purpose = "Test purpose".into();
terms.commitments.clear();
assert!(tx.validate().is_err());
}
}

#[test]
fn test_resource_validation() {
let sender = DID::from("did:icn:test");
let action = CooperativeAction::Resource(
ResourceAction::Offer {
    resource: ResourceOffer {
        id: "resource1".into(),
        resource_type: "test-type".into(),
        details: "Test resource".into(),
        constraints: vec![],
        required_capabilities: vec![],
    },
    recipients: None,
    available_until: None,
}
);
let mut tx = Transaction::new(sender, action, 100);
tx.relationship_proof = Some(RelationshipProof::default());
assert!(tx.validate().is_ok());

// Test invalid cases
if let CooperativeAction::Resource(ResourceAction::Offer { resource, .. }) = &mut tx.action {
resource.id.clear();
assert!(tx.validate().is_err());

resource.id = "resource1".into();
resource.resource_type.clear();
assert!(tx.validate().is_err());

resource.resource_type = "test-type".into();
resource.details.clear();
assert!(tx.validate().is_err());
}
}

#[test]
fn test_governance_validation() {
let sender = DID::from("did:icn:test");
let action = CooperativeAction::Governance(
GovernanceAction::Propose {
    title: "Test proposal".into(),
    description: "Test description".into(),
    action_data: vec![],
    voting_period: 1000,
}
);
let mut tx = Transaction::new(sender, action, 100);
assert!(tx.validate().is_ok());

// Test invalid cases
if let CooperativeAction::Governance(GovernanceAction::Propose { title, description, voting_period, .. }) = &mut tx.action {
title.clear();
assert!(tx.validate().is_err());

*title = "Test proposal".into();
description.clear();
assert!(tx.validate().is_err());

*description = "Test description".into();
*voting_period = 0;
assert!(tx.validate().is_err());
}
}

#[test]
fn test_contribution_validation() {
let sender = DID::from("did:icn:test");
let action = CooperativeAction::Contribution(
ContributionAction::Record {
    contribution_type: "test-type".into(),
    description: "Test contribution".into(),
    evidence: None,
    beneficiaries: vec!["beneficiary1".into()],
}
);
let mut tx = Transaction::new(sender, action, 100);
assert!(tx.validate().is_ok());

// Test invalid cases
if let CooperativeAction::Contribution(ContributionAction::Record { contribution_type, description, beneficiaries, .. }) = &mut tx.action {
contribution_type.clear();
assert!(tx.validate().is_err());

*contribution_type = "test-type".into();
description.clear();
assert!(tx.validate().is_err());

*description = "Test contribution".into();
beneficiaries.clear();
assert!(tx.validate().is_err());
}
}
}