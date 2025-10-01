use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::config::CacheConfig;
use crate::database::ScanDatabase;
use crate::llm_client::{LlmClientError, LlmClientTrait, LlmRequest, LlmResponse};

pub struct CachedLlmClient<T: LlmClientTrait + Send> {
    inner_client: T,
    database: Option<ScanDatabase>,
    cache_config: CacheConfig,
    cache_hits: u32,
    cache_misses: u32,
    llm_model: String,
}

impl<T: LlmClientTrait + Send> CachedLlmClient<T> {
    /// Create a new cached LLM client wrapper
    pub async fn new(
        inner_client: T,
        cache_config: CacheConfig,
        llm_model: String,
    ) -> Result<Self> {
        let database = if cache_config.enabled.unwrap_or(true) {
            let db_path = if let Some(path) = &cache_config.database_path {
                PathBuf::from(path)
            } else {
                // Use default location in user's local data directory
                let mut default_path = dirs::data_local_dir()
                    .or_else(|| dirs::data_dir())
                    .unwrap_or_else(|| PathBuf::from("."));
                default_path.push("RustRecon");
                default_path.push("scan_cache.db");
                default_path
            };

            println!(
                "📂 Initializing scan cache database at: {}",
                db_path.display()
            );
            match ScanDatabase::new(&db_path).await {
                Ok(db) => {
                    println!("✅ Cache database initialized");
                    Some(db)
                }
                Err(e) => {
                    println!("⚠️  Failed to initialize cache database: {}", e);
                    println!("   Continuing without cache...");
                    None
                }
            }
        } else {
            println!("📊 Database caching disabled in configuration");
            None
        };

        Ok(Self {
            inner_client,
            database,
            cache_config,
            cache_hits: 0,
            cache_misses: 0,
            llm_model,
        })
    }

    /// Analyze code with caching support for packages
    pub async fn analyze_package(
        &mut self,
        package_name: &str,
        package_version: &str,
        content: &str,
        request: LlmRequest,
    ) -> Result<LlmResponse, LlmClientError> {
        // Generate content hash for cache lookup
        let content_hash = if let Some(ref _db) = self.database {
            ScanDatabase::generate_content_hash(content)
        } else {
            String::new()
        };

        // Try to get cached result first
        if let Some(ref db) = self.database {
            match db
                .get_cached_result(package_name, package_version, &content_hash)
                .await
            {
                Ok(Some(cached_result)) => {
                    self.cache_hits += 1;
                    println!(
                        "  💾 Cache HIT for {} v{} (saved API call!)",
                        package_name, package_version
                    );

                    return Ok(LlmResponse {
                        analysis: cached_result.analysis,
                        flagged_patterns: cached_result.flagged_patterns,
                    });
                }
                Ok(None) => {
                    // Cache miss, proceed with LLM call
                    self.cache_misses += 1;
                }
                Err(e) => {
                    println!("⚠️  Cache lookup failed: {}", e);
                    self.cache_misses += 1;
                }
            }
        }

        // Cache miss or no cache - call the actual LLM
        println!(
            "  🔍 Cache MISS for {} v{} - calling LLM...",
            package_name, package_version
        );
        let response = self.inner_client.analyze_code(request).await?;

        // Store result in cache for future use
        if let Some(ref db) = self.database {
            if let Err(e) = db
                .store_scan_result(
                    package_name,
                    package_version,
                    &content_hash,
                    &response.analysis,
                    &response.flagged_patterns,
                    &self.llm_model,
                )
                .await
            {
                println!("⚠️  Failed to cache scan result: {}", e);
            } else {
                println!(
                    "  💾 Cached result for {} v{}",
                    package_name, package_version
                );
            }
        }

        Ok(response)
    }

    /// Get cache statistics
    pub async fn get_cache_statistics(&self) -> Result<CacheStatistics> {
        let mut stats = CacheStatistics {
            cache_hits: self.cache_hits,
            cache_misses: self.cache_misses,
            hit_rate: if (self.cache_hits + self.cache_misses) > 0 {
                (self.cache_hits as f32 / (self.cache_hits + self.cache_misses) as f32) * 100.0
            } else {
                0.0
            },
            total_cached_entries: 0,
            recent_scans: 0,
            api_calls_saved: self.cache_hits,
        };

        if let Some(ref db) = self.database {
            if let Ok(db_stats) = db.get_cache_stats().await {
                stats.total_cached_entries = db_stats.total_cached_entries as u32;
                stats.recent_scans = db_stats.recent_scans_7_days as u32;
            }
        }

        Ok(stats)
    }

    /// Clean up old cache entries
    pub async fn cleanup_cache(&self) -> Result<u32> {
        if let Some(ref db) = self.database {
            let max_age = self.cache_config.max_age_days.unwrap_or(90);
            let deleted = db.cleanup_old_entries(max_age).await?;
            println!(
                "🧹 Cleaned up {} old cache entries (older than {} days)",
                deleted, max_age
            );
            Ok(deleted as u32)
        } else {
            Ok(0)
        }
    }

    /// Record scan session statistics
    pub async fn record_session_stats(&self, total_packages: u32) -> Result<()> {
        if let Some(ref db) = self.database {
            db.record_scan_session(
                total_packages as i32,
                self.cache_hits as i32,
                self.cache_misses as i32,
            )
            .await?;
        }
        Ok(())
    }

    /// Print cache summary
    pub async fn print_cache_summary(&self) {
        if let Ok(stats) = self.get_cache_statistics().await {
            println!("\n📊 Cache Performance Summary:");
            println!("   💾 Cache Hits: {} (API calls saved)", stats.cache_hits);
            println!("   🔍 Cache Misses: {} (new LLM calls)", stats.cache_misses);
            println!("   📈 Hit Rate: {:.1}%", stats.hit_rate);
            println!(
                "   🗂️  Total Cached Entries: {}",
                stats.total_cached_entries
            );

            if stats.api_calls_saved > 0 {
                let time_saved = stats.api_calls_saved as f32 * 2.0; // Assuming 2s per call
                println!("   ⏱️  Estimated Time Saved: {:.0} seconds", time_saved);
            }
        }
    }

    /// Export cache data for backup
    pub async fn export_cache(&self) -> Result<String> {
        if let Some(ref db) = self.database {
            let entries = db.export_cache().await?;
            serde_json::to_string_pretty(&entries).context("Failed to serialize cache data")
        } else {
            Ok("[]".to_string())
        }
    }
}

#[async_trait::async_trait]
impl<T: LlmClientTrait + Send> LlmClientTrait for CachedLlmClient<T> {
    async fn analyze_code(&mut self, request: LlmRequest) -> Result<LlmResponse, LlmClientError> {
        // For generic code analysis (not package-specific), we can't use caching
        // as we don't have package name/version context
        self.inner_client.analyze_code(request).await
    }
}

#[derive(Debug)]
pub struct CacheStatistics {
    pub cache_hits: u32,
    pub cache_misses: u32,
    pub hit_rate: f32,
    pub total_cached_entries: u32,
    pub recent_scans: u32,
    pub api_calls_saved: u32,
}

impl std::fmt::Display for CacheStatistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cache Stats: {}/{} hits ({:.1}%), {} entries, {} API calls saved",
            self.cache_hits,
            self.cache_hits + self.cache_misses,
            self.hit_rate,
            self.total_cached_entries,
            self.api_calls_saved
        )
    }
}
