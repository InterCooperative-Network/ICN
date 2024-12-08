use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::interval;
use sqlx::PgPool;
use tracing::{debug, error, info, warn};

use crate::error::{StorageError, StorageResult};

/// Types of metrics collected by the storage system
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MetricType {
    BlockWriteTime,
    BlockReadTime,
    TransactionWriteTime,
    TransactionReadTime,
    CacheHitRate,
    CacheMissRate,
    ConnectionPoolSize,
    ConnectionPoolWaitTime,
    QueryExecutionTime,
    DiskUsage,
    IndexUsage,
    BackupOperation,
    BackupDuration,
    RestoreDuration,
}

// Rest of the metrics.rs file remains the same...

/// Metric data point with associated metadata
#[derive(Debug, Clone)]
pub struct MetricPoint {
    pub metric_type: MetricType,
    pub value: f64,
    pub timestamp: i64,
    pub tags: HashMap<String, String>,
}

/// Manages metric collection and storage
#[derive(Clone)]
pub struct MetricsManager {
    pool: PgPool,
    metrics_cache: Arc<RwLock<HashMap<MetricType, Vec<MetricPoint>>>>,
    flush_interval: Duration,
}

impl MetricsManager {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            metrics_cache: Arc::new(RwLock::new(HashMap::new())),
            flush_interval: Duration::from_secs(60),
        }
    }

    /// Start the metrics collection background task
    pub async fn start_collection(&self) -> StorageResult<()> {
        let metrics_manager = self.clone();
        tokio::spawn(async move {
            let mut interval = interval(metrics_manager.flush_interval);
            loop {
                interval.tick().await;
                if let Err(e) = metrics_manager.flush_metrics().await {
                    error!("Failed to flush metrics: {}", e);
                }
            }
        });
        Ok(())
    }

    /// Record a new metric data point
    pub async fn record_metric(
        &self,
        metric_type: MetricType,
        value: f64,
        tags: Option<HashMap<String, String>>,
    ) -> StorageResult<()> {
        let timestamp = chrono::Utc::now().timestamp();
        let metric = MetricPoint {
            metric_type: metric_type.clone(),
            value,
            timestamp,
            tags: tags.unwrap_or_default(),
        };

        let mut cache = self.metrics_cache.write().await;
        cache.entry(metric_type)
            .or_insert_with(Vec::new)
            .push(metric);

        Ok(())
    }

    /// Record operation duration
    pub async fn record_duration(
        &self,
        metric_type: MetricType,
        duration: Duration,
        tags: Option<HashMap<String, String>>,
    ) -> StorageResult<()> {
        self.record_metric(
            metric_type,
            duration.as_secs_f64() * 1000.0, // Convert to milliseconds
            tags,
        ).await
    }

    /// Flush metrics to database
    async fn flush_metrics(&self) -> StorageResult<()> {
        let mut cache = self.metrics_cache.write().await;
        if cache.is_empty() {
            return Ok(());
        }

        let mut tx = self.pool
            .begin()
            .await
            .map_err(|e| StorageError::TransactionError(e.to_string()))?;

        for (metric_type, points) in cache.iter() {
            for point in points {
                sqlx::query(
                    "INSERT INTO metrics (metric_type, value, tags, timestamp)
                     VALUES ($1, $2, $3, $4)"
                )
                .bind(format!("{:?}", metric_type))
                .bind(point.value)
                .bind(serde_json::to_value(&point.tags)
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?)
                .bind(point.timestamp)
                .execute(&mut *tx)
                .await
                .map_err(|e| StorageError::QueryError(e.to_string()))?;
            }
        }

        tx.commit()
            .await
            .map_err(|e| StorageError::TransactionError(e.to_string()))?;

        cache.clear();
        Ok(())
    }

    /// Get metrics for a specific type within a time range
    pub async fn get_metrics(
        &self,
        metric_type: MetricType,
        start_time: i64,
        end_time: i64,
    ) -> StorageResult<Vec<MetricPoint>> {
        let rows = sqlx::query(
            "SELECT value, tags, timestamp
             FROM metrics
             WHERE metric_type = $1
             AND timestamp >= $2
             AND timestamp <= $3
             ORDER BY timestamp ASC"
        )
        .bind(format!("{:?}", metric_type))
        .bind(start_time)
        .bind(end_time)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::QueryError(e.to_string()))?;

        let mut metrics = Vec::with_capacity(rows.len());
        for row in rows {
            let tags: HashMap<String, String> = serde_json::from_value(row.get("tags"))
                .map_err(|e| StorageError::SerializationError(e.to_string()))?;

            metrics.push(MetricPoint {
                metric_type: metric_type.clone(),
                value: row.get("value"),
                timestamp: row.get("timestamp"),
                tags,
            });
        }

        Ok(metrics)
    }

    /// Calculate average value for a metric type within a time range
    pub async fn get_metric_average(
        &self,
        metric_type: MetricType,
        start_time: i64,
        end_time: i64,
    ) -> StorageResult<f64> {
        let row = sqlx::query(
            "SELECT AVG(value) as avg_value
             FROM metrics
             WHERE metric_type = $1
             AND timestamp >= $2
             AND timestamp <= $3"
        )
        .bind(format!("{:?}", metric_type))
        .bind(start_time)
        .bind(end_time)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| StorageError::QueryError(e.to_string()))?;

        Ok(row.get("avg_value"))
    }

    /// Clean up old metrics data
    pub async fn cleanup_old_metrics(&self, older_than: i64) -> StorageResult<u64> {
        let result = sqlx::query(
            "DELETE FROM metrics
             WHERE timestamp < $1"
        )
        .bind(older_than)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::QueryError(e.to_string()))?;

        Ok(result.rows_affected())
    }
}

/// Utility struct for timing operations
pub struct OperationTimer {
    start: Instant,
    metric_type: MetricType,
    metrics_manager: MetricsManager,
    tags: Option<HashMap<String, String>>,
}

impl OperationTimer {
    pub fn new(
        metric_type: MetricType,
        metrics_manager: MetricsManager,
        tags: Option<HashMap<String, String>>,
    ) -> Self {
        Self {
            start: Instant::now(),
            metric_type,
            metrics_manager,
            tags,
        }
    }
}

impl Drop for OperationTimer {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        let metrics_manager = self.metrics_manager.clone();
        let metric_type = self.metric_type.clone();
        let tags = self.tags.clone();

        tokio::spawn(async move {
            if let Err(e) = metrics_manager.record_duration(metric_type, duration, tags).await {
                error!("Failed to record operation duration: {}", e);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_metrics_recording() {
        let config = crate::StorageConfig::default();
        let pool = sqlx::PgPool::connect(&config.database_url)
            .await
            .unwrap();
        
        let metrics = MetricsManager::new(pool);
        
        let mut tags = HashMap::new();
        tags.insert("test_tag".to_string(), "test_value".to_string());
        
        metrics.record_metric(
            MetricType::BlockWriteTime,
            42.0,
            Some(tags)
        ).await.unwrap();
        
        // Allow time for flush
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        let now = chrono::Utc::now().timestamp();
        let metrics = metrics.get_metrics(
            MetricType::BlockWriteTime,
            now - 3600,
            now
        ).await.unwrap();
        
        assert!(!metrics.is_empty());
        assert_eq!(metrics[0].value, 42.0);
    }

    #[tokio::test]
    async fn test_operation_timer() {
        let config = crate::StorageConfig::default();
        let pool = sqlx::PgPool::connect(&config.database_url)
            .await
            .unwrap();
        
        let metrics = MetricsManager::new(pool);
        
        {
            let _timer = OperationTimer::new(
                MetricType::QueryExecutionTime,
                metrics.clone(),
                None
            );
            // Simulate some work
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        // Allow time for flush
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        let now = chrono::Utc::now().timestamp();
        let avg_duration = metrics.get_metric_average(
            MetricType::QueryExecutionTime,
            now - 3600,
            now
        ).await.unwrap();
        
        assert!(avg_duration >= 100.0); // Should be at least 100ms
    }
}