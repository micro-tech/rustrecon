use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::dependency_scanner::{DependencyAnalysisResult, RiskScore};
use crate::llm_client::FlaggedPattern;

#[derive(Debug, Serialize, Deserialize)]
pub struct RiskReport {
    pub crate_name: String,
    pub timestamp: String,
    pub findings: Vec<CrateFinding>,
    pub dependency_findings: Vec<DependencyAnalysisResult>,
    pub summary: ReportSummary,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CrateFinding {
    pub file_path: PathBuf,
    pub llm_analysis: String,
    pub flagged_patterns: Vec<FlaggedPattern>,
    // Potentially add findings from initial static analysis here
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportSummary {
    pub total_files_scanned: usize,
    pub total_flagged_patterns: usize,
    pub total_dependencies_scanned: usize,
    pub high_risk_dependencies: usize,
    pub severity_counts: HashMap<String, usize>,
    pub dependency_risk_counts: HashMap<String, usize>,
    // Overall risk score or other high-level metrics
}

impl RiskReport {
    pub fn new(crate_name: String) -> Self {
        RiskReport {
            crate_name,
            timestamp: chrono::Utc::now().to_rfc3339(),
            findings: Vec::new(),
            dependency_findings: Vec::new(),
            summary: ReportSummary {
                total_files_scanned: 0,
                total_flagged_patterns: 0,
                total_dependencies_scanned: 0,
                high_risk_dependencies: 0,
                severity_counts: HashMap::new(),
                dependency_risk_counts: HashMap::new(),
            },
        }
    }

    pub fn add_file_finding(
        &mut self,
        file_path: PathBuf,
        llm_analysis: String,
        flagged_patterns: Vec<FlaggedPattern>,
    ) {
        self.findings.push(CrateFinding {
            file_path,
            llm_analysis,
            flagged_patterns: flagged_patterns.clone(),
        });
        self.summary.total_files_scanned += 1;
        self.summary.total_flagged_patterns += flagged_patterns.len();
        for pattern in flagged_patterns {
            *self
                .summary
                .severity_counts
                .entry(pattern.severity)
                .or_insert(0) += 1;
        }
    }

    pub fn add_dependency_findings(&mut self, dependency_findings: Vec<DependencyAnalysisResult>) {
        self.summary.total_dependencies_scanned = dependency_findings.len();

        for finding in &dependency_findings {
            // Count risk levels
            let risk_key = match finding.risk_score {
                RiskScore::Critical => "Critical",
                RiskScore::High => "High",
                RiskScore::Medium => "Medium",
                RiskScore::Low => "Low",
                RiskScore::Clean => "Clean",
            };
            *self
                .summary
                .dependency_risk_counts
                .entry(risk_key.to_string())
                .or_insert(0) += 1;

            // Count high-risk dependencies
            if matches!(finding.risk_score, RiskScore::Critical | RiskScore::High) {
                self.summary.high_risk_dependencies += 1;
            }
        }

        self.dependency_findings = dependency_findings;
    }

    pub fn generate_report(&self, format: &str, output_path: Option<&Path>) -> anyhow::Result<()> {
        let report_content = match format {
            "json" => self.to_json()?,
            "markdown" => self.to_markdown()?,
            "condensed" => self.to_markdown_condensed()?,
            "summary" => self.to_summary()?,
            _ => anyhow::bail!("Unsupported report format: {}", format),
        };

        if let Some(path) = output_path {
            let mut file = File::create(path)?;
            file.write_all(report_content.as_bytes())?;
            println!("Report successfully written to {}", path.display());
        } else {
            io::stdout().write_all(report_content.as_bytes())?;
        }
        Ok(())
    }

    fn to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    fn to_markdown(&self) -> anyhow::Result<String> {
        let mut md = String::new();
        md.push_str(&format!("# RustRecon Scan Report: {}\n", self.crate_name));
        md.push_str(&format!("*Timestamp: {}*\n\n", self.timestamp));

        md.push_str("## Summary\n");
        md.push_str(&format!(
            "- Total files scanned: {}\n",
            self.summary.total_files_scanned
        ));
        md.push_str(&format!(
            "- Total flagged patterns: {}\n",
            self.summary.total_flagged_patterns
        ));
        md.push_str(&format!(
            "- Total dependencies scanned: {}\n",
            self.summary.total_dependencies_scanned
        ));
        md.push_str(&format!(
            "- High-risk dependencies: {}\n",
            self.summary.high_risk_dependencies
        ));
        md.push_str("### Severity Counts:\n");
        for (severity, count) in &self.summary.severity_counts {
            md.push_str(&format!("  - {}: {}\n", severity, count));
        }
        md.push_str("### Dependency Risk Counts:\n");
        for (risk, count) in &self.summary.dependency_risk_counts {
            md.push_str(&format!("  - {}: {}\n", risk, count));
        }
        md.push_str("\n");

        md.push_str("## Supply Chain Analysis\n");
        if self.dependency_findings.is_empty() {
            md.push_str("No dependency analysis performed.\n");
        } else {
            let high_risk_deps: Vec<_> = self
                .dependency_findings
                .iter()
                .filter(|d| matches!(d.risk_score, RiskScore::Critical | RiskScore::High))
                .collect();

            if !high_risk_deps.is_empty() {
                md.push_str("### ⚠️ High-Risk Dependencies\n");
                for dep in high_risk_deps {
                    md.push_str(&format!(
                        "#### {} v{} - {:?}\n",
                        dep.package_name, dep.version, dep.risk_score
                    ));
                    if !dep.metadata_flags.is_empty() {
                        md.push_str("**Flags:**\n");
                        for flag in &dep.metadata_flags {
                            md.push_str(&format!(
                                "- {} ({}): {}\n",
                                flag.severity,
                                format!("{:?}", flag.flag_type).replace("_", " "),
                                flag.description
                            ));
                        }
                    }
                    if let Some(analysis) = &dep.code_analysis {
                        md.push_str(&format!("**Analysis:** {}\n", analysis));
                    }
                    md.push_str("\n");
                }
            }

            md.push_str("### All Dependencies\n");
            for dep in &self.dependency_findings {
                md.push_str(&format!(
                    "- **{}** v{} - {:?}\n",
                    dep.package_name, dep.version, dep.risk_score
                ));
            }
        }

        md.push_str("\n## Detailed Code Findings\n");
        if self.findings.is_empty() {
            md.push_str("No suspicious patterns or findings detected.\n");
        } else {
            for finding in &self.findings {
                md.push_str(&format!("### File: `{}`\n", finding.file_path.display()));
                md.push_str(&format!(
                    "#### LLM Analysis:\n```\n{}\n```\n",
                    finding.llm_analysis
                ));
                if !finding.flagged_patterns.is_empty() {
                    md.push_str("#### Flagged Patterns:\n");
                    for pattern in &finding.flagged_patterns {
                        md.push_str(&format!(
                            "- **Severity**: {}\n  - **Line**: {}\n  - **Description**: {}\n  - **Code Snippet**:\n```rust\n{}\n```\n\n",
                            pattern.severity, pattern.line, pattern.description, pattern.code_snippet
                        ));
                    }
                } else {
                    md.push_str("No specific patterns flagged by LLM in this file.\n\n");
                }
                md.push_str("---\n\n");
            }
        }

        Ok(md)
    }

    fn to_markdown_condensed(&self) -> anyhow::Result<String> {
        let mut md = String::new();
        md.push_str(&format!("# RustRecon Scan Report: {}\n", self.crate_name));
        md.push_str(&format!("*Timestamp: {}*\n\n", self.timestamp));

        // Summary section
        md.push_str("## Summary\n");
        md.push_str(&format!(
            "- **Files**: {} | **Flagged Patterns**: {} | **Dependencies**: {} | **High-Risk Deps**: {}\n",
            self.summary.total_files_scanned,
            self.summary.total_flagged_patterns,
            self.summary.total_dependencies_scanned,
            self.summary.high_risk_dependencies
        ));

        // Only show severity/risk counts if they exist
        if !self.summary.severity_counts.is_empty() {
            md.push_str("- **Severity**: ");
            let severity_summary: Vec<String> = self
                .summary
                .severity_counts
                .iter()
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect();
            md.push_str(&severity_summary.join(" | "));
            md.push_str("\n");
        }

        if !self.summary.dependency_risk_counts.is_empty() {
            md.push_str("- **Dependency Risk**: ");
            let risk_summary: Vec<String> = self
                .summary
                .dependency_risk_counts
                .iter()
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect();
            md.push_str(&risk_summary.join(" | "));
            md.push_str("\n");
        }
        md.push_str("\n");

        // High-risk dependencies only (condensed)
        let high_risk_deps: Vec<_> = self
            .dependency_findings
            .iter()
            .filter(|d| matches!(d.risk_score, RiskScore::Critical | RiskScore::High))
            .collect();

        if !high_risk_deps.is_empty() {
            md.push_str("## ⚠️ High-Risk Dependencies\n");
            for dep in &high_risk_deps {
                md.push_str(&format!(
                    "- **{}** v{} ({:?})",
                    dep.package_name, dep.version, dep.risk_score
                ));

                if !dep.metadata_flags.is_empty() {
                    let flag_summary: Vec<String> = dep
                        .metadata_flags
                        .iter()
                        .map(|f| {
                            format!(
                                "{} ({})",
                                f.severity,
                                format!("{:?}", f.flag_type).replace("_", " ")
                            )
                        })
                        .collect();
                    md.push_str(&format!(" - Flags: {}", flag_summary.join(", ")));
                }
                md.push_str("\n");
            }
            md.push_str("\n");
        }

        // Code findings - only show files with issues
        let files_with_issues: Vec<_> = self
            .findings
            .iter()
            .filter(|f| {
                !f.flagged_patterns.is_empty()
                    || f.llm_analysis.contains("concern")
                    || f.llm_analysis.contains("vulnerability")
                    || f.llm_analysis.contains("risk")
            })
            .collect();

        if !files_with_issues.is_empty() {
            md.push_str("## Code Findings\n");
            for finding in &files_with_issues {
                md.push_str(&format!("### `{}`\n", finding.file_path.display()));

                // Extract key concerns from LLM analysis (first sentence or key phrases)
                let analysis_summary = if finding.llm_analysis.len() > 200 {
                    let first_sentence = finding
                        .llm_analysis
                        .split('.')
                        .next()
                        .unwrap_or(&finding.llm_analysis[..200])
                        .trim();
                    format!("{}...", first_sentence)
                } else {
                    finding.llm_analysis.clone()
                };

                md.push_str(&format!("**Analysis**: {}\n", analysis_summary));

                if !finding.flagged_patterns.is_empty() {
                    md.push_str("**Patterns**: ");
                    let pattern_summary: Vec<String> = finding
                        .flagged_patterns
                        .iter()
                        .map(|p| format!("{} (L{})", p.severity, p.line))
                        .collect();
                    md.push_str(&pattern_summary.join(", "));
                    md.push_str("\n");
                }
                md.push_str("\n");
            }
        } else {
            md.push_str("## Code Findings\n");
            md.push_str("No significant security concerns detected in code analysis.\n\n");
        }

        // Add a quick dependency list if there are dependencies but no high-risk ones
        if !self.dependency_findings.is_empty() && high_risk_deps.is_empty() {
            md.push_str("## Dependencies Status\n");
            md.push_str(&format!(
                "All {} dependencies appear to be low-risk.\n",
                self.dependency_findings.len()
            ));
        }

        Ok(md)
    }

    fn to_summary(&self) -> anyhow::Result<String> {
        let mut summary = String::new();

        // One-line summary
        summary.push_str(&format!(
            "📊 {} | Files: {} | Patterns: {} | Deps: {} | High-Risk: {}",
            self.crate_name,
            self.summary.total_files_scanned,
            self.summary.total_flagged_patterns,
            self.summary.total_dependencies_scanned,
            self.summary.high_risk_dependencies
        ));

        // Show high-risk dependencies inline if any
        let high_risk_deps: Vec<_> = self
            .dependency_findings
            .iter()
            .filter(|d| matches!(d.risk_score, RiskScore::Critical | RiskScore::High))
            .collect();

        if !high_risk_deps.is_empty() {
            summary.push_str(" | ⚠️ ");
            let risk_names: Vec<String> = high_risk_deps
                .iter()
                .map(|d| format!("{}:{:?}", d.package_name, d.risk_score))
                .collect();
            summary.push_str(&risk_names.join(", "));
        }

        // Show files with issues inline if any
        let files_with_issues: Vec<_> = self
            .findings
            .iter()
            .filter(|f| {
                !f.flagged_patterns.is_empty()
                    || f.llm_analysis.contains("concern")
                    || f.llm_analysis.contains("vulnerability")
                    || f.llm_analysis.contains("risk")
            })
            .collect();

        if !files_with_issues.is_empty() {
            summary.push_str(" | 🔍 Issues in: ");
            let file_names: Vec<String> = files_with_issues
                .iter()
                .map(|f| {
                    f.file_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string()
                })
                .collect();
            summary.push_str(&file_names.join(", "));
        }

        if high_risk_deps.is_empty() && files_with_issues.is_empty() {
            summary.push_str(" | ✅ Clean");
        }

        summary.push('\n');
        Ok(summary)
    }
}
