use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TreasuryConfig {
    sharing_fee_percent: Decimal,
    exit_bond_amount: Decimal,
    min_resolution_period: chrono::Duration,
}

pub struct FederationTreasury {
    config: TreasuryConfig,
    balance: Decimal,
    exit_bonds: HashMap<String, ExitBond>,
}

impl FederationTreasury {
    pub fn new(config: TreasuryConfig) -> Self {
        Self {
            config,
            balance: Decimal::zero(),
            exit_bonds: HashMap::new(),
        }
    }

    pub fn require_exit_bond(&mut self, federation_id: String) -> Result<ExitBond, String> {
        let bond = ExitBond {
            amount: self.config.exit_bond_amount,
            locked_until: chrono::Utc::now() + self.config.min_resolution_period,
        };
        self.exit_bonds.insert(federation_id, bond.clone());
        Ok(bond)
    }

    pub fn process_sharing_fee(&mut self, transaction_amount: Decimal) -> Decimal {
        let fee = transaction_amount * self.config.sharing_fee_percent;
        self.balance += fee;
        fee
    }
}
