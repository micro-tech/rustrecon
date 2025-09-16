use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{Row, SqlitePool};
use std::path::Path;

use crate::llm_client::FlaggedPattern;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedScanResult {
    pub id: i64,
    pub package_name: String,
    pub package_version: String,
    pub content_hash: String,
    pub analysis: String,
    pub flagged_patterns: Vec<FlaggedPattern>,
    pub scan_date: DateTime<Utc>,
    pub llm_model: String,
    pub cache_hit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageCacheEntry {
    pub package_name: String,
    pub package_version: String,
    pub content_hash: String,
    pub analysis: String,
    pub flagged_patterns_json: String,
    pub scan_date: DateTime<Utc>,
    pub llm_model: String,
}

pub struct ScanDatabase {
    pool: SqlitePool,
}

impl ScanDatabase {
    /// Initialize the database connection and create tables if they don't exist
    pub async fn new(database_path: &Path) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = database_path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create database directory")?;
        }

        let database_url = format!("sqlite://{}", database_path.display());
        let pool = SqlitePool::connect(&database_url)
            .await
            .context("Failed to connect to SQLite database")?;

        let db = Self { pool };
        db.initialize_tables().await?;
        Ok(db)
    }

    /// Create the necessary tables for scan result caching
    async fn initialize_tables(&self) -> Result<()> {
        // Create scan results table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS scan_results (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                package_name TEXT NOT NULL,
                package_version TEXT NOT NULL,
                content_hash TEXT NOT NULL,
                analysis TEXT NOT NULL,
                flagged_patterns_json TEXT NOT NULL,
                scan_date DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                llm_model TEXT NOT NULL DEFAULT 'gemini-1.5-flash',
                UNIQUE(package_name, package_version, content_hash)
            );
            "#,
        )
        .execute(&self.pool)
        .await
        .context("Failed to create scan_results table")?;

        // Create index for fast lookups
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_package_lookup
            ON scan_results(package_name, package_version, content_hash);
            "#,
        )
        .execute(&self.pool)
        .await
        .context("Failed to create package lookup index")?;

        // Create cache statistics table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS cache_stats (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                scan_date DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                total_packages INTEGER NOT NULL DEFAULT 0,
                cache_hits INTEGER NOT NULL DEFAULT 0,
                new_scans INTEGER NOT NULL DEFAULT 0,
                api_calls_saved INTEGER NOT NULL DEFAULT 0
            );
            "#,
        )
        .execute(&self.pool)
        .await
        .context("Failed to create cache_stats table")?;

        Ok(())
    }

    /// Generate a hash for package content to detect changes
    pub fn generate_content_hash(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Check if we have a cached result for this package
    pub async fn get_cached_result(
        &self,
        package_name: &str,
        package_version: &str,
        content_hash: &str,
    ) -> Result<Option<CachedScanResult>> {
        let row = sqlx::query(
            r#"
            SELECT id, package_name, package_version, content_hash, analysis,
                   flagged_patterns_json, scan_date, llm_model
            FROM scan_results
            WHERE package_name = ? AND package_version = ? AND content_hash = ?
            ORDER BY scan_date DESC
            LIMIT 1
            "#,
        )
        .bind(package_name)
        .bind(package_version)
        .bind(content_hash)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to query cached scan result")?;

        if let Some(row) = row {
            let flagged_patterns_json: String = row.get("flagged_patterns_json");
            let flagged_patterns: Vec<FlaggedPattern> =
                serde_json::from_str(&flagged_patterns_json)
                    .context("Failed to deserialize flagged patterns")?;

            Ok(Some(CachedScanResult {
                id: row.get("id"),
                package_name: row.get("package_name"),
                package_version: row.get("package_version"),
                content_hash: row.get("content_hash"),
                analysis: row.get("analysis"),
                flagged_patterns,
                scan_date: row.get("scan_date"),
                llm_model: row.get("llm_model"),
                cache_hit: true,
            }))
        } else {
            Ok(None)
        }
    }

    /// Store a new scan result in the cache
    pub async fn store_scan_result(
        &self,
        package_name: &str,
        package_version: &str,
        content_hash: &str,
        analysis: &str,
        flagged_patterns: &[FlaggedPattern],
        llm_model: &str,
    ) -> Result<i64> {
        let flagged_patterns_json = serde_json::to_string(flagged_patterns)
            .context("Failed to serialize flagged patterns")?;

        let result = sqlx::query(
            r#"
            INSERT OR REPLACE INTO scan_results
            (package_name, package_version, content_hash, analysis, flagged_patterns_json, scan_date, llm_model)
            VALUES (?, ?, ?, ?, ?, CURRENT_TIMESTAMP, ?)
            "#,
        )
        .bind(package_name)
        .bind(package_version)
        .bind(content_hash)
        .bind(analysis)
        .bind(flagged_patterns_json)
        .bind(llm_model)
        .execute(&self.pool)
        .await
        .context("Failed to store scan result")?;

        Ok(result.last_insert_rowid())
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> Result<CacheStats> {
        let total_entries: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM scan_results")
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);

        let recent_scans: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM scan_results WHERE scan_date > datetime('now', '-7 days')",
        )
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);

        Ok(CacheStats {
            total_cached_entries: total_entries,
            recent_scans_7_days: recent_scans,
        })
    }

    /// Clean old cache entries (older than specified days)
    pub async fn cleanup_old_entries(&self, max_age_days: u32) -> Result<u64> {
        let result = sqlx::query(
            "DELETE FROM scan_results WHERE scan_date < datetime('now', '-' || ? || ' days')",
        )
        .bind(max_age_days)
        .execute(&self.pool)
        .await
        .context("Failed to cleanup old cache entries")?;

        Ok(result.rows_affected())
    }

    /// Get most scanned packages (for statistics)
    pub async fn get_popular_packages(&self, limit: i32) -> Result<Vec<PackageStats>> {
        let rows = sqlx::query(
            r#"
            SELECT package_name, COUNT(*) as scan_count,
                   MAX(scan_date) as last_scan_date
            FROM scan_results
            GROUP BY package_name
            ORDER BY scan_count DESC
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .context("Failed to get popular packages")?;

        let mut packages = Vec::new();
        for row in rows {
            packages.push(PackageStats {
                package_name: row.get("package_name"),
                scan_count: row.get("scan_count"),
                last_scan_date: row.get("last_scan_date"),
            });
        }

        Ok(packages)
    }

    /// Record cache statistics for a scan session
    pub async fn record_scan_session(
        &self,
        total_packages: i32,
        cache_hits: i32,
        new_scans: i32,
    ) -> Result<()> {
        let api_calls_saved = cache_hits;

        sqlx::query(
            r#"
            INSERT INTO cache_stats (total_packages, cache_hits, new_scans, api_calls_saved)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(total_packages)
        .bind(cache_hits)
        .bind(new_scans)
        .bind(api_calls_saved)
        .execute(&self.pool)
        .await
        .context("Failed to record scan session stats")?;

        Ok(())
    }

    /// Export cache data for backup or analysis
    pub async fn export_cache(&self) -> Result<Vec<PackageCacheEntry>> {
        let rows = sqlx::query(
            r#"
            SELECT package_name, package_version, content_hash, analysis,
                   flagged_patterns_json, scan_date, llm_model
            FROM scan_results
            ORDER BY scan_date DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to export cache data")?;

        let mut entries = Vec::new();
        for row in rows {
            entries.push(PackageCacheEntry {
                package_name: row.get("package_name"),
                package_version: row.get("package_version"),
                content_hash: row.get("content_hash"),
                analysis: row.get("analysis"),
                flagged_patterns_json: row.get("flagged_patterns_json"),
                scan_date: row.get("scan_date"),
                llm_model: row.get("llm_model"),
            });
        }

        Ok(entries)
    }

    /// Close the database connection
    pub async fn close(&self) {
        self.pool.close().await;
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_cached_entries: i64,
    pub recent_scans_7_days: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageStats {
    pub package_name: String,
    pub scan_count: i64,
    pub last_scan_date: DateTime<Utc>,
}

/// Configuration for database caching
#[derive(Debug, Serialize, Deserialize)]
pub struct CacheConfig {
    pub enabled: bool,
    pub database_path: Option<String>,
    pub max_age_days: Option<u32>,
    pub auto_cleanup: Option<bool>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            database_path: None, // Will use default location
            max_age_days: Some(90), // Keep cache for 3 months
            auto_cleanup: Some(true),
        }
    }
}
