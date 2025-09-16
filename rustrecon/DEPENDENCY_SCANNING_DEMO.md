# RustRecon Dependency Scanning Demo

This document demonstrates RustRecon's advanced dependency scanning capabilities for detecting supply chain attacks, package injection 0-days, and malicious dependencies.

## Overview

RustRecon now includes comprehensive dependency analysis that goes beyond just scanning your source code. It analyzes every package in your `Cargo.toml` and `Cargo.lock` files to detect:

- **Package Injection Attacks** - Newly published malicious packages
- **Typosquatting** - Packages with names similar to popular ones
- **Supply Chain Compromises** - Legitimate packages that have been compromised
- **Suspicious Package Behavior** - Network access, file system manipulation, process execution
- **0-Day Package Injections** - Recently published packages that could be malicious

## How Dependency Scanning Works

### 1. Metadata Analysis
RustRecon examines package metadata from crates.io:
- **Publication Date**: Flags packages published in the last 7 days
- **Download Count**: Identifies packages with unusually low downloads
- **Author Information**: Checks for suspicious author patterns
- **Version History**: Analyzes release patterns

### 2. Typosquatting Detection
Uses Levenshtein distance algorithm to detect packages with names similar to popular crates:
- `serde` vs `sede` (distance: 1) ⚠️ SUSPICIOUS
- `tokio` vs `tokyio` (distance: 1) ⚠️ SUSPICIOUS
- `reqwest` vs `request` (distance: 2) ⚠️ SUSPICIOUS

### 3. Dependency Tree Analysis
Analyzes what capabilities each dependency has:
- **Networking**: `reqwest`, `hyper`, `curl` dependencies
- **File System**: `walkdir`, `glob`, `tempfile` dependencies
- **Process Execution**: `tokio-process`, `async-process` dependencies
- **Cryptographic**: Crypto-related dependencies

### 4. Smart LLM Analysis
Prioritizes suspicious packages for deep AI analysis while skipping trusted packages to conserve API quota:

**High Priority (Deep Analysis)**:
- Packages with typosquatting potential
- Recently published packages
- Packages with suspicious names containing: `steal`, `hack`, `backdoor`, `malware`, etc.
- Unknown/untrusted packages

**Low Priority (Quick Scan)**:
- Well-known trusted packages: `serde`, `tokio`, `clap`, `anyhow`, etc.
- Popular packages with high download counts

## Usage Examples

### Basic Dependency Scan
```bash
# Scan with dependency analysis (default)
cargo run -- scan /path/to/project -o full_security_report.md

# Dependency scanning is enabled by default
cargo run -- scan . --scan-dependencies -o report.md
```

### Code-Only Scan (Skip Dependencies)
```bash
# Skip dependency analysis to save API calls
cargo run -- scan . --skip-dependencies -o code_only_report.md
```

### JSON Output for Automation
```bash
# Generate machine-readable JSON report
cargo run -- scan . -f json -o security_report.json
```

## Sample Report Structure

```markdown
# RustRecon Scan Report: my_project

## Summary
- Total files scanned: 15
- Total flagged patterns: 3
- Total dependencies scanned: 45
- High-risk dependencies: 2

### Dependency Risk Counts:
- Critical: 1
- High: 1
- Medium: 5
- Low: 12
- Clean: 26

## Supply Chain Analysis

### ⚠️ High-Risk Dependencies

#### suspicious-crypto v0.1.0 - Critical
**Flags:**
- High (Typosquatting): Package name 'suspicious-crypto' is similar to popular package 'ring'
- Medium (Recent Publication): Package was published very recently, could be a 0-day injection
- High (Networking Capabilities): Package has networking dependencies - review network usage

**Analysis:** This package contains obfuscated network requests that could be used for data exfiltration...

#### process-runner v2.3.1 - High
**Flags:**
- High (Process Execution): Package can execute external processes
- Medium (Low Downloads): Package has unusually low download count for its age

**Analysis:** Package executes system commands without proper validation...

### All Dependencies
- **serde** v1.0.210 - Clean
- **tokio** v1.40.0 - Clean
- **suspicious-crypto** v0.1.0 - Critical
- **process-runner** v2.3.1 - High
- **clap** v4.5.20 - Clean
- ...
```

## Risk Scoring Algorithm

RustRecon uses a weighted scoring system:

**Metadata Flags:**
- Typosquatting: +50 points
- Process Execution: +30 points  
- Networking Capabilities: +20 points
- Suspicious Author: +40 points
- Recent Publication: +15 points
- Low Downloads: +10 points

**LLM Pattern Flags:**
- High Severity: +30 points
- Medium Severity: +15 points
- Low Severity: +5 points

**Risk Levels:**
- **Critical** (80+ points): Definite malicious behavior
- **High** (50-79 points): Very suspicious patterns
- **Medium** (25-49 points): Some concerning patterns  
- **Low** (10-24 points): Minor concerns
- **Clean** (0-9 points): No issues detected

## Real-World Attack Examples

### 1. Typosquatting Attack
```toml
[dependencies]
# Legitimate
serde = "1.0"

# Malicious typosquat (would be flagged)
sede = "1.0"  # ⚠️ Similar to 'serde'
```

### 2. Package Injection 0-Day
```toml
[dependencies]
# Newly published package with suspicious name
crypto-stealer = "0.1.0"  # ⚠️ Recent + suspicious name
```

### 3. Dependency Confusion
```toml
[dependencies]
# Internal package name that exists publicly
my-company-utils = "1.0"  # ⚠️ Could be dependency confusion
```

## API Quota Management

RustRecon intelligently manages API usage:

- **Trusted Package Skip**: Skips deep analysis of well-known packages
- **Rate Limiting**: 4-second delays between API calls
- **Smart Prioritization**: Analyzes suspicious packages first
- **Timeout Handling**: 60-second timeout per dependency
- **Graceful Degradation**: Continues scan even if some packages fail

### Free Tier Limits (Gemini):
- **15 requests/minute**
- **50 requests/day**
- **32K tokens/minute**

### Optimization Tips:
1. Use `--skip-dependencies` for faster code-only scans
2. Run dependency scans during off-peak hours
3. Consider upgrading to paid tier for large projects
4. Use JSON output for automated processing

## Integration with CI/CD

### GitHub Actions Example
```yaml
name: Security Scan
on: [push, pull_request]

jobs:
  security-scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Run RustRecon
        env:
          GEMINI_API_KEY: ${{ secrets.GEMINI_API_KEY }}
        run: |
          cargo install rustrecon
          rustrecon scan . -f json -o security-report.json
          
      - name: Upload Security Report
        uses: actions/upload-artifact@v3
        with:
          name: security-report
          path: security-report.json
```

### Fail Build on High-Risk Dependencies
```bash
# Exit with error code if critical/high risk dependencies found
rustrecon scan . -f json -o report.json
if grep -q '"risk_score":"Critical\|High"' report.json; then
    echo "⚠️ High-risk dependencies detected!"
    exit 1
fi
```

## Advanced Configuration

### Custom Trusted Packages
Extend the trusted packages list in `dependency_scanner.rs`:
```rust
trusted_packages.insert("my-company-crate".to_string());
trusted_packages.insert("internal-utils".to_string());
```

### Custom Suspicious Patterns
Add custom patterns to detect:
```rust
let suspicious_patterns = [
    "steal", "hack", "backdoor", "malware", 
    "keylog", "bitcoin", "mining", "payload",
    "exfiltrate", "trojan", "rootkit"
];
```

## Comparison with Other Tools

| Feature | RustRecon | cargo-audit | cargo-deny |
|---------|-----------|-------------|------------|
| CVE Detection | ❌ | ✅ | ✅ |
| Typosquatting | ✅ | ❌ | ❌ |
| 0-Day Detection | ✅ | ❌ | ❌ |
| LLM Analysis | ✅ | ❌ | ❌ |
| Supply Chain | ✅ | Partial | Partial |
| Code Analysis | ✅ | ❌ | ❌ |

## Future Enhancements

- **CVE Database Integration**: Combine with cargo-audit functionality
- **Package Source Verification**: Verify package source matches repository
- **Binary Analysis**: Analyze compiled artifacts for malicious code
- **Community Threat Intel**: Crowdsourced malicious package database
- **Automated Updates**: Regular updates of suspicious package lists
- **Custom Rules Engine**: User-defined security rules
- **Integration APIs**: REST API for CI/CD integration

## Troubleshooting

### Common Issues

**"Too many API requests"**
- Solution: Use `--skip-dependencies` or wait for quota reset
- Alternative: Upgrade to paid Gemini API plan

**"Failed to analyze dependency X"**
- Usually network timeouts or API issues
- The scan continues with other dependencies
- Check internet connection and API key

**"No high-risk dependencies found"**
- This is good! Your dependencies appear secure
- Try scanning a different project with more dependencies

### Debug Mode
```bash
RUST_LOG=debug cargo run -- scan . -o debug_report.md
```

## Best Practices

1. **Regular Scans**: Run dependency scans on every major dependency update
2. **CI Integration**: Automate scans in your build pipeline  
3. **Review Reports**: Manually review flagged dependencies
4. **Keep Updated**: Regularly update RustRecon for latest threat intelligence
5. **Combine Tools**: Use alongside cargo-audit and cargo-deny for comprehensive coverage
6. **Monitor New Deps**: Be especially cautious of newly added dependencies

## Contributing

Help improve dependency scanning:
- **Report False Positives**: Help us tune the detection algorithms
- **Add Threat Intelligence**: Contribute known malicious packages
- **Improve Algorithms**: Enhance typosquatting detection
- **Add Integrations**: Support for more package registries

## Conclusion

RustRecon's dependency scanning provides comprehensive supply chain security analysis, going beyond traditional vulnerability databases to detect emerging threats like package injection 0-days and typosquatting attacks. Combined with AI-powered code analysis, it offers unprecedented visibility into your project's security posture.

For questions or support, please open an issue on the GitHub repository.