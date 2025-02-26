use std::collections::HashMap;
use log::{info, error};

pub trait NetworkingOperations {
    fn start(&self) -> Result<(), String>;
    fn stop(&self) -> Result<(), String>;
    fn connect(&self, address: &str) -> Result<(), String>;
    fn disconnect(&self, address: &str) -> Result<(), String>;
    fn send_message(&self, address: &str, message: &[u8]) -> Result<(), String>;
    fn receive_message(&self, address: &str) -> Result<Vec<u8>, String>;
}

pub struct NetworkManager {
    cache: HashMap<String, Vec<u8>>,
}

impl NetworkManager {
    pub fn new() -> Self {
        NetworkManager {
            cache: HashMap::new(),
        }
    }
}

impl NetworkingOperations for NetworkManager {
    fn start(&self) -> Result<(), String> {
        info!("Starting network connections");
        Ok(())
    }

    fn stop(&self) -> Result<(), String> {
        info!("Stopping network connections");
        Ok(())
    }

    fn connect(&self, address: &str) -> Result<(), String> {
        info!("Connecting to network address: {}", address);
        Ok(())
    }

    fn disconnect(&self, address: &str) -> Result<(), String> {
        info!("Disconnecting from network address: {}", address);
        Ok(())
    }

    fn send_message(&self, address: &str, message: &[u8]) -> Result<(), String> {
        info!("Sending message to network address: {}", address);
        self.cache.insert(address.to_string(), message.to_vec());
        Ok(())
    }

    fn receive_message(&self, address: &str) -> Result<Vec<u8>, String> {
        info!("Receiving message from network address: {}", address);
        if let Some(message) = self.cache.get(address) {
            Ok(message.clone())
        } else {
            Ok(vec![])
        }
    }
}
