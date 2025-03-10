use std::env;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
    pub cors_origins: Vec<String>,
    pub log_level: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 8085,
            host: "0.0.0.0".to_string(),
            cors_origins: vec!["*".to_string()],
            log_level: "info".to_string(),
        }
    }
}

impl ServerConfig {
    pub fn from_env() -> Self {
        let port = env::var("ICN_SERVER_PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(8085);

        let host = env::var("ICN_SERVER_HOST")
            .unwrap_or_else(|_| "0.0.0.0".to_string());

        let log_level = env::var("RUST_LOG")
            .unwrap_or_else(|_| "info".to_string());

        let cors_origins = env::var("ICN_CORS_ORIGINS")
            .map(|origins| origins.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_else(|_| vec!["*".to_string()]);

        Self {
            port,
            host,
            cors_origins,
            log_level,
        }
    }

    pub fn socket_addr(&self) -> std::net::SocketAddr {
        use std::net::ToSocketAddrs;
        let addr = format!("{}:{}", self.host, self.port);
        addr.to_socket_addrs()
            .expect("Failed to parse socket address")
            .next()
            .expect("No socket addresses found")
    }
} 