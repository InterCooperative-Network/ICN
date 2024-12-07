use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RelationshipType {
    Cooperation,
    MutualAid,
    ResourceSharing,
    Endorsement,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub id: uuid::Uuid,
    pub relationship_type: RelationshipType,
    pub party_a: String,
    pub party_b: String,
    pub metadata: HashMap<String, String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Relationship {
    pub fn new(
        relationship_type: RelationshipType,
        party_a: String,
        party_b: String,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: uuid::Uuid::new_v4(),
            relationship_type,
            party_a,
            party_b,
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }
}
