use std::sync::Arc;
use tokio::sync::Mutex;
use crate::networking::NetworkManager;

pub struct Core {
    pub network_manager: Arc<Mutex<NetworkManager>>,
}

impl Core {
    pub fn new(network_manager: Arc<Mutex<NetworkManager>>) -> Self {
        Self { network_manager }
    }
}
