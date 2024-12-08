use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use sqlx::PgPool;
use tracing::{debug, error, info, warn};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

use crate::error::{StorageError, StorageResult};
use crate::metrics::{MetricsManager, MetricType};

/// Configuration for the backup system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    /// Directory to store backups
    pub backup_dir: PathBuf,
    /// Maximum number of backups to keep
    pub max_backups: usize,
    /// Backup file prefix
    pub backup_prefix: String,
    /// Whether to compress backups
    pub compress: bool,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            backup_dir: PathBuf::from("backups"),
            max_backups: 5,
            backup_prefix: "icn_backup".to_string(),
            compress: true,
        }
    }
}

/// Information about a backup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    /// Unique identifier for the backup
    pub id: String,
    /// When the backup was created
    pub created_at: DateTime<Utc>,
    /// Size of the backup in bytes
    pub size: u64,
    /// Path to the backup file
    pub path: PathBuf,
    /// Whether the backup is compressed
    pub compressed: bool,
    /// Hash of the backup content
    pub hash: String,
}

/// Manages database backups and recovery
pub struct BackupManager {
    pool: PgPool,
    config: BackupConfig,
    metrics: MetricsManager,
}

impl BackupManager {
    pub fn new(pool: PgPool, config: BackupConfig, metrics: MetricsManager) -> Self {
        Self {
            pool,
            config,
            metrics,
        }
    }

    /// Create a new backup of the database
    pub async fn create_backup(&self) -> StorageResult<BackupInfo> {
        let start = SystemTime::now();
        
        // Ensure backup directory exists
        fs::create_dir_all(&self.config.backup_dir).await
            .map_err(|e| StorageError::BackupError(format!("Failed to create backup directory: {}", e)))?;

        // Generate backup filename
        let timestamp = start.duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let backup_id = format!("{}_{}", self.config.backup_prefix, timestamp);
        let mut backup_path = self.config.backup_dir.join(&backup_id);
        backup_path.set_extension("sql");

        // Create backup using pg_dump
        let output = tokio::process::Command::new("pg_dump")
            .arg("--format=custom")
            .arg("--file").arg(&backup_path)
            .arg(self.get_database_url())
            .output()
            .await
            .map_err(|e| StorageError::BackupError(format!("Failed to execute pg_dump: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(StorageError::BackupError(format!("pg_dump failed: {}", error)));
        }

        // Compress if configured
        let (final_path, compressed) = if self.config.compress {
            let compressed_path = backup_path.with_extension("sql.gz");
            self.compress_file(&backup_path, &compressed_path).await?;
            fs::remove_file(&backup_path).await
                .map_err(|e| StorageError::BackupError(format!("Failed to remove uncompressed backup: {}", e)))?;
            (compressed_path, true)
        } else {
            (backup_path, false)
        };

        // Calculate hash and size
        let content = fs::read(&final_path).await
            .map_err(|e| StorageError::BackupError(format!("Failed to read backup: {}", e)))?;
        let hash = sha256::digest(&content);
        let size = content.len() as u64;

        let backup_info = BackupInfo {
            id: backup_id,
            created_at: Utc::now(),
            size,
            path: final_path,
            compressed,
            hash,
        };

        // Record metrics
        let duration = start.elapsed().unwrap();
        self.metrics.record_metric(
            MetricType::BackupDuration,
            duration.as_secs_f64(),
            Some(std::collections::HashMap::from([
                ("size".to_string(), size.to_string()),
                ("compressed".to_string(), compressed.to_string())
            ]))
        ).await?;

        // Cleanup old backups
        self.cleanup_old_backups().await?;

        Ok(backup_info)
    }

    /// Restore from a backup
    pub async fn restore_backup(&self, backup_info: &BackupInfo) -> StorageResult<()> {
        let start = SystemTime::now();

        // Verify backup exists
        if !backup_info.path.exists() {
            return Err(StorageError::BackupError("Backup file not found".to_string()));
        }

        // Verify hash
        let content = fs::read(&backup_info.path).await
            .map_err(|e| StorageError::BackupError(format!("Failed to read backup: {}", e)))?;
        let hash = sha256::digest(&content);
        if hash != backup_info.hash {
            return Err(StorageError::BackupError("Backup hash mismatch".to_string()));
        }

        // Decompress if needed
        let restore_path = if backup_info.compressed {
            let temp_path = backup_info.path.with_extension("sql");
            self.decompress_file(&backup_info.path, &temp_path).await?;
            temp_path
        } else {
            backup_info.path.clone()
        };

        // Restore using pg_restore
        let output = tokio::process::Command::new("pg_restore")
            .arg("--clean")
            .arg("--if-exists")
            .arg("--format=custom")
            .arg("--dbname").arg(self.get_database_url())
            .arg(&restore_path)
            .output()
            .await
            .map_err(|e| StorageError::BackupError(format!("Failed to execute pg_restore: {}", e)))?;

        // Cleanup temporary file if needed
        if backup_info.compressed {
            fs::remove_file(&restore_path).await
                .map_err(|e| StorageError::BackupError(format!("Failed to remove temporary file: {}", e)))?;
        }

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(StorageError::BackupError(format!("pg_restore failed: {}", error)));
        }

        // Record metrics
        let duration = start.elapsed().unwrap();
        self.metrics.record_metric(
            MetricType::RestoreDuration,
            duration.as_secs_f64(),
            Some(std::collections::HashMap::from([
                ("backup_id".to_string(), backup_info.id.clone()),
                ("size".to_string(), backup_info.size.to_string())
            ]))
        ).await?;

        Ok(())
    }

    /// List available backups
    pub async fn list_backups(&self) -> StorageResult<Vec<BackupInfo>> {
        let mut backups = Vec::new();
        let mut entries = fs::read_dir(&self.config.backup_dir).await
            .map_err(|e| StorageError::BackupError(format!("Failed to read backup directory: {}", e)))?;

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| StorageError::BackupError(format!("Failed to read directory entry: {}", e)))? {
                
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "sql" || ext == "gz" {
                    if let Some(backup_info) = self.load_backup_info(&path).await? {
                        backups.push(backup_info);
                    }
                }
            }
        }

        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(backups)
    }

    /// Load backup information from a file
    async fn load_backup_info(&self, path: &Path) -> StorageResult<Option<BackupInfo>> {
        let metadata = fs::metadata(path).await
            .map_err(|e| StorageError::BackupError(format!("Failed to read file metadata: {}", e)))?;

        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| StorageError::BackupError("Invalid backup filename".to_string()))?;

        if !filename.starts_with(&self.config.backup_prefix) {
            return Ok(None);
        }

        let content = fs::read(path).await
            .map_err(|e| StorageError::BackupError(format!("Failed to read backup: {}", e)))?;
        
        let hash = sha256::digest(&content);
        let compressed = path.extension().map(|ext| ext == "gz").unwrap_or(false);

        Ok(Some(BackupInfo {
            id: filename.to_string(),
            created_at: DateTime::from(metadata.created().unwrap_or(UNIX_EPOCH)),
            size: metadata.len(),
            path: path.to_path_buf(),
            compressed,
            hash,
        }))
    }

    /// Clean up old backups exceeding max_backups
    async fn cleanup_old_backups(&self) -> StorageResult<()> {
        let mut backups = self.list_backups().await?;
        if backups.len() <= self.config.max_backups {
            return Ok(());
        }

        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        for backup in backups.iter().skip(self.config.max_backups) {
            fs::remove_file(&backup.path).await
                .map_err(|e| StorageError::BackupError(format!("Failed to remove old backup: {}", e)))?;
        }

        Ok(())
    }

    /// Compress a file using gzip
    async fn compress_file(&self, input: &Path, output: &Path) -> StorageResult<()> {
        use tokio::io::AsyncReadExt;
        use flate2::write::GzEncoder;
        use flate2::Compression;
        
        let mut input_file = fs::File::open(input).await
            .map_err(|e| StorageError::BackupError(format!("Failed to open file for compression: {}", e)))?;
        
        let mut buffer = Vec::new();
        input_file.read_to_end(&mut buffer).await
            .map_err(|e| StorageError::BackupError(format!("Failed to read file for compression: {}", e)))?;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&buffer)
            .map_err(|e| StorageError::BackupError(format!("Failed to compress data: {}", e)))?;
        
        let compressed_data = encoder.finish()
            .map_err(|e| StorageError::BackupError(format!("Failed to finish compression: {}", e)))?;

        fs::write(output, compressed_data).await
            .map_err(|e| StorageError::BackupError(format!("Failed to write compressed file: {}", e)))?;

        Ok(())
    }

    /// Decompress a gzip file
    async fn decompress_file(&self, input: &Path, output: &Path) -> StorageResult<()> {
        use tokio::io::AsyncReadExt;
        use flate2::read::GzDecoder;
        
        let input_data = fs::read(input).await
            .map_err(|e| StorageError::BackupError(format!("Failed to read compressed file: {}", e)))?;

        let mut decoder = GzDecoder::new(&input_data[..]);
        let mut decompressed_data = Vec::new();
        std::io::Read::read_to_end(&mut decoder, &mut decompressed_data)
            .map_err(|e| StorageError::BackupError(format!("Failed to decompress data: {}", e)))?;

        fs::write(output, decompressed_data).await
            .map_err(|e| StorageError::BackupError(format!("Failed to write decompressed file: {}", e)))?;

        Ok(())
    }

    /// Get database URL for pg_dump and pg_restore
    fn get_database_url(&self) -> String {
        // Implementation should get this from configuration
        // For security, we don't want to expose this in logs
        "postgresql://localhost/icn".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_backup_creation() {
        let temp_dir = tempdir().unwrap();
        let config = BackupConfig {
            backup_dir: temp_dir.path().to_path_buf(),
            max_backups: 3,
            backup_prefix: "test_backup".to_string(),
            compress: true,
        };

        let pool = PgPool::connect("postgresql://localhost/icn_test")
            .await
            .unwrap();
        let metrics = MetricsManager::new(pool.clone());
        let backup_manager = BackupManager::new(pool, config, metrics);

        // Create a backup
        let backup_info = backup_manager.create_backup().await.unwrap();
        assert!(backup_info.path.exists());
        assert!(backup_info.compressed);
        assert!(backup_info.size > 0);
    }

    #[tokio::test]
    async fn test_backup_rotation() {
        let temp_dir = tempdir().unwrap();
        let config = BackupConfig {
            backup_dir: temp_dir.path().to_path_buf(),
            max_backups: 2,
            backup_prefix: "test_backup".to_string(),
            compress: false,
        };

        let pool = PgPool::connect("postgresql://localhost/icn_test")
            .await
            .unwrap();
        let metrics = MetricsManager::new(pool.clone());
        let backup_manager = BackupManager::new(pool, config.clone(), metrics);

        // Create 3 backups (exceeding max_backups)
        for _ in 0..3 {
            backup_manager.create_backup().await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }

        let backups = backup_manager.list_backups().await.unwrap();
        assert_eq!(backups.len(), config.max_backups);
    }

    #[tokio::test]
    async fn test_backup_restore() {
        let temp_dir = tempdir().unwrap();
        let config = BackupConfig {
            backup_dir: temp_dir.path().to_path_buf(),
            max_backups: 3,
            backup_prefix: "test_backup".to_string(),
            compress: true,
        };

        let pool = PgPool::connect("postgresql://localhost/icn_test")
            .await
            .unwrap();
        let metrics = MetricsManager::new(pool.clone());
        let backup_manager = BackupManager::new(pool, config, metrics);

        // Create a backup and then restore from it
        let backup_info = backup_manager.create_backup().await.unwrap();
        assert!(backup_manager.restore_backup(&backup_info).await.is_ok());
    }

    #[tokio::test]
    async fn test_compression() {
        let temp_dir = tempdir().unwrap();
        let test_data = b"test data for compression";
        let input_path = temp_dir.path().join("test.txt");
        let compressed_path = temp_dir.path().join("test.txt.gz");
        let decompressed_path = temp_dir.path().join("test_restored.txt");

        // Write test data
        fs::write(&input_path, test_data).await.unwrap();

        let pool = PgPool::connect("postgresql://localhost/icn_test")
            .await
            .unwrap();
        let metrics = MetricsManager::new(pool.clone());
        let backup_manager = BackupManager::new(
            pool,
            BackupConfig::default(),
            metrics
        );

        // Test compression
        backup_manager.compress_file(&input_path, &compressed_path).await.unwrap();
        assert!(compressed_path.exists());

        // Test decompression
        backup_manager.decompress_file(&compressed_path, &decompressed_path).await.unwrap();
        assert!(decompressed_path.exists());

        // Verify data integrity
        let restored_data = fs::read(&decompressed_path).await.unwrap();
        assert_eq!(restored_data, test_data);
    }
}