use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// Records interactions within relationships.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interaction {
    /// When the interaction occurred
    pub date: DateTime<Utc>,
    
    /// Description of what happened
    pub description: String,
    
    /// Impact or outcome of the interaction
    pub impact: Option<String>,
    
    /// Type of interaction
    pub interaction_type: InteractionType,
}

/// Types of interactions between members.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InteractionType {
    /// Working together
    Collaboration,
    
    /// Providing support
    Support,
    
    /// Exchanging resources
    ResourceExchange,
    
    /// Sharing knowledge
    KnowledgeSharing,
    
    /// Working through challenges
    ConflictResolution,
    
    /// Other types of interactions
    Other(String),
}