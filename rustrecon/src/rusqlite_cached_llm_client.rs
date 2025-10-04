use anyhow::Result;
use std::fs;
use std::path::PathBuf;

use crate::llm_client::{LlmClientError, LlmClientTrait, LlmRequest, LlmResponse};
use crate::rusqlite_database::{CacheStats, PackageStats, RusqliteDatabase};
use regex::Regex;

pub struct RusqliteCachedLlmClient<T: LlmClientTrait + Send> {
    inner_client: T,
    database: Option<RusqliteDatabase>,
    cache_hits: u32,
    cache_misses: u32,
    llm_model: String,
}

impl<T: LlmClientTrait + Send> RusqliteCachedLlmClient<T> {
    /// Create a new cached LLM client wrapper using rusqlite
    pub fn new(inner_client: T, llm_model: String) -> Result<Self> {
        let db_path = {
            // Use default location in user's local data directory
            let mut default_path = dirs::data_local_dir()
                .or_else(|| dirs::data_local_dir())
                .unwrap_or_else(|| PathBuf::from("."));
            default_path.push("RustRecon");
            default_path
        };

        // Create the directory if it doesn't exist
        if let Some(parent) = db_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        let db_path = if db_path.is_dir() {
            db_path.join("scan_cache.db")
        } else {
            db_path
        };

        println!(
            "📂 Initializing scan cache database at: {}",
            db_path.display()
        );
        let database = match RusqliteDatabase::new(&db_path) {
            Ok(db) => {
                println!("✅ Cache database initialized successfully");
                Some(db)
            }
            Err(e) => {
                println!("⚠️  Failed to initialize cache database: {}", e);
                println!("   Continuing without cache...");
                None
            }
        };

        Ok(Self {
            inner_client,
            database,
            cache_hits: 0,
            cache_misses: 0,
            llm_model,
        })
    }

    /// Extracts package name, version, and content from the prompt.
    /// For file-level analysis, uses file path as package name and content hash as version.
    fn extract_metadata_from_prompt(&self, prompt: &str) -> (String, String, String) {
        // First try to match dependency format: "Package: [name] v[version]"
        let package_re = Regex::new(r"Package: (.+) v(.+)").unwrap();
        if let Some(caps) = package_re.captures(prompt) {
            let package_name = caps.get(1).map_or("", |m| m.as_str()).to_string();
            let package_version = caps.get(2).map_or("", |m| m.as_str()).to_string();
            return (package_name, package_version, prompt.to_string());
        }

        // Try to match file analysis format: "File: [path]"
        // Look for file path patterns in the prompt
        let file_re = Regex::new(r"File:\s*([^\n]+\.rs)").unwrap();
        if let Some(caps) = file_re.captures(prompt) {
            let file_path = caps.get(1).map_or("", |m| m.as_str()).to_string();
            let content_hash = RusqliteDatabase::generate_content_hash(prompt);
            let short_hash = &content_hash[0..12]; // Use first 12 chars as version
            return (file_path, short_hash.to_string(), prompt.to_string());
        }

        // For any other content, use content hash as both name and version
        let content_hash = RusqliteDatabase::generate_content_hash(prompt);
        let short_hash = &content_hash[0..12];
        (
            format!("content_{}", short_hash),
            short_hash.to_string(),
            prompt.to_string(),
        )
    }

    /// Get cache statistics
    pub fn get_cache_statistics(&self) -> Result<CacheStatistics> {
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
            if let Ok(db_stats) = db.get_cache_stats() {
                stats.total_cached_entries = db_stats.total_cached_entries as u32;
                stats.recent_scans = db_stats.recent_scans_7_days as u32;
            }
        }

        Ok(stats)
    }

    /// Record scan session statistics
    pub fn record_session_stats(&self, total_packages: u32) -> Result<()> {
        if let Some(ref db) = self.database {
            db.record_scan_session(
                total_packages as i32,
                self.cache_hits as i32,
                self.cache_misses as i32,
            )?;
        }
        Ok(())
    }

    /// Print cache summary
    pub fn print_cache_summary(&self) {
        if let Ok(stats) = self.get_cache_statistics() {
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

    /// Get cache statistics from database
    pub fn get_cache_stats(&self) -> Result<CacheStats> {
        if let Some(ref db) = self.database {
            db.get_cache_stats()
        } else {
            Ok(CacheStats {
                total_cached_entries: 0,
                recent_scans_7_days: 0,
            })
        }
    }

    /// Get popular packages from database
    pub fn get_popular_packages(&self, limit: i32) -> Result<Vec<PackageStats>> {
        if let Some(ref db) = self.database {
            db.get_popular_packages(limit)
        } else {
            Ok(vec![])
        }
    }

    /// Clear all cached entries
    pub fn cleanup_old_entries(&self, max_age_days: u32) -> Result<u64> {
        if let Some(ref db) = self.database {
            db.cleanup_old_entries(max_age_days)
        } else {
            Ok(0)
        }
    }

    /// Export cache data
    pub fn export_cache(&self) -> Result<Vec<crate::rusqlite_database::PackageCacheEntry>> {
        if let Some(ref db) = self.database {
            db.export_cache()
        } else {
            Ok(vec![])
        }
    }
}

#[async_trait::async_trait]
impl<T: LlmClientTrait + Send> LlmClientTrait for RusqliteCachedLlmClient<T> {
    async fn analyze_code(&mut self, request: LlmRequest) -> Result<LlmResponse, LlmClientError> {
        // Extract metadata from the prompt
        let (package_name, package_version, content) =
            self.extract_metadata_from_prompt(&request.prompt);

        // Generate content hash for cache lookup
        let content_hash = RusqliteDatabase::generate_content_hash(&content);

        // Try to get cached result first
        if let Some(ref db) = self.database {
            if !package_name.is_empty() && !package_version.is_empty() {
                match db.get_cached_result(&package_name, &package_version, &content_hash) {
                    Ok(Some(cached_result)) => {
                        self.cache_hits += 1;
                        if package_name.starts_with("content_") {
                            println!("  💾 Cache HIT for content analysis (saved API call!)");
                        } else if package_name.ends_with(".rs") {
                            println!("  💾 Cache HIT for {} (saved API call!)", package_name);
                        } else {
                            println!(
                                "  💾 Cache HIT for {} v{} (saved API call!)",
                                package_name, package_version
                            );
                        }

                        return Ok(LlmResponse {
                            analysis: cached_result.analysis,
                            flagged_patterns: cached_result.flagged_patterns,
                        });
                    }
                    Ok(None) => {
                        self.cache_misses += 1;
                    }
                    Err(e) => {
                        println!("⚠️  Cache lookup failed: {}", e);
                        self.cache_misses += 1;
                    }
                }
            } else {
                // Even if metadata extraction failed, still increment cache misses
                self.cache_misses += 1;
            }
        }

        // Cache miss or no cache - call the actual LLM
        if !package_name.is_empty() {
            if package_name.starts_with("content_") {
                println!("  🔍 Cache MISS for content analysis - calling LLM...");
            } else if package_name.ends_with(".rs") {
                println!("  🔍 Cache MISS for {} - calling LLM...", package_name);
            } else {
                println!(
                    "  🔍 Cache MISS for {} v{} - calling LLM...",
                    package_name, package_version
                );
            }
        } else {
            println!("  🔍 Analyzing content - calling LLM...");
        }
        let response = self.inner_client.analyze_code(request).await?;

        // Store result in cache for future use
        if let Some(ref db) = self.database {
            // Always try to cache the result if we have any identifier
            let cache_package = if package_name.is_empty() {
                let hash = RusqliteDatabase::generate_content_hash(&content);
                format!("content_{}", &hash[0..12])
            } else {
                package_name.clone()
            };

            let cache_version = if package_version.is_empty() {
                let hash = RusqliteDatabase::generate_content_hash(&content);
                hash[0..12].to_string()
            } else {
                package_version.clone()
            };

            if let Err(e) = db.store_scan_result(
                &cache_package,
                &cache_version,
                &content_hash,
                &response.analysis,
                &response.flagged_patterns,
                &self.llm_model,
            ) {
                println!("⚠️  Failed to cache scan result: {}", e);
            } else {
                if cache_package.starts_with("content_") {
                    println!("  💾 Cached content analysis result");
                } else if cache_package.ends_with(".rs") {
                    println!("  💾 Cached result for {}", cache_package);
                } else {
                    println!(
                        "  💾 Cached result for {} v{}",
                        cache_package, cache_version
                    );
                }
            }
        }

        Ok(response)
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
