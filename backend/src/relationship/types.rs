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
    pub fn from_str(relationship_type: &str) -> Self {
        match relationship_type.to_lowercase().as_str() {
            "collaboration" => RelationshipType::Collaboration,
            "mentorship" => RelationshipType::Mentorship,
            "resource_sharing" => RelationshipType::ResourceSharing,
            "mutual_aid" => RelationshipType::MutualAid,
            "project_partnership" => RelationshipType::ProjectPartnership,
            other => RelationshipType::Custom(other.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            RelationshipType::Collaboration => "collaboration",
            RelationshipType::Mentorship => "mentorship",
            RelationshipType::ResourceSharing => "resource_sharing",
            RelationshipType::MutualAid => "mutual_aid",
            RelationshipType::ProjectPartnership => "project_partnership",
            RelationshipType::Custom(s) => s,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relationship_type_conversion() {
        assert!(matches!(
            RelationshipType::from_str("collaboration"),
            RelationshipType::Collaboration
        ));
        assert!(matches!(
            RelationshipType::from_str("custom_type"),
            RelationshipType::Custom(s) if s == "custom_type"
        ));
    }

    #[test]
    fn test_relationship_type_as_str() {
        assert_eq!(RelationshipType::Collaboration.as_str(), "collaboration");
        assert_eq!(
            RelationshipType::Custom("test".to_string()).as_str(),
            "test"
        );
    }
}
