use anyhow::{bail, Result};
use cargo_metadata::{Metadata, MetadataCommand, Package};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use tokio::time::{sleep, timeout, Duration};

use crate::llm_client::{FlaggedPattern, LlmClientTrait, LlmRequest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyAnalysisResult {
    pub package_name: String,
    pub version: String,
    pub source: DependencySource,
    pub risk_score: RiskScore,
    pub suspicious_patterns: Vec<FlaggedPattern>,
    pub metadata_flags: Vec<MetadataFlag>,
    pub code_analysis: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencySource {
    CratesIo {
        registry_url: String,
    },
    Git {
        repository: String,
        rev: Option<String>,
    },
    Path {
        path: String,
    },
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskScore {
    Critical, // Definite malicious behavior detected
    High,     // Very suspicious patterns
    Medium,   // Some concerning patterns
    Low,      // Minor concerns
    Clean,    // No issues detected
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataFlag {
    pub flag_type: MetadataFlagType,
    pub description: String,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetadataFlagType {
    Typosquatting,
    RecentPublication,
    LowDownloads,
    SuspiciousAuthor,
    UnusualDependencies,
    NetworkingCapabilities,
    FileSystemAccess,
    ProcessExecution,
    CryptoOperations,
}

pub struct DependencyScanner {
    client: Client,
    known_malicious: HashSet<String>,
    popular_packages: HashMap<String, u64>, // package_name -> download_count
    trusted_packages: HashSet<String>,
}

impl DependencyScanner {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        // Initialize with known malicious packages and popular package names
        let mut known_malicious = HashSet::new();
        // Add known malicious packages (this would be updated regularly)
        known_malicious.insert("malicious-example".to_string());

        let mut popular_packages = HashMap::new();
        // Add popular packages for typosquatting detection
        popular_packages.insert("serde".to_string(), 100_000_000);
        popular_packages.insert("tokio".to_string(), 50_000_000);
        popular_packages.insert("clap".to_string(), 30_000_000);
        popular_packages.insert("reqwest".to_string(), 20_000_000);
        popular_packages.insert("anyhow".to_string(), 40_000_000);
        popular_packages.insert("thiserror".to_string(), 25_000_000);

        let mut trusted_packages = HashSet::new();
        // Add well-known trusted packages that don't need deep LLM analysis
        trusted_packages.insert("serde".to_string());
        trusted_packages.insert("tokio".to_string());
        trusted_packages.insert("clap".to_string());
        trusted_packages.insert("anyhow".to_string());
        trusted_packages.insert("thiserror".to_string());
        trusted_packages.insert("regex".to_string());
        trusted_packages.insert("chrono".to_string());
        trusted_packages.insert("uuid".to_string());
        trusted_packages.insert("log".to_string());
        trusted_packages.insert("once_cell".to_string());
        trusted_packages.insert("parking_lot".to_string());
        trusted_packages.insert("rayon".to_string());

        DependencyScanner {
            client,
            known_malicious,
            popular_packages,
            trusted_packages,
        }
    }

    pub async fn scan_dependencies<T: LlmClientTrait>(
        &self,
        project_path: &Path,
        llm_client: &mut T,
    ) -> Result<Vec<DependencyAnalysisResult>> {
        println!("🔍 Scanning dependencies for supply chain security...");

        // Get cargo metadata
        let metadata = self.get_cargo_metadata(project_path)?;
        let mut results = Vec::new();

        // Filter and prioritize dependencies for analysis
        let mut dependencies_to_analyze = Vec::new();
        let mut low_priority_deps = Vec::new();

        for package in &metadata.packages {
            // Skip the workspace packages (focus on external dependencies)
            let workspace_package_ids: Vec<_> = metadata
                .workspace_packages()
                .into_iter()
                .map(|wp| &wp.id)
                .collect();
            if workspace_package_ids.contains(&&package.id) {
                continue;
            }

            // Prioritize suspicious packages for LLM analysis
            if self.should_analyze_with_llm(&package.name) {
                dependencies_to_analyze.push(package);
            } else {
                low_priority_deps.push(package);
            }
        }

        println!(
            "📊 Found {} dependencies ({} high-priority for deep analysis)",
            dependencies_to_analyze.len() + low_priority_deps.len(),
            dependencies_to_analyze.len()
        );

        // Analyze high-priority dependencies with LLM (with rate limiting)
        for (i, package) in dependencies_to_analyze.iter().enumerate() {
            println!(
                "   🔍 Deep analysis [{}/{}]: {} v{}",
                i + 1,
                dependencies_to_analyze.len(),
                package.name,
                package.version
            );

            let analysis = self.analyze_dependency(package, llm_client).await?;
            results.push(analysis);

            // Rate limiting: sleep between requests to avoid quota issues
            if i < dependencies_to_analyze.len() - 1 {
                tokio::time::sleep(Duration::from_millis(4000)).await; // 4 second delay
            }
        }

        // Analyze low-priority dependencies without LLM (metadata only)
        for package in low_priority_deps {
            println!("   📦 Quick scan: {} v{}", package.name, package.version);
            let analysis = self.analyze_dependency_light(package).await?;
            results.push(analysis);
        }

        // Sort by risk score for reporting
        results.sort_by(|a, b| self.compare_risk_scores(&a.risk_score, &b.risk_score));

        Ok(results)
    }

    fn get_cargo_metadata(&self, project_path: &Path) -> Result<Metadata> {
        let mut cmd = MetadataCommand::new();
        cmd.manifest_path(project_path.join("Cargo.toml"));
        cmd.exec()
            .map_err(|e| anyhow::anyhow!("Failed to get cargo metadata: {}", e))
    }

    fn should_analyze_with_llm(&self, package_name: &str) -> bool {
        // Skip trusted packages to save API calls
        if self.trusted_packages.contains(package_name) {
            return false;
        }

        // Always analyze if potentially malicious
        if self.known_malicious.contains(package_name) {
            return true;
        }

        // Analyze packages with suspicious names
        if self.check_typosquatting(package_name).is_some() {
            return true;
        }

        // Analyze packages with suspicious patterns in name
        let suspicious_patterns = [
            "steal", "hack", "backdoor", "malware", "virus", "trojan", "keylog", "password",
            "credit", "bank", "wallet", "bitcoin", "mining", "miner", "crypto", "shell", "reverse",
            "payload",
        ];

        if suspicious_patterns
            .iter()
            .any(|&pattern| package_name.contains(pattern))
        {
            return true;
        }

        // For now, limit deep analysis to reduce API calls
        // In production, you might want more sophisticated filtering
        false
    }

    async fn analyze_dependency_light(
        &self,
        package: &Package,
    ) -> Result<DependencyAnalysisResult> {
        // Quick analysis without LLM - just metadata checks
        let source = self.determine_dependency_source(package);
        let metadata_flags = self.analyze_package_metadata(package).await?;
        let risk_score = self.calculate_risk_score(&metadata_flags, &[]);

        Ok(DependencyAnalysisResult {
            package_name: package.name.clone(),
            version: package.version.to_string(),
            source,
            risk_score,
            suspicious_patterns: Vec::new(),
            metadata_flags,
            code_analysis: Some(
                "Quick scan - no deep code analysis performed for trusted package".to_string(),
            ),
        })
    }

    async fn analyze_dependency<T: LlmClientTrait>(
        &self,
        package: &Package,
        llm_client: &mut T,
    ) -> Result<DependencyAnalysisResult> {
        // Determine dependency source
        let source = self.determine_dependency_source(package);

        // Check metadata for red flags
        let metadata_flags = self.analyze_package_metadata(package).await?;

        // Download and analyze source code (with size limits)
        let (code_analysis, suspicious_patterns) = if self.trusted_packages.contains(&package.name)
        {
            // Skip LLM analysis for trusted packages to save API calls
            (
                Some("Trusted package - skipped deep analysis".to_string()),
                Vec::new(),
            )
        } else {
            match timeout(
                Duration::from_secs(60),
                self.download_and_analyze_source(package, llm_client),
            )
            .await
            {
                Ok(Ok(result)) => result,
                Ok(Err(e)) => {
                    println!(
                        "   ⚠️  Could not analyze source for {}: {}",
                        package.name, e
                    );
                    (Some(format!("Failed to analyze source: {}", e)), Vec::new())
                }
                Err(_) => {
                    println!("   ⏰ Analysis timeout for {}", package.name);
                    (Some("Analysis timed out".to_string()), Vec::new())
                }
            }
        };

        // Calculate overall risk score
        let risk_score = self.calculate_risk_score(&metadata_flags, &suspicious_patterns);

        Ok(DependencyAnalysisResult {
            package_name: package.name.clone(),
            version: package.version.to_string(),
            source,
            risk_score,
            suspicious_patterns,
            metadata_flags,
            code_analysis,
        })
    }

    fn determine_dependency_source(&self, package: &Package) -> DependencySource {
        if let Some(source) = &package.source {
            let source_str = source.to_string();
            if source_str.contains("registry+") {
                DependencySource::CratesIo {
                    registry_url: source_str,
                }
            } else if source_str.contains("git+") {
                DependencySource::Git {
                    repository: source_str,
                    rev: None, // Could be extracted from source string
                }
            } else {
                DependencySource::Unknown
            }
        } else {
            // Likely a path dependency
            DependencySource::Path {
                path: "unknown".to_string(),
            }
        }
    }

    async fn analyze_package_metadata(&self, package: &Package) -> Result<Vec<MetadataFlag>> {
        let mut flags = Vec::new();

        // Check for typosquatting
        if let Some(similar_package) = self.check_typosquatting(&package.name) {
            flags.push(MetadataFlag {
                flag_type: MetadataFlagType::Typosquatting,
                description: format!(
                    "Package name '{}' is similar to popular package '{}'",
                    package.name, similar_package
                ),
                severity: "High".to_string(),
            });
        }

        // Check if package was published recently (potential 0-day)
        if let Some(metadata) = self.fetch_crates_io_metadata(&package.name).await? {
            // Check publication date
            if self.is_recently_published(&metadata) {
                flags.push(MetadataFlag {
                    flag_type: MetadataFlagType::RecentPublication,
                    description: "Package was published very recently, could be a 0-day injection"
                        .to_string(),
                    severity: "Medium".to_string(),
                });
            }

            // Check download count
            if self.has_low_downloads(&metadata) {
                flags.push(MetadataFlag {
                    flag_type: MetadataFlagType::LowDownloads,
                    description: "Package has unusually low download count for its age".to_string(),
                    severity: "Low".to_string(),
                });
            }
        }

        // Analyze dependencies for suspicious patterns
        self.analyze_dependency_tree(package, &mut flags);

        Ok(flags)
    }

    fn check_typosquatting(&self, package_name: &str) -> Option<String> {
        for popular_name in self.popular_packages.keys() {
            if self.is_similar_name(package_name, popular_name) && package_name != popular_name {
                return Some(popular_name.clone());
            }
        }
        None
    }

    fn is_similar_name(&self, name1: &str, name2: &str) -> bool {
        // Simple similarity check - could be enhanced with more sophisticated algorithms
        let distance = levenshtein_distance(name1, name2);
        distance <= 2 && distance > 0
    }

    async fn fetch_crates_io_metadata(
        &self,
        package_name: &str,
    ) -> Result<Option<serde_json::Value>> {
        let url = format!("https://crates.io/api/v1/crates/{}", package_name);

        match timeout(Duration::from_secs(10), self.client.get(&url).send()).await {
            Ok(Ok(response)) => {
                if response.status().is_success() {
                    let metadata = response.json::<serde_json::Value>().await?;
                    Ok(Some(metadata))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None), // Timeout or error - don't fail the entire scan
        }
    }

    fn is_recently_published(&self, metadata: &serde_json::Value) -> bool {
        // Check if package was created in the last 7 days
        if let Some(created_at) = metadata["crate"]["created_at"].as_str() {
            if let Ok(created_date) = chrono::DateTime::parse_from_rfc3339(created_at) {
                let now = chrono::Utc::now();
                let days_old = now.signed_duration_since(created_date).num_days();
                return days_old <= 7;
            }
        }
        false
    }

    fn has_low_downloads(&self, metadata: &serde_json::Value) -> bool {
        if let Some(downloads) = metadata["crate"]["downloads"].as_u64() {
            downloads < 1000 // Threshold for "low" downloads
        } else {
            true // No download info is suspicious
        }
    }

    fn analyze_dependency_tree(&self, package: &Package, flags: &mut Vec<MetadataFlag>) {
        // Check for suspicious dependency patterns
        let dep_names: Vec<&str> = package
            .dependencies
            .iter()
            .map(|d| d.name.as_str())
            .collect();

        // Check for networking capabilities
        let network_deps = ["reqwest", "hyper", "curl", "ureq", "attohttpc"];
        if dep_names.iter().any(|&name| network_deps.contains(&name)) {
            flags.push(MetadataFlag {
                flag_type: MetadataFlagType::NetworkingCapabilities,
                description: "Package has networking dependencies - review network usage"
                    .to_string(),
                severity: "Medium".to_string(),
            });
        }

        // Check for file system access
        let fs_deps = ["walkdir", "glob", "tempfile"];
        if dep_names.iter().any(|&name| fs_deps.contains(&name)) {
            flags.push(MetadataFlag {
                flag_type: MetadataFlagType::FileSystemAccess,
                description: "Package has file system access dependencies".to_string(),
                severity: "Low".to_string(),
            });
        }

        // Check for process execution
        let process_deps = ["tokio-process", "async-process"];
        if dep_names.iter().any(|&name| process_deps.contains(&name)) {
            flags.push(MetadataFlag {
                flag_type: MetadataFlagType::ProcessExecution,
                description: "Package can execute external processes".to_string(),
                severity: "High".to_string(),
            });
        }
    }

    async fn download_and_analyze_source<T: LlmClientTrait>(
        &self,
        package: &Package,
        llm_client: &mut T,
    ) -> Result<(Option<String>, Vec<FlaggedPattern>)> {
        // For now, we'll analyze the package's lib.rs or main.rs if accessible
        // In a full implementation, we'd download the crate source from crates.io

        // This is a simplified version - we'd need to implement actual source downloading
        let analysis_prompt = format!(
            "Analyze this Rust package for potential security threats, supply chain attacks, or malicious behavior:

Package: {} v{}
Dependencies: {}

Look specifically for:
1. Unexpected network requests or data exfiltration
2. File system manipulation beyond normal operations
3. Process execution or system command usage
4. Cryptographic operations that could be backdoors
5. Code obfuscation or suspicious patterns
6. Supply chain attack indicators

Provide analysis and flag any suspicious patterns with line numbers if possible.",
            package.name,
            package.version,
            package.dependencies.iter()
                .map(|d| format!("{}", d.name))
                .collect::<Vec<_>>()
                .join(", ")
        );

        let request = LlmRequest {
            prompt: analysis_prompt,
        };

        match timeout(Duration::from_secs(45), llm_client.analyze_code(request)).await {
            Ok(Ok(response)) => Ok((Some(response.analysis), response.flagged_patterns)),
            Ok(Err(e)) => {
                bail!("LLM analysis failed: {}", e)
            }
            Err(_) => {
                bail!("LLM analysis timed out")
            }
        }
    }

    fn calculate_risk_score(
        &self,
        metadata_flags: &[MetadataFlag],
        patterns: &[FlaggedPattern],
    ) -> RiskScore {
        let mut score = 0i32;

        // Weight metadata flags
        for flag in metadata_flags {
            match flag.flag_type {
                MetadataFlagType::Typosquatting => score += 50,
                MetadataFlagType::ProcessExecution => score += 30,
                MetadataFlagType::NetworkingCapabilities => score += 20,
                MetadataFlagType::SuspiciousAuthor => score += 40,
                MetadataFlagType::RecentPublication => score += 15,
                MetadataFlagType::LowDownloads => score += 10,
                _ => score += 5,
            }
        }

        // Weight flagged patterns
        for pattern in patterns {
            match pattern.severity.as_str() {
                "High" => score += 30,
                "Medium" => score += 15,
                "Low" => score += 5,
                _ => score += 5,
            }
        }

        // Convert score to risk level
        match score {
            s if s >= 80 => RiskScore::Critical,
            s if s >= 50 => RiskScore::High,
            s if s >= 25 => RiskScore::Medium,
            s if s >= 10 => RiskScore::Low,
            _ => RiskScore::Clean,
        }
    }

    fn compare_risk_scores(&self, a: &RiskScore, b: &RiskScore) -> std::cmp::Ordering {
        let a_val = match a {
            RiskScore::Critical => 4,
            RiskScore::High => 3,
            RiskScore::Medium => 2,
            RiskScore::Low => 1,
            RiskScore::Clean => 0,
        };
        let b_val = match b {
            RiskScore::Critical => 4,
            RiskScore::High => 3,
            RiskScore::Medium => 2,
            RiskScore::Low => 1,
            RiskScore::Clean => 0,
        };
        b_val.cmp(&a_val) // Reverse order (highest risk first)
    }
}

// Simple Levenshtein distance implementation
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.chars().count();
    let len2 = s2.chars().count();
    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    for (i, &c1) in s1_chars.iter().enumerate() {
        for (j, &c2) in s2_chars.iter().enumerate() {
            let cost = if c1 == c2 { 0 } else { 1 };
            matrix[i + 1][j + 1] = std::cmp::min(
                std::cmp::min(
                    matrix[i][j + 1] + 1, // deletion
                    matrix[i + 1][j] + 1, // insertion
                ),
                matrix[i][j] + cost, // substitution
            );
        }
    }

    matrix[len1][len2]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("serde", "sede"), 1);
        assert_eq!(levenshtein_distance("tokio", "tokyio"), 1);
        assert_eq!(levenshtein_distance("clap", "clep"), 1);
        assert_eq!(levenshtein_distance("completely", "different"), 9);
    }

    #[test]
    fn test_typosquatting_detection() {
        let scanner = DependencyScanner::new();
        assert!(scanner.check_typosquatting("serde-json").is_none()); // This is legitimate
        assert!(scanner.check_typosquatting("sede").is_some()); // This would be flagged
    }
}
