use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Proposal {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub created_by: String,
    pub ends_at: chrono::NaiveDateTime,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Vote {
    pub proposal_id: i64,
    pub voter: String,
    pub approve: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Contribution {
    pub id: i64,
    pub did: String,
    pub score: i64,
    pub timestamp: f64,
    pub zk_snark_proof: Option<String>, // Added zk-SNARK proof field
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Federation {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Resource {
    pub id: i64,
    pub name: String,
    pub resource_type: String,
    pub owner: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_proposal_serialization() {
        let proposal = Proposal {
            id: 1,
            title: "Test Proposal".to_string(),
            description: "This is a test proposal".to_string(),
            created_by: "did:icn:test".to_string(),
            ends_at: chrono::NaiveDateTime::from_timestamp(1_614_000_000, 0),
            created_at: chrono::NaiveDateTime::from_timestamp(1_614_000_000, 0),
        };

        let serialized = serde_json::to_string(&proposal).unwrap();
        let deserialized: Proposal = serde_json::from_str(&serialized).unwrap();

        assert_eq!(proposal, deserialized);
    }

    #[test]
    fn test_vote_serialization() {
        let vote = Vote {
            proposal_id: 1,
            voter: "did:icn:test".to_string(),
            approve: true,
        };

        let serialized = serde_json::to_string(&vote).unwrap();
        let deserialized: Vote = serde_json::from_str(&serialized).unwrap();

        assert_eq!(vote, deserialized);
    }

    #[test]
    fn test_contribution_serialization() {
        let contribution = Contribution {
            id: 1,
            did: "did:icn:test".to_string(),
            score: 100,
            timestamp: 1_614_000_000.0,
            zk_snark_proof: Some("proof".to_string()),
        };

        let serialized = serde_json::to_string(&contribution).unwrap();
        let deserialized: Contribution = serde_json::from_str(&serialized).unwrap();

        assert_eq!(contribution, deserialized);
    }

    #[test]
    fn test_federation_serialization() {
        let federation = Federation {
            id: 1,
            name: "Test Federation".to_string(),
            description: "This is a test federation".to_string(),
            created_at: chrono::NaiveDateTime::from_timestamp(1_614_000_000, 0),
        };

        let serialized = serde_json::to_string(&federation).unwrap();
        let deserialized: Federation = serde_json::from_str(&serialized).unwrap();

        assert_eq!(federation, deserialized);
    }

    #[test]
    fn test_resource_serialization() {
        let resource = Resource {
            id: 1,
            name: "Test Resource".to_string(),
            resource_type: "cpu".to_string(),
            owner: "did:icn:test".to_string(),
            created_at: chrono::NaiveDateTime::from_timestamp(1_614_000_000, 0),
            updated_at: chrono::NaiveDateTime::from_timestamp(1_614_000_000, 0),
        };

        let serialized = serde_json::to_string(&resource).unwrap();
        let deserialized: Resource = serde_json::from_str(&serialized).unwrap();

        assert_eq!(resource, deserialized);
    }
}
