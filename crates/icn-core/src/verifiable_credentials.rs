use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiableCredential {
    pub credential_type: String,
    pub issuer_did: String,
    pub subject_did: String,
    pub issuance_date: String,
    pub expiration_date: Option<String>,
    pub credential_status: Option<CredentialStatus>,
    pub credential_schema: Option<String>,
    pub proof: Proof,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialStatus {
    pub id: String,
    pub type_: String,
    pub status: String,
    pub status_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proof {
    pub type_: String,
    pub created: String,
    pub proof_purpose: String,
    pub verification_method: String,
    pub jws: String,
}

#[derive(Debug)]
pub enum VerificationError {
    InvalidSignature,
    ExpiredCredential,
    RevokedCredential,
    MalformedCredential,
    UnknownIssuer,
}

impl VerifiableCredential {
    pub fn new(
        credential_type: String,
        issuer_did: String,
        subject_did: String,
        proof: Proof,
    ) -> Self {
        Self {
            credential_type,
            issuer_did,
            subject_did,
            issuance_date: Utc::now().to_rfc3339(),
            expiration_date: None,
            credential_status: None,
            credential_schema: None,
            proof,
        }
    }
    
    pub fn is_valid(&self) -> bool {
        // Check if credential is expired
        if let Some(expiry_date) = &self.expiration_date {
            if let Ok(expiry) = DateTime::parse_from_rfc3339(expiry_date) {
                if Utc::now() > expiry.into() {
                    return false;
                }
            } else {
                return false; // Could not parse expiry date
            }
        }
        
        // Check if credential is revoked
        if let Some(status) = &self.credential_status {
            if status.status == "revoked" {
                return false;
            }
        }
        
        // In a real implementation, we would also verify the signature
        // using the issuer's public key
        
        true
    }
    
    pub fn verify(&self, _trusted_issuers: &[String]) -> Result<bool, VerificationError> {
        if !self.is_valid() {
            if let Some(expiry_date) = &self.expiration_date {
                if let Ok(expiry) = DateTime::parse_from_rfc3339(expiry_date) {
                    if Utc::now() > expiry.into() {
                        return Err(VerificationError::ExpiredCredential);
                    }
                }
            }
            
            if let Some(status) = &self.credential_status {
                if status.status == "revoked" {
                    return Err(VerificationError::RevokedCredential);
                }
            }
            
            return Err(VerificationError::MalformedCredential);
        }
        
        // In a real implementation, this would verify the signature using the 
        // issuer's public key from a trusted source
        
        Ok(true)
    }
}