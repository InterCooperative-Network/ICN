use sqlx::{PgPool, Row};
use log::{info, warn};

// Index usage monitoring
pub async fn analyze_index_usage(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Query to get index usage statistics
    let index_stats = sqlx::query(r#"
        SELECT
            schemaname,
            relname AS table_name,
            indexrelname AS index_name,
            idx_scan AS scan_count,
            idx_tup_read AS tuples_read,
            idx_tup_fetch AS tuples_fetched
        FROM
            pg_stat_user_indexes
        ORDER BY
            idx_scan DESC
    "#)
    .fetch_all(pool)
    .await?;
    
    info!("Index usage statistics:");
    for stat in index_stats {
        let schema: String = stat.get("schemaname");
        let table: String = stat.get("table_name");
        let index: String = stat.get("index_name");
        let scans: i64 = stat.get("scan_count");
        
        if scans == 0 {
            warn!("Unused index: {}.{}.{}", schema, table, index);
        } else {
            info!("Index {}.{}.{} used {} times", schema, table, index, scans);
        }
    }
    
    Ok