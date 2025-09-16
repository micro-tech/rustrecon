use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

mod cli;
mod config;
mod dependency_scanner;
mod llm_client;
mod report;
mod scanner;
mod utils;

use cli::{Cli, Commands};
use config::Config;
use dependency_scanner::DependencyScanner;
use llm_client::{GeminiClient, LlmClientTrait, LlmRequest};
use report::RiskReport;
use scanner::Scanner;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Init { config_path }) => {
            println!("Initializing configuration file at: {}", config_path);
            Config::generate_default_config(PathBuf::from(config_path))?;
            println!("Default configuration written successfully.");
        }
        Some(Commands::Test) => {
            println!("🔍 Testing LLM API connection...");

            // Load configuration
            let config = Config::load_from_default_paths()?;
            let llm_config = config.llm.ok_or_else(|| {
                anyhow::anyhow!("LLM configuration not found. Please run 'init' first and configure your API key.")
            })?;

            // Initialize LLM client
            let gemini_client = GeminiClient::new(
                llm_config.gemini_api_key.clone(),
                llm_config.gemini_api_endpoint.clone(),
            );

            // Simple test request
            let test_request = LlmRequest {
                prompt: "Hello! Please respond with 'API test successful' to confirm the connection is working.".to_string(),
            };

            match gemini_client.analyze_code(test_request).await {
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

            // Initialize LLM client
            let gemini_client =
                GeminiClient::new(llm_config.gemini_api_key, llm_config.gemini_api_endpoint);

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
                    .scan_dependencies(&project_path, &gemini_client)
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

            for file_result in file_analysis_results {
                println!("Analyzing file: {}", file_result.path.display());

                // Placeholder for actual LLM interaction
                let prompt = format!(
                    "Analyze the following Rust code for malicious behavior, backdoors, or unsafe patterns. Provide a summary of findings and specific flagged lines with severity (High, Medium, Low) and a brief description:\n\n{}",
                    file_result.content
                );
                let llm_request = LlmRequest { prompt };

                match gemini_client.analyze_code(llm_request).await {
                    Ok(llm_response) => {
                        println!(
                            "LLM Analysis for {}: {}",
                            file_result.path.display(),
                            llm_response.analysis
                        );
                        risk_report.add_file_finding(
                            file_result.path,
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
                            file_result.path,
                            format!("LLM analysis failed: {}", e),
                            vec![],
                        );
                    }
                }
            }

            let output_path = output.as_ref().map(PathBuf::from);
            risk_report.generate_report(format, output_path.as_deref())?;

            println!("Scan complete. Report generated.");
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
