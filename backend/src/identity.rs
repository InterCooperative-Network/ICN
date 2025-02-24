use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct IdentityManager {
    identities: Arc<Mutex<HashMap<String, String>>>,
    local_clusters: Arc<Mutex<HashMap<String, Vec<String>>>>,
}

impl IdentityManager {
    pub fn new() -> Self {
        IdentityManager {
            identities: Arc::new(Mutex::new(HashMap::new())),
            local_clusters: Arc::new(Mutex::new(HashMap::new())),
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

    pub async fn create_local_cluster(&self, cluster_name: &str, members: Vec<String>) -> Result<(), String> {
        let mut local_clusters = self.local_clusters.lock().await;
        if local_clusters.contains_key(cluster_name) {
            return Err("Local cluster already exists".to_string());
        }
        local_clusters.insert(cluster_name.to_string(), members);
        Ok(())
    }

    pub async fn get_local_cluster(&self, cluster_name: &str) -> Result<Vec<String>, String> {
        let local_clusters = self.local_clusters.lock().await;
        local_clusters.get(cluster_name).cloned().ok_or_else(|| "Local cluster not found".to_string())
    }

    pub async fn add_member_to_cluster(&self, cluster_name: &str, member: String) -> Result<(), String> {
        let mut local_clusters = self.local_clusters.lock().await;
        if let Some(cluster) = local_clusters.get_mut(cluster_name) {
            if cluster.contains(&member) {
                return Err("Member already in cluster".to_string());
            }
            cluster.push(member);
            Ok(())
        } else {
            Err("Local cluster not found".to_string())
        }
    }

    pub async fn remove_member_from_cluster(&self, cluster_name: &str, member: &str) -> Result<(), String> {
        let mut local_clusters = self.local_clusters.lock().await;
        if let Some(cluster) = local_clusters.get_mut(cluster_name) {
            if let Some(pos) = cluster.iter().position(|x| x == member) {
                cluster.remove(pos);
                Ok(())
            } else {
                Err("Member not found in cluster".to_string())
            }
        } else {
            Err("Local cluster not found".to_string())
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

    #[test]
    fn test_create_local_cluster() {
        let rt = Runtime::new().unwrap();
        let identity_manager = IdentityManager::new();

        rt.block_on(async {
            let result = identity_manager.create_local_cluster("test_cluster", vec!["member1".to_string(), "member2".to_string()]).await;
            assert!(result.is_ok());
        });
    }

    #[test]
    fn test_get_local_cluster() {
        let rt = Runtime::new().unwrap();
        let identity_manager = IdentityManager::new();

        rt.block_on(async {
            identity_manager.create_local_cluster("test_cluster", vec!["member1".to_string(), "member2".to_string()]).await.unwrap();
            let result = identity_manager.get_local_cluster("test_cluster").await;
            assert_eq!(result.unwrap(), vec!["member1".to_string(), "member2".to_string()]);
        });
    }

    #[test]
    fn test_add_member_to_cluster() {
        let rt = Runtime::new().unwrap();
        let identity_manager = IdentityManager::new();

        rt.block_on(async {
            identity_manager.create_local_cluster("test_cluster", vec!["member1".to_string(), "member2".to_string()]).await.unwrap();
            let result = identity_manager.add_member_to_cluster("test_cluster", "member3".to_string()).await;
            assert!(result.is_ok());
            let cluster = identity_manager.get_local_cluster("test_cluster").await.unwrap();
            assert_eq!(cluster, vec!["member1".to_string(), "member2".to_string(), "member3".to_string()]);
        });
    }

    #[test]
    fn test_remove_member_from_cluster() {
        let rt = Runtime::new().unwrap();
        let identity_manager = IdentityManager::new();

        rt.block_on(async {
            identity_manager.create_local_cluster("test_cluster", vec!["member1".to_string(), "member2".to_string()]).await.unwrap();
            let result = identity_manager.remove_member_from_cluster("test_cluster", "member1").await;
            assert!(result.is_ok());
            let cluster = identity_manager.get_local_cluster("test_cluster").await.unwrap();
            assert_eq!(cluster, vec!["member2".to_string()]);
        });
    }
}
