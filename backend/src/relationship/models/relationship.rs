use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use super::{Interaction, Endorsement};
use crate::relationship::types::RelationshipType;

/// Represents a relationship between members.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    /// DID of the first member
    pub member_one: String,
    
    /// DID of the second member
    pub member_two: String,
    
    /// Type of relationship
    pub relationship_type: RelationshipType,
    
    /// When the relationship began
    pub started: DateTime<Utc>,
    
    /// Story of how the relationship formed and evolved
    pub story: String,
    
    /// History of interactions between members
    pub interactions: Vec<Interaction>,
    
    /// Endorsements members have given each other
    pub mutual_endorsements: Vec<Endorsement>,
    
    /// Notes about the relationship
    pub notes: Vec<RelationshipNote>,
}

/// Notes associated with a relationship.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipNote {
    /// DID of the note author
    pub author_did: String,
    
    /// Content of the note
    pub content: String,
    
    /// When the note was written
    pub date: DateTime<Utc>,
    
    /// Who can see this note
    pub visibility: Visibility,
}

/// Visibility levels for notes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Visibility {
    /// Visible to everyone
    Public,
    
    /// Only visible to those in the relationship
    RelationshipParticipants,
    
    /// Visible to all cooperative members
    CooperativeMembers,
    
    /// Only visible to the author
    Private,
}
