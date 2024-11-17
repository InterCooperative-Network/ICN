// src/relationship/types.rs

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RelationshipType {
    Collaboration,
    Mentorship,
    ResourceSharing,
    MutualAid,
    ProjectPartnership,
    Custom(String),
}

impl RelationshipType {
    pub fn new(relationship_type: &str) -> Self {
        match relationship_type.to_lowercase().as_str() {
            "collaboration" => RelationshipType::Collaboration,
            "mentorship" => RelationshipType::Mentorship,
            "resource_sharing" => RelationshipType::ResourceSharing,
            "mutual_aid" => RelationshipType::MutualAid,
            "project_partnership" => RelationshipType::ProjectPartnership,
            other => RelationshipType::Custom(other.to_string()),
        }
    }

    pub fn as_str(&self) -> String {
        match self {
            RelationshipType::Collaboration => "collaboration".to_string(),
            RelationshipType::Mentorship => "mentorship".to_string(),
            RelationshipType::ResourceSharing => "resource_sharing".to_string(),
            RelationshipType::MutualAid => "mutual_aid".to_string(),
            RelationshipType::ProjectPartnership => "project_partnership".to_string(),
            RelationshipType::Custom(s) => s.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relationship_type_creation() {
        assert!(matches!(RelationshipType::new("collaboration"), RelationshipType::Collaboration));
        assert!(matches!(RelationshipType::new("mentorship"), RelationshipType::Mentorship));
        assert!(matches!(
            RelationshipType::new("custom_type"),
            RelationshipType::Custom(s) if s == "custom_type"
        ));
    }

    #[test]
    fn test_relationship_type_string_conversion() {
        let r_type = RelationshipType::Collaboration;
        assert_eq!(r_type.as_str(), "collaboration");

        let custom = RelationshipType::Custom("test".to_string());
        assert_eq!(custom.as_str(), "test");
    }
}