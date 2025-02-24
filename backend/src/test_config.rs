use lazy_static::lazy_static;
use std::env;
use std::sync::Once;

static INIT: Once = Once::new();

lazy_static! {
    pub static ref TEST_CONFIG: TestConfig = TestConfig::new();
}

pub struct TestConfig {
    pub database_url: String,
    pub test_did: String,
    pub test_public_key: String,
    pub test_private_key: String,
}

impl TestConfig {
    pub fn new() -> Self {
        // Initialize test environment
        INIT.call_once(|| {
            env_logger::init();
        });

        Self {
            database_url: env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgres://icnuser:icnpass@db:5432/icndb_test".to_string()),
            test_did: "did:icn:test".to_string(),
            test_public_key: "test_public_key".to_string(),
            test_private_key: "test_private_key".to_string(),
        }
    }
}

/// Test environment setup
pub fn setup_test_env() {
    INIT.call_once(|| {
        env_logger::init();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_initialization() {
        let config = TestConfig::new();
        assert!(!config.database_url.is_empty());
        assert!(!config.test_did.is_empty());
    }

    #[test]
    fn test_env_setup() {
        setup_test_env();
        // Verify it can be called multiple times without issues
        setup_test_env();
    }
} 