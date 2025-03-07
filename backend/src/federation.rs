use std::collections::HashMap;
use crate::governance::{Federation, FederationType, FederationTerms};

pub struct FederationManager {
    federations: HashMap<String, Federation>,
}

impl FederationManager {
    pub fn new() -> Self {
        Self {
            federations: HashMap::new(),
        }
    }
    
    pub fn create_federation(&mut self, id: String, federation_type: FederationType, terms: FederationTerms, admin: String) -> Result<(), String> {
        if self.federations.contains_key(&id) {
            return Err("Federation already exists".to_string());
        }
        
        let federation = Federation::new(id.clone(), federation_type, terms, admin);
        self.federations.insert(id, federation);
        
        Ok(())
    }
    
    pub fn get_federation(&self, id: &str) -> Option<&Federation> {
        self.federations.get(id)
    }
    
    pub fn get_federations_by_member(&self, member_did: &str) -> Vec<&Federation> {
        self.federations.values()
            .filter(|f| f.members.contains_key(member_did))
            .collect()
    }
    
    pub fn add_member_to_federation(&mut self, federation_id: &str, member_did: String, role: crate::governance::MemberRole) -> Result<(), String> {
        let federation = self.federations.get_mut(federation_id)
            .ok_or("Federation not found")?;
            
        federation.add_member(member_did, role)
    }
    
    pub async fn detect_conflicts(&self, federation_id: &str) -> Result<Vec<(String, String)>, String> {
        let federation = self.federations.get(federation_id)
            .ok_or("Federation not found")?;
            
        Ok(federation.detect_resource_conflicts().await)
    }
    
    pub async fn resolve_conflicts(&mut self, federation_id: &str, conflicts: Vec<(String, String)>) -> Result<(), String> {
        let federation = self.federations.get(federation_id)
            .ok_or("Federation not found")?;
            
        federation.resolve_conflicts(conflicts).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_federation() {
        let mut manager = FederationManager::new();
        
        let result = manager.create_federation(
            "fed1".to_string(),
            FederationType::Cooperative,
            FederationTerms::default(),
            "admin1".to_string()
        );
        
        assert!(result.is_ok());
        assert!(manager.get_federation("fed1").is_some());
    }
    
    #[test]
    fn test_add_member_to_federation() {
        let mut manager = FederationManager::new();
        
        manager.create_federation(
            "fed1".to_string(),
            FederationType::Cooperative,
            FederationTerms::default(),
            "admin1".to_string()
        ).unwrap();
        
        let result = manager.add_member_to_federation(
            "fed1",
            "member1".to_string(),
            crate::governance::MemberRole::Member
        );
        
        assert!(result.is_ok());
        
        let federation = manager.get_federation("fed1").unwrap();
        assert!(federation.members.contains_key("member1"));
    }
}