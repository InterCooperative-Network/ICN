use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{RwLock, oneshot};
use super::{ShardConfig, ShardingError, ShardingResult, ShardInfo};
use crate::proof_of_cooperation::ProofOfCooperation;
use icn_types::{Block, Transaction};

/// Status of a cross-shard transaction
#[derive(Debug, Clone, PartialEq)]
pub enum CrossShardStatus {
    /// Transaction is being prepared across shards
    Preparing,
    /// All shards have agreed to commit
    ReadyToCommit,
    /// Transaction has been committed
    Committed,
    /// Transaction has been aborted
    Aborted,
    /// Transaction timed out
    TimedOut,
}

/// Represents a cross-shard transaction
#[derive(Debug)]
pub struct CrossShardTransaction {
    /// Unique transaction ID
    pub id: String,
    /// Source shard ID
    pub source_shard: u32,
    /// Target shard ID
    pub target_shard: u32,
    /// The actual transaction
    pub transaction: Transaction,
    /// Current status
    pub status: CrossShardStatus,
    /// Timestamp when transaction was created
    pub created_at: u64,
    /// Channel to notify when transaction completes
    pub completion_tx: Option<oneshot::Sender<CrossShardStatus>>,
}

/// Manages cross-shard consensus and coordination
pub struct CrossShardConsensus {
    config: ShardConfig,
    consensus: Arc<ProofOfCooperation>,
    active_transactions: Arc<RwLock<HashMap<String, CrossShardTransaction>>>,
    prepared_transactions: Arc<RwLock<HashSet<String>>>,
    committed_transactions: Arc<RwLock<HashSet<String>>>,
}

impl CrossShardConsensus {
    pub fn new(config: ShardConfig, consensus: Arc<ProofOfCooperation>) -> Self {
        Self {
            config,
            consensus,
            active_transactions: Arc::new(RwLock::new(HashMap::new())),
            prepared_transactions: Arc::new(RwLock::new(HashSet::new())),
            committed_transactions: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Start a new cross-shard transaction
    pub async fn begin_transaction(
        &self,
        transaction: Transaction,
        source_shard: u32,
        target_shard: u32,
    ) -> ShardingResult<oneshot::Receiver<CrossShardStatus>> {
        let (completion_tx, completion_rx) = oneshot::channel();

        let cross_shard_tx = CrossShardTransaction {
            id: format!("cst_{}", transaction.id),
            source_shard,
            target_shard,
            transaction,
            status: CrossShardStatus::Preparing,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            completion_tx: Some(completion_tx),
        };

        // Store transaction
        let mut active = self.active_transactions.write().await;
        active.insert(cross_shard_tx.id.clone(), cross_shard_tx);

        // Start two-phase commit protocol
        self.prepare_transaction(&cross_shard_tx.id).await?;

        Ok(completion_rx)
    }

    /// Phase 1: Prepare transaction on all shards
    async fn prepare_transaction(&self, tx_id: &str) -> ShardingResult<()> {
        let active = self.active_transactions.read().await;
        let tx = active.get(tx_id).ok_or_else(|| {
            ShardingError::TransactionNotFound(tx_id.to_string())
        })?;

        // Prepare on source shard
        self.prepare_on_shard(tx.source_shard, tx).await?;

        // Prepare on target shard
        self.prepare_on_shard(tx.target_shard, tx).await?;

        // Mark as prepared
        let mut prepared = self.prepared_transactions.write().await;
        prepared.insert(tx_id.to_string());

        // Move to commit phase
        self.update_transaction_status(tx_id, CrossShardStatus::ReadyToCommit).await?;

        // Start commit phase
        self.commit_transaction(tx_id).await?;

        Ok(())
    }

    /// Prepare transaction on a specific shard
    async fn prepare_on_shard(&self, shard_id: u32, tx: &CrossShardTransaction) -> ShardingResult<()> {
        // Create prepare block for the shard
        let prepare_block = Block {
            index: 0, // Will be set by consensus
            previous_hash: String::new(), // Will be set by consensus
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            transactions: vec![tx.transaction.clone()],
            hash: String::new(), // Will be set by consensus
            proposer: String::new(), // Will be set by consensus
            signatures: Vec::new(), // Will be filled during consensus
        };

        // Run consensus for prepare phase
        self.consensus.finalize_block(prepare_block).await
            .map_err(|e| ShardingError::ConsensusError(e.to_string()))?;

        Ok(())
    }

    /// Phase 2: Commit transaction on all shards
    async fn commit_transaction(&self, tx_id: &str) -> ShardingResult<()> {
        let active = self.active_transactions.read().await;
        let tx = active.get(tx_id).ok_or_else(|| {
            ShardingError::TransactionNotFound(tx_id.to_string())
        })?;

        // Verify transaction is ready to commit
        if tx.status != CrossShardStatus::ReadyToCommit {
            return Err(ShardingError::InvalidConfig(
                "Transaction not ready to commit".to_string()
            ));
        }

        // Commit on source shard
        self.commit_on_shard(tx.source_shard, tx).await?;

        // Commit on target shard
        self.commit_on_shard(tx.target_shard, tx).await?;

        // Mark as committed
        let mut committed = self.committed_transactions.write().await;
        committed.insert(tx_id.to_string());

        // Update status and notify completion
        self.update_transaction_status(tx_id, CrossShardStatus::Committed).await?;

        Ok(())
    }

    /// Commit transaction on a specific shard
    async fn commit_on_shard(&self, shard_id: u32, tx: &CrossShardTransaction) -> ShardingResult<()> {
        // Create commit block for the shard
        let commit_block = Block {
            index: 0, // Will be set by consensus
            previous_hash: String::new(), // Will be set by consensus
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            transactions: vec![tx.transaction.clone()],
            hash: String::new(), // Will be set by consensus
            proposer: String::new(), // Will be set by consensus
            signatures: Vec::new(), // Will be filled during consensus
        };

        // Run consensus for commit phase
        self.consensus.finalize_block(commit_block).await
            .map_err(|e| ShardingError::ConsensusError(e.to_string()))?;

        Ok(())
    }

    /// Update transaction status and notify completion if needed
    async fn update_transaction_status(
        &self,
        tx_id: &str,
        new_status: CrossShardStatus,
    ) -> ShardingResult<()> {
        let mut active = self.active_transactions.write().await;
        if let Some(tx) = active.get_mut(tx_id) {
            tx.status = new_status.clone();

            // If transaction is complete (committed or aborted), notify completion
            if new_status == CrossShardStatus::Committed || new_status == CrossShardStatus::Aborted {
                if let Some(completion_tx) = tx.completion_tx.take() {
                    let _ = completion_tx.send(new_status);
                }
            }
        }

        Ok(())
    }

    /// Abort a cross-shard transaction
    pub async fn abort_transaction(&self, tx_id: &str) -> ShardingResult<()> {
        self.update_transaction_status(tx_id, CrossShardStatus::Aborted).await?;
        Ok(())
    }

    /// Check for timed out transactions and abort them
    pub async fn check_timeouts(&self) -> ShardingResult<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut active = self.active_transactions.write().await;
        let timeout = self.config.cross_shard_timeout;

        for (tx_id, tx) in active.iter_mut() {
            if tx.status != CrossShardStatus::Committed && tx.status != CrossShardStatus::Aborted {
                if now - tx.created_at > timeout {
                    tx.status = CrossShardStatus::TimedOut;
                    if let Some(completion_tx) = tx.completion_tx.take() {
                        let _ = completion_tx.send(CrossShardStatus::TimedOut);
                    }
                }
            }
        }

        Ok(())
    }

    /// Get transaction status
    pub async fn get_transaction_status(&self, tx_id: &str) -> Option<CrossShardStatus> {
        let active = self.active_transactions.read().await;
        active.get(tx_id).map(|tx| tx.status.clone())
    }

    /// Get all active cross-shard transactions
    pub async fn get_active_transactions(&self) -> Vec<CrossShardTransaction> {
        let active = self.active_transactions.read().await;
        active.values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cross_shard_transaction() {
        let config = ShardConfig::default();
        let consensus = Arc::new(ProofOfCooperation::new(/* ... */));
        let cross_shard = CrossShardConsensus::new(config, consensus);

        let transaction = Transaction::default();
        let completion_rx = cross_shard.begin_transaction(
            transaction,
            0, // source shard
            1, // target shard
        ).await.unwrap();

        // Wait for completion
        let status = completion_rx.await.unwrap();
        assert_eq!(status, CrossShardStatus::Committed);
    }

    #[tokio::test]
    async fn test_transaction_timeout() {
        let mut config = ShardConfig::default();
        config.cross_shard_timeout = 1; // 1 second timeout
        let consensus = Arc::new(ProofOfCooperation::new(/* ... */));
        let cross_shard = CrossShardConsensus::new(config, consensus);

        let transaction = Transaction::default();
        let completion_rx = cross_shard.begin_transaction(
            transaction,
            0,
            1,
        ).await.unwrap();

        // Wait for more than timeout
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Check timeouts
        cross_shard.check_timeouts().await.unwrap();

        let status = completion_rx.await.unwrap();
        assert_eq!(status, CrossShardStatus::TimedOut);
    }
} 