use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
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

pub struct RusqliteDatabase {
    conn: rusqlite::Connection,
}

impl RusqliteDatabase {
    /// Initialize the database connection and create tables if they don't exist
    pub fn new(database_path: &Path) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = database_path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create database directory")?;
        }

        println!("📂 Opening database at: {}", database_path.display());
        let conn = rusqlite::Connection::open(database_path).with_context(|| {
            format!(
                "Failed to open SQLite database at: {}",
                database_path.display()
            )
        })?;

        let db = Self { conn };
        db.initialize_tables()?;
        println!("✅ Database initialized successfully");
        Ok(db)
    }

    /// Create the necessary tables for scan result caching
    fn initialize_tables(&self) -> Result<()> {
        // Create scan results table
        self.conn
            .execute(
                r#"
            CREATE TABLE IF NOT EXISTS scan_results (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                package_name TEXT NOT NULL,
                package_version TEXT NOT NULL,
                content_hash TEXT NOT NULL,
                analysis TEXT NOT NULL,
                flagged_patterns_json TEXT NOT NULL,
                scan_date DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                llm_model TEXT NOT NULL DEFAULT 'gemini-2.5-flash',
                UNIQUE(package_name, package_version, content_hash)
            );
            "#,
                [],
            )
            .context("Failed to create scan_results table")?;

        // Create index for fast lookups
        self.conn
            .execute(
                r#"
            CREATE INDEX IF NOT EXISTS idx_package_lookup
            ON scan_results(package_name, package_version, content_hash);
            "#,
                [],
            )
            .context("Failed to create package lookup index")?;

        // Create cache statistics table
        self.conn
            .execute(
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
                [],
            )
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
    pub fn get_cached_result(
        &self,
        package_name: &str,
        package_version: &str,
        content_hash: &str,
    ) -> Result<Option<CachedScanResult>> {
        let mut stmt = self
            .conn
            .prepare(
                r#"
            SELECT id, package_name, package_version, content_hash, analysis,
                   flagged_patterns_json, scan_date, llm_model
            FROM scan_results
            WHERE package_name = ?1 AND package_version = ?2 AND content_hash = ?3
            ORDER BY scan_date DESC
            LIMIT 1
            "#,
            )
            .context("Failed to prepare query for cached scan result")?;

        let result = stmt.query_row(
            rusqlite::params![package_name, package_version, content_hash],
            |row| {
                let flagged_patterns_json: String = row.get("flagged_patterns_json")?;
                let flagged_patterns: Vec<FlaggedPattern> =
                    serde_json::from_str(&flagged_patterns_json).map_err(|e| {
                        rusqlite::Error::InvalidColumnType(
                            0,
                            "flagged_patterns_json".to_string(),
                            rusqlite::types::Type::Text,
                        )
                    })?;

                let scan_date_str: String = row.get("scan_date")?;
                let scan_date = DateTime::parse_from_rfc3339(&scan_date_str)
                    .map_err(|_| {
                        rusqlite::Error::InvalidColumnType(
                            0,
                            "scan_date".to_string(),
                            rusqlite::types::Type::Text,
                        )
                    })?
                    .with_timezone(&Utc);

                Ok(CachedScanResult {
                    id: row.get("id")?,
                    package_name: row.get("package_name")?,
                    package_version: row.get("package_version")?,
                    content_hash: row.get("content_hash")?,
                    analysis: row.get("analysis")?,
                    flagged_patterns,
                    scan_date,
                    llm_model: row.get("llm_model")?,
                    cache_hit: true,
                })
            },
        );

        match result {
            Ok(cached_result) => Ok(Some(cached_result)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e).context("Failed to query cached scan result"),
        }
    }

    /// Store a new scan result in the cache
    pub fn store_scan_result(
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

        let scan_date = Utc::now().to_rfc3339();

        let result = self.conn.execute(
            r#"
            INSERT OR REPLACE INTO scan_results
            (package_name, package_version, content_hash, analysis, flagged_patterns_json, scan_date, llm_model)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            rusqlite::params![
                package_name,
                package_version,
                content_hash,
                analysis,
                flagged_patterns_json,
                scan_date,
                llm_model
            ],
        )
        .context("Failed to store scan result")?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> Result<CacheStats> {
        let total_entries: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM scan_results", [], |row| row.get(0))
            .unwrap_or(0);

        let recent_scans: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM scan_results WHERE scan_date > datetime('now', '-7 days')",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        Ok(CacheStats {
            total_cached_entries: total_entries,
            recent_scans_7_days: recent_scans,
        })
    }

    /// Clean old cache entries (older than specified days)
    pub fn cleanup_old_entries(&self, max_age_days: u32) -> Result<u64> {
        let result = self
            .conn
            .execute(
                "DELETE FROM scan_results WHERE scan_date < datetime('now', '-' || ?1 || ' days')",
                rusqlite::params![max_age_days],
            )
            .context("Failed to cleanup old cache entries")?;

        Ok(result as u64)
    }

    /// Get most scanned packages (for statistics)
    pub fn get_popular_packages(&self, limit: i32) -> Result<Vec<PackageStats>> {
        let mut stmt = self
            .conn
            .prepare(
                r#"
            SELECT package_name, COUNT(*) as scan_count,
                   MAX(scan_date) as last_scan_date
            FROM scan_results
            GROUP BY package_name
            ORDER BY scan_count DESC
            LIMIT ?1
            "#,
            )
            .context("Failed to prepare popular packages query")?;

        let package_iter = stmt
            .query_map(rusqlite::params![limit], |row| {
                let last_scan_date_str: String = row.get("last_scan_date")?;
                let last_scan_date = DateTime::parse_from_rfc3339(&last_scan_date_str)
                    .map_err(|_| {
                        rusqlite::Error::InvalidColumnType(
                            0,
                            "last_scan_date".to_string(),
                            rusqlite::types::Type::Text,
                        )
                    })?
                    .with_timezone(&Utc);

                Ok(PackageStats {
                    package_name: row.get("package_name")?,
                    scan_count: row.get("scan_count")?,
                    last_scan_date,
                })
            })
            .context("Failed to execute popular packages query")?;

        let mut packages = Vec::new();
        for package in package_iter {
            packages.push(package.context("Failed to process package stats row")?);
        }

        Ok(packages)
    }

    /// Record cache statistics for a scan session
    pub fn record_scan_session(
        &self,
        total_packages: i32,
        cache_hits: i32,
        new_scans: i32,
    ) -> Result<()> {
        let api_calls_saved = cache_hits;
        let scan_date = Utc::now().to_rfc3339();

        self.conn.execute(
            r#"
            INSERT INTO cache_stats (scan_date, total_packages, cache_hits, new_scans, api_calls_saved)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
            rusqlite::params![scan_date, total_packages, cache_hits, new_scans, api_calls_saved],
        )
        .context("Failed to record scan session stats")?;

        Ok(())
    }

    /// Export cache data for backup or analysis
    pub fn export_cache(&self) -> Result<Vec<PackageCacheEntry>> {
        let mut stmt = self
            .conn
            .prepare(
                r#"
            SELECT package_name, package_version, content_hash, analysis,
                   flagged_patterns_json, scan_date, llm_model
            FROM scan_results
            ORDER BY scan_date DESC
            "#,
            )
            .context("Failed to prepare export query")?;

        let entry_iter = stmt
            .query_map([], |row| {
                let scan_date_str: String = row.get("scan_date")?;
                let scan_date = DateTime::parse_from_rfc3339(&scan_date_str)
                    .map_err(|_| {
                        rusqlite::Error::InvalidColumnType(
                            0,
                            "scan_date".to_string(),
                            rusqlite::types::Type::Text,
                        )
                    })?
                    .with_timezone(&Utc);

                Ok(PackageCacheEntry {
                    package_name: row.get("package_name")?,
                    package_version: row.get("package_version")?,
                    content_hash: row.get("content_hash")?,
                    analysis: row.get("analysis")?,
                    flagged_patterns_json: row.get("flagged_patterns_json")?,
                    scan_date,
                    llm_model: row.get("llm_model")?,
                })
            })
            .context("Failed to execute export query")?;

        let mut entries = Vec::new();
        for entry in entry_iter {
            entries.push(entry.context("Failed to process export entry")?);
        }

        Ok(entries)
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
