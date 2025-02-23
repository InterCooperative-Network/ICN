use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct IdentityManager {
    identities: Arc<Mutex<HashMap<String, String>>>,
}

impl IdentityManager {
    pub fn new() -> Self {
        IdentityManager {
            identities: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn create_identity(&self, identity: &str) -> Result<(), String> {
        let mut identities = self.identities.lock().await;
        if identities.contains_key(identity) {
            return Err("Identity already exists".to_string());
        }
        identities.insert(identity.to_string(), String::new());
        Ok(())
    }

    pub async fn get_identity(&self, identity: &str) -> Result<String, String> {
        let identities = self.identities.lock().await;
        identities.get(identity).cloned().ok_or_else(|| "Identity not found".to_string())
    }

    pub async fn update_identity(&self, identity: &str, new_data: &str) -> Result<(), String> {
        let mut identities = self.identities.lock().await;
        if let Some(existing_identity) = identities.get_mut(identity) {
            *existing_identity = new_data.to_string();
            Ok(())
        } else {
            Err("Identity not found".to_string())
        }
    }

    pub async fn delete_identity(&self, identity: &str) -> Result<(), String> {
        let mut identities = self.identities.lock().await;
        if identities.remove(identity).is_some() {
            Ok(())
        } else {
            Err("Identity not found".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;

    #[test]
    fn test_create_identity() {
        let rt = Runtime::new().unwrap();
        let identity_manager = IdentityManager::new();

        rt.block_on(async {
            let result = identity_manager.create_identity("test_identity").await;
            assert!(result.is_ok());
        });
    }

    #[test]
    fn test_get_identity() {
        let rt = Runtime::new().unwrap();
        let identity_manager = IdentityManager::new();

        rt.block_on(async {
            identity_manager.create_identity("test_identity").await.unwrap();
            let result = identity_manager.get_identity("test_identity").await;
            assert_eq!(result.unwrap(), "");
        });
    }

    #[test]
    fn test_update_identity() {
        let rt = Runtime::new().unwrap();
        let identity_manager = IdentityManager::new();

        rt.block_on(async {
            identity_manager.create_identity("test_identity").await.unwrap();
            let result = identity_manager.update_identity("test_identity", "new_data").await;
            assert!(result.is_ok());
            let updated_identity = identity_manager.get_identity("test_identity").await.unwrap();
            assert_eq!(updated_identity, "new_data");
        });
    }

    #[test]
    fn test_delete_identity() {
        let rt = Runtime::new().unwrap();
        let identity_manager = IdentityManager::new();

        rt.block_on(async {
            identity_manager.create_identity("test_identity").await.unwrap();
            let result = identity_manager.delete_identity("test_identity").await;
            assert!(result.is_ok());
            let deleted_identity = identity_manager.get_identity("test_identity").await;
            assert!(deleted_identity.is_err());
        });
    }
}
