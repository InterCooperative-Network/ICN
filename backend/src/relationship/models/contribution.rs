use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// Records a concrete contribution made to the cooperative community.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contribution {
    /// DID of the contributing member
    pub contributor_did: String,
    
    /// Brief description of the contribution
    pub description: String,
    
    /// Detailed story about the contribution's impact on the community
    pub impact_story: String,
    
    /// When the contribution occurred
    pub date: DateTime<Utc>,
    
    /// Context or category of the contribution
    pub context: String,
    
    /// DIDs of members who witnessed or verified the contribution
    pub witnesses: Vec<String>,
    
    /// Feedback and endorsements from other members
    pub feedback: Vec<Feedback>,
    
    /// Tags for categorizing and finding related contributions
    pub tags: Vec<String>,
}

/// Feedback provided on a contribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feedback {
    /// DID of the member providing feedback
    pub from_did: String,
    
    /// The actual feedback content
    pub content: String,
    
    /// When the feedback was given
    pub date: DateTime<Utc>,
    
    /// Type of endorsement this feedback represents
    pub endorsement_type: EndorsementType,
}

/// Types of endorsements for contributions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EndorsementType {
    /// Confirms the contribution occurred as described
    Verification,
    
    /// Speaks to the contribution's effect on the community
    Impact,
    
    /// Endorses the contributor's character and reliability
    Character,
    
    /// Validates specific skills demonstrated
    Skill,
}
