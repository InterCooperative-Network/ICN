use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// Endorsements within relationships.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endorsement {
    /// DID of the endorsing member
    pub from_did: String,
    
    /// Content of the endorsement
    pub content: String,
    
    /// When the endorsement was given
    pub date: DateTime<Utc>,
    
    /// Context in which the skills were demonstrated
    pub context: String,
    
    /// Specific skills being endorsed
    pub skills: Vec<String>,
}
