use serde::{Serialize, Deserialize};
use std::collections::HashSet;

#[derive(Debug, Serialize, Deserialize)]
pub struct GenesisConfig {
    pub timestamp: u64,
    pub initial_validators: HashSet<ValidatorInfo>,
    pub governance_params: GovernanceParams,
}

#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct ValidatorInfo {
    pub did: String,
    pub coop_id: String,
    pub public_key: String,
    pub stake: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GovernanceParams {
    pub min_validators: u32,
    pub max_validators_per_coop: u32,
    pub election_period: u64,
}

impl GenesisConfig {
    pub fn new_testnet() -> Self {
        Self {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            initial_validators: HashSet::new(),
            governance_params: GovernanceParams {
                min_validators: 4,
                max_validators_per_coop: 2,
                election_period: 40320, // ~7 days with 15s blocks
            },
        }
    }
}
