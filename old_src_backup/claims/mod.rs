use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claim {
    pub id: String,
    pub issuer: String,
    pub subject: String,
    pub claim_type: ClaimType,
    pub value: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub verification_method: String,
    pub proof: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClaimType {
    Skill,
    Reputation,
    Membership,
    Role,
    Contribution,
    Verification,
    Custom(String),
}

impl Claim {
    pub fn new(
        issuer: String,
        subject: String,
        claim_type: ClaimType,
        value: String,
        verification_method: String,
    ) -> Self {
        Claim {
            id: uuid::Uuid::new_v4().to_string(),
            issuer,
            subject,
            claim_type,
            value,
            issued_at: Utc::now(),
            expires_at: None,
            verification_method,
            proof: None,
        }
    }

    pub fn verify(&self) -> bool {
        // Basic validation
        if self.issuer.is_empty() || self.subject.is_empty() {
            return false;
        }

        // Check expiration
        if let Some(expires_at) = self.expires_at {
            if Utc::now() > expires_at {
                return false;
            }
        }

        // Check proof if present
        if let Some(proof) = &self.proof {
            if proof.is_empty() {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claim_creation() {
        let claim = Claim::new(
            "issuer".to_string(),
            "subject".to_string(),
            ClaimType::Skill,
            "programming".to_string(),
            "manual".to_string(),
        );

        assert!(!claim.id.is_empty());
        assert_eq!(claim.issuer, "issuer");
        assert_eq!(claim.subject, "subject");
        assert!(matches!(claim.claim_type, ClaimType::Skill));
        assert!(claim.verify());
    }

    #[test]
    fn test_claim_verification() {
        let mut claim = Claim::new(
            "issuer".to_string(),
            "subject".to_string(),
            ClaimType::Skill,
            "programming".to_string(),
            "manual".to_string(),
        );

        assert!(claim.verify());

        claim.issuer = "".to_string();
        assert!(!claim.verify());
    }
}