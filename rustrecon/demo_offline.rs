use std::collections::HashMap;

/// Offline demo of RustRecon's dependency scanning capabilities
/// This shows what the tool can detect without making API calls

fn main() {
    println!("🔍 RustRecon Dependency Scanner - Offline Demo");
    println!("=================================================\n");

    // Simulate analyzing a project's dependencies
    let demo_dependencies = vec![
        ("serde", "1.0.210", false, "Clean - trusted package"),
        ("tokio", "1.40.0", false, "Clean - trusted package"),
        ("clap", "4.5.20", false, "Clean - trusted package"),
        (
            "sede",
            "1.0.0",
            true,
            "⚠️  TYPOSQUATTING: Similar to 'serde'",
        ),
        (
            "tokyio",
            "0.1.0",
            true,
            "⚠️  TYPOSQUATTING: Similar to 'tokio'",
        ),
        (
            "crypto-stealer",
            "0.1.0",
            true,
            "⚠️  SUSPICIOUS NAME: Contains 'steal'",
        ),
        (
            "backdoor-utils",
            "1.2.3",
            true,
            "⚠️  SUSPICIOUS NAME: Contains 'backdoor'",
        ),
        (
            "reqwest",
            "0.11.27",
            false,
            "Clean but has NETWORKING capabilities",
        ),
        (
            "process-runner",
            "0.1.0",
            true,
            "⚠️  LOW DOWNLOADS + PROCESS EXECUTION",
        ),
        (
            "bitcoin-miner",
            "2.0.0",
            true,
            "⚠️  SUSPICIOUS: Potential mining malware",
        ),
    ];

    println!("📊 Dependency Analysis Results:");
    println!("Found {} dependencies\n", demo_dependencies.len());

    let mut risk_counts = HashMap::new();
    let mut suspicious_count = 0;

    for (name, version, is_suspicious, analysis) in &demo_dependencies {
        let risk_level = if *is_suspicious {
            suspicious_count += 1;
            if analysis.contains("TYPOSQUATTING") {
                *risk_counts.entry("Critical").or_insert(0) += 1;
                "🔴 CRITICAL"
            } else if analysis.contains("SUSPICIOUS NAME") {
                *risk_counts.entry("High").or_insert(0) += 1;
                "🟠 HIGH"
            } else {
                *risk_counts.entry("Medium").or_insert(0) += 1;
                "🟡 MEDIUM"
            }
        } else {
            *risk_counts.entry("Clean").or_insert(0) += 1;
            "🟢 CLEAN"
        };

        println!("📦 {} v{}", name, version);
        println!("   Risk Level: {}", risk_level);
        println!("   Analysis: {}\n", analysis);
    }

    println!("📈 Risk Summary:");
    println!("===============");
    for (risk, count) in &risk_counts {
        println!("   {}: {} packages", risk, count);
    }
    println!(
        "   Total Suspicious: {}/{}",
        suspicious_count,
        demo_dependencies.len()
    );

    println!("\n🛡️  Supply Chain Security Checks:");
    println!("=================================");

    // Demonstrate different types of checks
    println!("✅ Typosquatting Detection:");
    println!("   - Detected 'sede' (similar to 'serde')");
    println!("   - Detected 'tokyio' (similar to 'tokio')");

    println!("\n✅ Suspicious Name Patterns:");
    println!("   - 'crypto-stealer' contains 'steal'");
    println!("   - 'backdoor-utils' contains 'backdoor'");
    println!("   - 'bitcoin-miner' suggests cryptocurrency mining");

    println!("\n✅ Capability Analysis:");
    println!("   - reqwest: Networking capabilities detected");
    println!("   - process-runner: Process execution capabilities");

    println!("\n✅ Metadata Analysis:");
    println!("   - Recently published packages flagged");
    println!("   - Low download counts detected");
    println!("   - Publication date analysis performed");

    println!("\n🚀 What RustRecon Actually Does:");
    println!("================================");
    println!("1. 📋 Parses your Cargo.toml and Cargo.lock files");
    println!("2. 🔍 Fetches metadata from crates.io API");
    println!("3. 🤖 Uses Gemini AI to analyze suspicious package code");
    println!("4. 📊 Generates detailed security reports");
    println!("5. ⚡ Prioritizes analysis to conserve API quota");

    println!("\n📝 Sample Commands:");
    println!("==================");
    println!("# Full scan with dependency analysis:");
    println!("cargo run -- scan /path/to/project -o report.md");
    println!();
    println!("# Code-only scan (skip dependencies):");
    println!("cargo run -- scan . --skip-dependencies -o code_report.md");
    println!();
    println!("# JSON output for automation:");
    println!("cargo run -- scan . -f json -o report.json");
    println!();
    println!("# Test API connection:");
    println!("cargo run -- test");

    println!("\n🔐 Security Features:");
    println!("====================");
    println!("• Package injection 0-day detection");
    println!("• Typosquatting attack prevention");
    println!("• Supply chain compromise detection");
    println!("• Malicious code pattern analysis");
    println!("• Dependency capability assessment");
    println!("• Smart API quota management");
    println!("• Comprehensive security reporting");

    println!("\n⚠️  Note: This is a demo showing detection capabilities.");
    println!("   Real scans require a Gemini API key and internet connection.");
    println!("   Get your free API key at: https://aistudio.google.com");

    // Simulate a simple typosquatting check
    println!("\n🔬 Typosquatting Algorithm Demo:");
    println!("===============================");
    let popular_packages = ["serde", "tokio", "clap", "reqwest", "anyhow"];
    let test_packages = ["sede", "tokyio", "clep", "request", "anyhwo"];

    for (popular, test) in popular_packages.iter().zip(test_packages.iter()) {
        let distance = levenshtein_distance(popular, test);
        println!(
            "   '{}' vs '{}' → Distance: {} {}",
            popular,
            test,
            distance,
            if distance <= 2 {
                "⚠️  SUSPICIOUS"
            } else {
                "✅ OK"
            }
        );
    }

    println!("\n🎯 Perfect for:");
    println!("==============");
    println!("• Security audits of Rust projects");
    println!("• CI/CD pipeline integration");
    println!("• Supply chain risk assessment");
    println!("• Open source project validation");
    println!("• Enterprise security compliance");
    println!("• Incident response and forensics");

    println!("\n🔥 Try RustRecon with your own projects!");
}

/// Simple Levenshtein distance calculation
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
    fn test_typosquatting_detection() {
        assert_eq!(levenshtein_distance("serde", "sede"), 1);
        assert_eq!(levenshtein_distance("tokio", "tokyio"), 1);
        assert_eq!(levenshtein_distance("clap", "clep"), 1);
        assert_eq!(levenshtein_distance("completely", "different"), 9);
    }
}
