use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// Records mutual aid interactions between members.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutualAidInteraction {
    /// When the interaction occurred
    pub date: DateTime<Utc>,
    
    /// DID of the member providing aid
    pub provider_did: String,
    
    /// DID of the member receiving aid
    pub receiver_did: String,
    
    /// Description of what was shared or exchanged
    pub description: String,
    
    /// Story about how this aid impacted the community
    pub impact_story: Option<String>,
    
    /// Notes about reciprocity and relationship building
    pub reciprocity_notes: Option<String>,
    
    /// Tags for categorizing types of mutual aid
    pub tags: Vec<String>,
}