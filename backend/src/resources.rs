use std::collections::HashMap;

pub struct ResourceManager {
    resources: HashMap<String, Resource>,
}

pub struct Resource {
    pub id: String,
    pub type_: String,
    pub capacity: u64,
    pub available: u64,
    pub owner_did: String,
}

impl ResourceManager {
    pub fn new() -> Self {
        ResourceManager {
            resources: HashMap::new(),
        }
    }
    
    pub fn register_resource(&mut self, resource: Resource) -> Result<(), String> {
        if self.resources.contains_key(&resource.id) {
            return Err("Resource already exists".to_string());
        }
        
        self.resources.insert(resource.id.clone(), resource);
        Ok(())
    }
    
    pub fn get_resource(&self, id: &str) -> Option<&Resource> {
        self.resources.get(id)
    }
    
    pub fn list_available_resources(&self) -> Vec<&Resource> {
        self.resources.values()
            .filter(|r| r.available > 0)
            .collect()
    }
    
    pub fn request_resource(&mut self, id: &str, amount: u64) -> Result<(), String> {
        let resource = self.resources.get_mut(id).ok_or("Resource not found")?;
        
        if resource.available < amount {
            return Err("Insufficient resources available".to_string());
        }
        
        resource.available -= amount;
        Ok(())
    }
    
    pub fn release_resource(&mut self, id: &str, amount: u64) -> Result<(), String> {
        let resource = self.resources.get_mut(id).ok_or("Resource not found")?;
        
        if resource.available + amount > resource.capacity {
            return Err("Cannot release more resources than capacity".to_string());
        }
        
        resource.available += amount;
        Ok(())
    }
}