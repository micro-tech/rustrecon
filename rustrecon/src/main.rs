use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use std::time::Duration;

mod cached_llm_client;
mod cli;
mod config;
mod database;
mod dependency_scanner;
mod llm_client;
mod report;
mod scanner;
mod utils;

use cached_llm_client::CachedLlmClient;
use cli::{Cli, Commands};
use config::Config;
use database::ScanDatabase;
use dependency_scanner::DependencyScanner;
use llm_client::{GeminiClient, LlmClientTrait, LlmRequest};
use report::RiskReport;
use scanner::Scanner;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Init { config_path }) => {
            let config_file_path = if let Some(path) = config_path {
                PathBuf::from(path)
            } else {
                Config::get_default_config_path()?
            };

            println!(
                "Initializing configuration file at: {}",
                config_file_path.display()
            );
            Config::generate_default_config(config_file_path.clone())?;
            println!("Default configuration written successfully.");
            println!("Edit the file and add your Gemini API key to get started!");
        }
        Some(Commands::Test) => {
            println!("🔍 Testing LLM API connection...");

            // Load configuration
            let config = Config::load_from_default_paths()?;
            let llm_config = config.llm.ok_or_else(|| {
                anyhow::anyhow!("LLM configuration not found. Please run 'init' first and configure your API key.")
            })?;

            // Initialize LLM client
            let model = llm_config
                .gemini_model
                .clone()
                .unwrap_or_else(|| "gemini-2.5-flash".to_string());
            let gemini_client = GeminiClient::new(
                llm_config.gemini_api_key.clone(),
                llm_config.gemini_api_endpoint.clone(),
                model.clone(),
            );

            // Wrap with caching layer
            let cache_config = config.cache.unwrap_or_default();
            let mut cached_client =
                CachedLlmClient::new(gemini_client, cache_config, model).await?;

            // Simple test request
            let test_request = LlmRequest {
                prompt: "Hello! Please respond with 'API test successful' to confirm the connection is working.".to_string(),
            };

            match cached_client.analyze_code(test_request).await {
                Ok(response) => {
                    println!("✅ API connection successful!");
                    println!("📋 Test response: {}", response.analysis);
                    if !response.flagged_patterns.is_empty() {
                        println!("🔍 Found {} test patterns", response.flagged_patterns.len());
                    }
                    println!("\n🎉 Your Gemini API is configured correctly!");
                    println!("   You can now run: cargo run -- scan . -o report.md");
                }
                Err(e) => {
                    println!("❌ API test failed: {}", e);
                    println!("\n💡 Check your configuration:");
                    println!("   1. Verify your API key in rustrecon_config.toml");
                    println!("   2. Ensure internet connectivity");
                    println!("   3. Check if you've exceeded rate limits");
                    if llm_config.gemini_api_key.starts_with("PASTE_")
                        || llm_config.gemini_api_key.len() < 20
                    {
                        println!(
                            "   4. Your API key looks like a placeholder - please set a real key"
                        );
                    }
                }
            }
        }
        Some(Commands::Scan {
            crate_path,
            format,
            output,
            scan_dependencies,
            skip_dependencies,
        }) => {
            println!("Scanning crate: {}", crate_path);
            println!("Output format: {}", format);
            if let Some(out_path) = output {
                println!("Output file: {}", out_path);
            }

            // Load configuration
            let config = Config::load_from_default_paths()?;
            let llm_config = config.llm.ok_or_else(|| {
                anyhow::anyhow!("LLM configuration not found. Please run `init` or provide config.")
            })?;

            // Initialize LLM client with rate limiting
            let rate_limit_config = config.rate_limiting.as_ref();
            let min_interval = if let Some(rate_config) = rate_limit_config {
                if rate_config.enable_rate_limiting.unwrap_or(true) {
                    Duration::from_secs_f32(rate_config.min_request_interval_seconds.unwrap_or(2.0))
                } else {
                    Duration::from_millis(100) // Minimal delay if disabled
                }
            } else {
                Duration::from_secs(2) // Default
            };

            println!(
                "⚠️  Rate limiting enabled: {:.1}s between API requests to avoid hitting limits",
                min_interval.as_secs_f32()
            );

            let model = llm_config
                .gemini_model
                .clone()
                .unwrap_or_else(|| "gemini-2.5-flash".to_string());
            let gemini_client = GeminiClient::with_rate_limit(
                llm_config.gemini_api_key,
                llm_config.gemini_api_endpoint,
                model.clone(),
                min_interval,
            );

            // Initialize cached LLM client
            let cache_config = config.cache.unwrap_or_default();
            let mut cached_client =
                CachedLlmClient::new(gemini_client, cache_config, model).await?;

            // Initialize scanners
            let project_path = PathBuf::from(crate_path);
            let mut scanner = Scanner::new(project_path.clone())?;
            let file_analysis_results = scanner.scan_crate()?;

            let mut risk_report =
                RiskReport::new(crate::utils::get_crate_name_from_path(&project_path));

            // Scan dependencies if enabled
            let should_scan_deps = *scan_dependencies && !skip_dependencies;
            if should_scan_deps {
                println!("🔍 Starting dependency analysis for supply chain security...");
                let dependency_scanner = DependencyScanner::new();
                match dependency_scanner
                    .scan_dependencies(&project_path, &mut cached_client)
                    .await
                {
                    Ok(dependency_results) => {
                        println!(
                            "✅ Dependency scan completed. Found {} dependencies.",
                            dependency_results.len()
                        );
                        risk_report.add_dependency_findings(dependency_results);
                    }
                    Err(e) => {
                        eprintln!("⚠️  Dependency scan failed: {}", e);
                        println!("   Continuing with code-only analysis...");
                    }
                }
            } else {
                println!("⏭️  Skipping dependency scan (disabled)");
            }

            let total_files = file_analysis_results.len();
            println!(
                "📁 Analyzing {} code files (this may take a while due to API rate limiting)...",
                total_files
            );
            for (index, file_result) in file_analysis_results.into_iter().enumerate() {
                println!(
                    "📄 [{}/{}] Analyzing file: {}",
                    index + 1,
                    total_files,
                    file_result.path.display()
                );

                // Placeholder for actual LLM interaction
                let prompt = format!(
                    "Analyze the following Rust code for malicious behavior, backdoors, or unsafe patterns. Provide a summary of findings and specific flagged lines with severity (High, Medium, Low) and a brief description:\n\n{}",
                    file_result.content
                );
                let llm_request = LlmRequest { prompt };

                match cached_client.analyze_code(llm_request).await {
                    Ok(llm_response) => {
                        println!(
                            "LLM Analysis for {}: {}",
                            file_result.path.display(),
                            llm_response.analysis
                        );
                        risk_report.add_file_finding(
                            file_result.path.clone(),
                            llm_response.analysis,
                            llm_response.flagged_patterns,
                        );
                    }
                    Err(e) => {
                        eprintln!(
                            "Error calling LLM for {}: {}",
                            file_result.path.display(),
                            e
                        );
                        // Add an empty finding or a finding indicating an error
                        risk_report.add_file_finding(
                            file_result.path.clone(),
                            format!("LLM analysis failed: {}", e),
                            vec![],
                        );
                    }
                }
            }

            let output_path = output.as_ref().map(PathBuf::from);
            risk_report.generate_report(format, output_path.as_deref())?;

            // Show cache performance summary
            cached_client.print_cache_summary().await;

            // Record session statistics
            if let Err(e) = cached_client.record_session_stats(total_files as u32).await {
                println!("⚠️  Failed to record cache statistics: {}", e);
            }

            // Clean up old cache entries if auto-cleanup is enabled
            if let Ok(stats) = cached_client.get_cache_statistics().await {
                if stats.cache_hits > 0 {
                    println!(
                        "💡 Tip: Cache saved {} API calls this session!",
                        stats.cache_hits
                    );
                }
            }

            println!("Scan complete. Report generated.");
        }
        Some(Commands::Cache {
            clear,
            stats,
            export,
        }) => {
            // Load configuration for cache access
            let config = Config::load_from_default_paths()?;
            let cache_config = config.cache.unwrap_or_default();

            // Initialize database connection
            let db_path = if let Some(path) = &cache_config.database_path {
                PathBuf::from(path)
            } else {
                let mut default_path = dirs::data_dir()
                    .or_else(|| dirs::config_dir())
                    .unwrap_or_else(|| PathBuf::from("."));
                default_path.push("rustrecon");
                default_path.push("scan_cache.db");
                default_path
            };

            println!("📊 RustRecon Cache Management");
            println!("Database: {}", db_path.display());

            if *clear {
                println!("\n🗑️ Clearing all cached scan results...");
                if let Ok(database) = ScanDatabase::new(&db_path).await {
                    match database.cleanup_old_entries(0).await {
                        Ok(deleted) => println!("✅ Cleared {} cached entries", deleted),
                        Err(e) => println!("❌ Failed to clear cache: {}", e),
                    }
                } else {
                    println!("❌ Could not access cache database");
                }
            }

            if *stats || (!clear && export.is_none()) {
                println!("\n📈 Cache Statistics:");
                if let Ok(database) = ScanDatabase::new(&db_path).await {
                    match database.get_cache_stats().await {
                        Ok(stats) => {
                            println!("   Total cached entries: {}", stats.total_cached_entries);
                            println!("   Recent scans (7 days): {}", stats.recent_scans_7_days);

                            // Show popular packages
                            if let Ok(popular) = database.get_popular_packages(5).await {
                                println!("\n📦 Most Scanned Packages:");
                                for pkg in popular {
                                    println!(
                                        "   {} ({} scans, last: {})",
                                        pkg.package_name,
                                        pkg.scan_count,
                                        pkg.last_scan_date.format("%Y-%m-%d %H:%M")
                                    );
                                }
                            }
                        }
                        Err(e) => println!("   ❌ Failed to get statistics: {}", e),
                    }
                } else {
                    println!("   No cache database found or accessible");
                }
            }

            if let Some(export_path) = export {
                println!("\n📤 Exporting cache data to: {}", export_path);
                if let Ok(database) = ScanDatabase::new(&db_path).await {
                    match database.export_cache().await {
                        Ok(data) => {
                            let json_data = serde_json::to_string_pretty(&data)?;
                            std::fs::write(export_path, json_data)?;
                            println!("✅ Cache data exported successfully");
                        }
                        Err(e) => println!("❌ Failed to export cache: {}", e),
                    }
                } else {
                    println!("❌ Could not access cache database");
                }
            }
        }
        None => {
            // If no subcommand is provided, print help
            use clap::CommandFactory;
            let mut cmd = Cli::command();
            cmd.print_help()?;
        }
    }

    Ok(())
}
