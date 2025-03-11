use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use thiserror::Error;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use icn_types::FederationId;
use crate::FederationError;
use crate::federation_id_to_string;

/// Error types for treasury operations
#[derive(Error, Debug)]
pub enum TreasuryError {
    #[error("Insufficient funds: required {required}, available {available}")]
    InsufficientFunds { required: u64, available: u64 },
    
    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),
    
    #[error("Unauthorized operation: {0}")]
    Unauthorized(String),
    
    #[error("Asset not found: {0}")]
    AssetNotFound(String),
    
    #[error("Federation not found: {0}")]
    FederationNotFound(String),
    
    #[error("Treasury operation failed: {0}")]
    OperationFailed(String),
}

/// Types of assets managed by the treasury
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum AssetType {
    /// Mutual credit units internal to federation
    Credit,
    
    /// Reputation tokens
    ReputationToken,
    
    /// Compute resource allocation
    ComputeResource,
    
    /// Storage resource allocation
    StorageResource,
    
    /// Bandwidth resource allocation
    BandwidthResource,
    
    /// Custom asset type
    Custom(String),
}

/// A transaction in the treasury ledger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreasuryTransaction {
    /// Unique transaction ID
    pub id: String,
    
    /// Asset type for this transaction
    pub asset_type: AssetType,
    
    /// Amount being transferred
    pub amount: u64,
    
    /// Sender DID or federation ID
    pub from: String,
    
    /// Recipient DID or federation ID
    pub to: String,
    
    /// Timestamp of the transaction
    pub timestamp: DateTime<Utc>,
    
    /// Transaction state
    pub state: TransactionState,
    
    /// Transaction purpose or description
    pub description: Option<String>,
    
    /// Optional reference to other transactions
    pub references: Vec<String>,
    
    /// Transaction metadata (custom key-value pairs)
    pub metadata: HashMap<String, String>,
}

/// Current state of a transaction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionState {
    /// Transaction is pending approval
    Pending,
    
    /// Transaction has been executed successfully
    Completed,
    
    /// Transaction has been rejected
    Rejected,
    
    /// Transaction has failed
    Failed,
}

/// Asset balance entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetBalance {
    /// Asset type
    pub asset_type: AssetType,
    
    /// Available balance
    pub available: u64,
    
    /// Reserved/allocated balance
    pub reserved: u64,
    
    /// Total balance (available + reserved)
    pub total: u64,
    
    /// Last update timestamp
    pub last_updated: DateTime<Utc>,
}

/// Budget allocation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetAllocation {
    /// Asset type
    pub asset_type: AssetType,
    
    /// Total amount to allocate
    pub amount: u64,
    
    /// Period for the allocation (in seconds)
    pub period_seconds: u64,
    
    /// Start timestamp for the allocation
    pub start_time: DateTime<Utc>,
    
    /// End timestamp for the allocation (None means indefinite)
    pub end_time: Option<DateTime<Utc>>,
    
    /// Purpose or use case for this budget
    pub purpose: String,
    
    /// Optional restrictions on usage
    pub restrictions: Option<HashMap<String, String>>,
}

/// Treasury for managing federation assets
pub struct Treasury {
    /// Federation ID owning this treasury
    federation_id: FederationId,
    
    /// Asset balances by asset type
    balances: RwLock<HashMap<AssetType, AssetBalance>>,
    
    /// Transaction history
    transactions: RwLock<Vec<TreasuryTransaction>>,
    
    /// Budget allocations
    budget_allocations: RwLock<HashMap<String, BudgetAllocation>>,
    
    /// Pending transactions requiring approval
    pending_transactions: RwLock<Vec<TreasuryTransaction>>,
    
    /// Multi-signature requirements for transactions
    required_signatures: RwLock<HashMap<AssetType, u32>>,
    
    /// Spending limits by asset type
    spending_limits: RwLock<HashMap<AssetType, u64>>,
}

impl Treasury {
    /// Create a new treasury for a federation
    pub fn new(federation_id: FederationId) -> Self {
        Self {
            federation_id,
            balances: RwLock::new(HashMap::new()),
            transactions: RwLock::new(Vec::new()),
            budget_allocations: RwLock::new(HashMap::new()),
            pending_transactions: RwLock::new(Vec::new()),
            required_signatures: RwLock::new(HashMap::new()),
            spending_limits: RwLock::new(HashMap::new()),
        }
    }

    /// Initialize an asset balance
    pub async fn initialize_asset(&self, asset_type: AssetType, initial_balance: u64) -> Result<(), TreasuryError> {
        // Check if asset already exists
        let mut balances = self.balances.write().await;
        
        // Create a borrowed reference for asset_type_to_string
        let asset_type_ref = &asset_type;
        if balances.contains_key(asset_type_ref) {
            return Err(TreasuryError::OperationFailed(
                format!("Asset already initialized: {}", asset_type_to_string(asset_type_ref))
            ));
        }
        
        // Create new balance entry
        let balance = AssetBalance {
            asset_type: asset_type.clone(),
            available: initial_balance,
            reserved: 0,
            total: initial_balance,
            last_updated: Utc::now(),
        };
        
        // Add to balances with a clone
        balances.insert(asset_type.clone(), balance);
        
        // Record initialization transaction
        let tx = TreasuryTransaction {
            id: Uuid::new_v4().to_string(),
            asset_type: asset_type.clone(),
            amount: initial_balance,
            from: "system".to_string(),
            to: federation_id_to_string(&self.federation_id),
            timestamp: Utc::now(),
            state: TransactionState::Completed,
            description: Some(format!("Initialize {} asset", asset_type_to_string(&asset_type))),
            references: vec![],
            metadata: HashMap::new(),
        };
        
        self.record_transaction(tx).await;
        
        Ok(())
    }

    /// Set required signature count for an asset type
    pub async fn set_required_signatures(&self, asset_type: AssetType, signatures: u32) {
        let mut required = self.required_signatures.write().await;
        required.insert(asset_type, signatures);
    }

    /// Set spending limit for an asset type
    pub async fn set_spending_limit(&self, asset_type: AssetType, limit: u64) {
        let mut limits = self.spending_limits.write().await;
        limits.insert(asset_type, limit);
    }

    /// Record a transaction in the ledger
    async fn record_transaction(&self, transaction: TreasuryTransaction) {
        let mut transactions = self.transactions.write().await;
        transactions.push(transaction);
    }

    /// Get balance for an asset type
    pub async fn get_balance(&self, asset_type: &AssetType) -> Result<AssetBalance, TreasuryError> {
        let balances = self.balances.read().await;
        
        balances.get(asset_type)
            .cloned()
            .ok_or_else(|| TreasuryError::AssetNotFound(asset_type_to_string(asset_type)))
    }

    /// Create a transfer transaction
    pub async fn create_transfer(
        &self,
        asset_type: AssetType,
        amount: u64,
        from: String,
        to: String,
        description: Option<String>,
        references: Vec<String>,
    ) -> Result<String, TreasuryError> {
        // Verify the asset exists
        let mut balances = self.balances.write().await;
        
        let balance = balances.get_mut(&asset_type)
            .ok_or_else(|| TreasuryError::AssetNotFound(asset_type_to_string(&asset_type)))?;
        
        // Check for sufficient funds
        if balance.available < amount {
            return Err(TreasuryError::InsufficientFunds {
                required: amount,
                available: balance.available,
            });
        }
        
        // Check if the transaction needs approval
        let required_signatures = {
            let sigs = self.required_signatures.read().await;
            sigs.get(&asset_type).cloned().unwrap_or(0)
        };
        
        // Check spending limits
        let is_within_limit = {
            let limits = self.spending_limits.read().await;
            match limits.get(&asset_type) {
                Some(&limit) => amount <= limit,
                None => true, // No limit set
            }
        };
        
        let transaction = TreasuryTransaction {
            id: Uuid::new_v4().to_string(),
            asset_type: asset_type.clone(),
            amount,
            from: from.clone(),
            to: to.clone(),
            timestamp: Utc::now(),
            state: if required_signatures > 0 { TransactionState::Pending } else { TransactionState::Completed },
            description,
            references,
            metadata: HashMap::new(),
        };
        
        // If no approval is needed, execute immediately
        if required_signatures == 0 && is_within_limit {
            // Update balances
            balance.available -= amount;
            balance.last_updated = Utc::now();
            
            // Record the transaction
            self.record_transaction(transaction.clone()).await;
            
            return Ok(transaction.id);
        }
        
        // Otherwise, queue for approval
        let mut pending = self.pending_transactions.write().await;
        pending.push(transaction.clone());
        
        // Record as pending transaction
        self.record_transaction(transaction.clone()).await;
        
        Ok(transaction.id)
    }

    /// Approve a pending transaction
    pub async fn approve_transaction(&self, transaction_id: &str, approver: &str) -> Result<(), TreasuryError> {
        let mut pending = self.pending_transactions.write().await;
        let tx_index = pending.iter().position(|tx| tx.id == transaction_id)
            .ok_or_else(|| TreasuryError::InvalidTransaction(format!("Transaction not found: {}", transaction_id)))?;
        
        let transaction = &mut pending[tx_index];
        
        // Add approval to metadata
        let approver_key = format!("approved_by_{}", approver);
        transaction.metadata.insert(approver_key, Utc::now().to_rfc3339());
        
        // Count approvals
        let approval_count = transaction.metadata.keys()
            .filter(|k| k.starts_with("approved_by_"))
            .count() as u32;
        
        // Get required approvals for this asset type
        let required_approvals = {
            let sigs = self.required_signatures.read().await;
            sigs.get(&transaction.asset_type).cloned().unwrap_or(0)
        };
        
        // Execute if we have enough approvals
        if approval_count >= required_approvals {
            // Update transaction state
            transaction.state = TransactionState::Completed;
            
            // Update balances
            let mut balances = self.balances.write().await;
            
            let from_balance = balances.get_mut(&transaction.asset_type)
                .ok_or_else(|| TreasuryError::AssetNotFound(asset_type_to_string(&transaction.asset_type)))?;
            
            // Update sender's balance
            from_balance.available -= transaction.amount;
            from_balance.last_updated = Utc::now();
            
            // Remove from pending queue
            pending.remove(tx_index);
        }
        
        Ok(())
    }

    /// Reject a pending transaction
    pub async fn reject_transaction(&self, transaction_id: &str, rejector: &str, reason: &str) -> Result<(), TreasuryError> {
        let mut pending = self.pending_transactions.write().await;
        let tx_index = pending.iter().position(|tx| tx.id == transaction_id)
            .ok_or_else(|| TreasuryError::InvalidTransaction(format!("Transaction not found: {}", transaction_id)))?;
        
        let transaction = &mut pending[tx_index];
        
        // Update transaction state
        transaction.state = TransactionState::Rejected;
        
        // Add rejection details to metadata
        let rejector_key = format!("rejected_by_{}", rejector);
        transaction.metadata.insert(rejector_key, Utc::now().to_rfc3339());
        transaction.metadata.insert("rejection_reason".to_string(), reason.to_string());
        
        // Remove from pending queue
        pending.remove(tx_index);
        
        Ok(())
    }

    /// Create a new budget allocation
    pub async fn create_budget_allocation(&self, allocation: BudgetAllocation) -> Result<String, TreasuryError> {
        // Verify the asset exists
        let balances = self.balances.read().await;
        if !balances.contains_key(&allocation.asset_type) {
            return Err(TreasuryError::AssetNotFound(asset_type_to_string(&allocation.asset_type)));
        }
        
        // Generate allocation ID
        let allocation_id = Uuid::new_v4().to_string();
        
        // Store the allocation
        let mut budget_allocations = self.budget_allocations.write().await;
        budget_allocations.insert(allocation_id.clone(), allocation);
        
        Ok(allocation_id)
    }

    /// Get all transactions for an asset type
    pub async fn get_transactions_by_asset(&self, asset_type: &AssetType) -> Vec<TreasuryTransaction> {
        let transactions = self.transactions.read().await;
        transactions.iter()
            .filter(|tx| tx.asset_type == *asset_type)
            .cloned()
            .collect()
    }

    /// Get all transactions within a time range
    pub async fn get_transactions_by_time_range(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Vec<TreasuryTransaction> {
        let transactions = self.transactions.read().await;
        transactions.iter()
            .filter(|tx| tx.timestamp >= start_time && tx.timestamp <= end_time)
            .cloned()
            .collect()
    }

    /// Get all pending transactions
    pub async fn get_pending_transactions(&self) -> Vec<TreasuryTransaction> {
        let pending = self.pending_transactions.read().await;
        pending.clone()
    }

    /// Reconcile and adjust balances (e.g., for periodic budget allocations)
    pub async fn reconcile_balances(&self) -> Result<HashMap<AssetType, u64>, TreasuryError> {
        let now = Utc::now();
        let mut adjustments = HashMap::new();
        
        // Process budget allocations
        let budget_allocations = self.budget_allocations.read().await;
        
        for (allocation_id, allocation) in budget_allocations.iter() {
            // Skip allocations that haven't started yet
            if allocation.start_time > now {
                continue;
            }
            
            // Skip allocations that have ended
            if let Some(end_time) = allocation.end_time {
                if end_time < now {
                    continue;
                }
            }
            
            // Calculate time since last allocation
            // In a real implementation, we'd track the last allocation time
            // For now, we'll just allocate the full amount as a demonstration
            
            // Update adjustments
            let entry = adjustments.entry(allocation.asset_type.clone()).or_insert(0);
            *entry += allocation.amount;
        }
        
        // Apply adjustments to balances
        let mut balances = self.balances.write().await;
        
        for (asset_type, adjustment) in &adjustments {
            let balance = balances.get_mut(asset_type)
                .ok_or_else(|| TreasuryError::AssetNotFound(asset_type_to_string(asset_type)))?;
            
            balance.available += *adjustment;
            balance.total += *adjustment;
            balance.last_updated = now;
        }
        
        Ok(adjustments)
    }
}

/// Helper function to convert asset type to string
fn asset_type_to_string(asset_type: &AssetType) -> String {
    match asset_type {
        AssetType::Credit => "Credit".to_string(),
        AssetType::ReputationToken => "ReputationToken".to_string(),
        AssetType::ComputeResource => "ComputeResource".to_string(),
        AssetType::StorageResource => "StorageResource".to_string(),
        AssetType::BandwidthResource => "BandwidthResource".to_string(),
        AssetType::Custom(name) => format!("Custom({})", name),
    }
}
