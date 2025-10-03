# RustRecon Report Formats

RustRecon supports multiple report formats to suit different use cases and verbosity preferences. Here's a comparison of the available formats:

## Available Formats

### 1. `markdown` (Default)
**Best for:** Detailed analysis, thorough documentation, sharing with teams
**File size:** Large (full content)

- Complete LLM analysis for each file
- Full code snippets for flagged patterns
- Detailed dependency analysis
- Comprehensive supply chain findings
- Perfect for thorough security reviews

### 2. `condensed`
**Best for:** Quick reviews, CI/CD integration, focused analysis
**File size:** Medium (summarized content)

- Summarized LLM analysis (first sentence + key findings)
- Only shows files with actual issues
- Pattern counts instead of full snippets
- High-risk dependencies only
- Removes verbose API errors and empty sections

### 3. `summary`
**Best for:** Dashboard views, quick status checks, batch scanning
**File size:** Minimal (single line)

- Ultra-compact one-line format
- Key metrics at a glance
- Immediate identification of issues
- Perfect for scripts and automation

### 4. `json`
**Best for:** Programmatic processing, integration with other tools
**File size:** Large (structured data)

- Machine-readable format
- Complete data structure
- Easy parsing and integration
- API-friendly output

## Format Examples

### Summary Format
```
📊 my_crate | Files: 15 | Patterns: 2 | Deps: 45 | High-Risk: 1 | ⚠️ serde:High | 🔍 Issues in: main.rs, auth.rs
```

### Condensed Format
```markdown
# RustRecon Scan Report: my_crate
*Timestamp: 2025-01-14T17:24:47Z*

## Summary
- **Files**: 15 | **Flagged Patterns**: 2 | **Dependencies**: 45 | **High-Risk Deps**: 1
- **Severity**: High: 1 | Medium: 1
- **Dependency Risk**: Critical: 0 | High: 1 | Medium: 3 | Low: 41

## ⚠️ High-Risk Dependencies
- **serde** v1.0.136 (High) - Flags: High (Typosquatting), Medium (Old Version)

## Code Findings
### `src/main.rs`
**Analysis**: Contains potential command injection vulnerability in user input handling...
**Patterns**: High (L45), Medium (L78)
```

### Full Markdown Format
```markdown
# RustRecon Scan Report: my_crate
*Timestamp: 2025-01-14T17:24:47Z*

## Summary
- Total files scanned: 15
- Total flagged patterns: 2
- Total dependencies scanned: 45
- High-risk dependencies: 1

### Severity Counts:
- High: 1
- Medium: 1

### Dependency Risk Counts:
- Critical: 0
- High: 1
- Medium: 3
- Low: 41

## Supply Chain Analysis
### ⚠️ High-Risk Dependencies
#### serde v1.0.136 - High
**Flags:**
- High (Typosquatting): Package name 'serde' is similar to popular package 'serdes'
- Medium (Old Version): Version is 6 months behind latest stable

**Analysis:** This dependency shows concerning patterns in recent updates. The maintainer changed recently and there are unusual network calls in the build script that weren't present in earlier versions. Recommend updating to latest version and reviewing changelog carefully.

### All Dependencies
- **serde** v1.0.136 - High
- **tokio** v1.24.1 - Low
- **clap** v4.1.0 - Low
[... full list continues ...]

## Detailed Code Findings
### File: `src/main.rs`
#### LLM Analysis:
```
The provided Rust code contains several security concerns that should be addressed:

1. Command Injection Risk (Line 45): The code directly passes user input to system commands without proper sanitization
2. Unsafe String Handling (Line 78): Raw string manipulation without bounds checking
3. Network Security (Line 123): HTTPS verification is disabled

Overall Risk Assessment: HIGH - This code should undergo security review before deployment.
```

#### Flagged Patterns:
- **Severity**: High
  - **Line**: 45
  - **Description**: Potential command injection via unsanitized user input
  - **Code Snippet**:
```rust
let output = Command::new("sh")
    .arg("-c")
    .arg(&user_input)  // Dangerous!
    .output()?;
```

- **Severity**: Medium
  - **Line**: 78
  - **Description**: Unsafe string slice operation
  - **Code Snippet**:
```rust
let result = &data[start..end];  // No bounds check
```

---
```

## Usage Examples

```bash
# Default detailed report
cargo run -- scan ./my_crate -o detailed_report.md

# Quick condensed report for CI/CD
cargo run -- scan ./my_crate --format condensed -o ci_report.md

# One-line summary for dashboard
cargo run -- scan ./my_crate --format summary

# JSON for tool integration
cargo run -- scan ./my_crate --format json -o results.json
```

## When to Use Each Format

| Format | Use Case | Best For |
|--------|----------|----------|
| `summary` | Quick status checks, batch processing | DevOps dashboards, automated scanning |
| `condensed` | Regular security reviews, CI/CD | Development teams, security reviews |
| `markdown` | Thorough analysis, documentation | Security audits, detailed investigations |
| `json` | Tool integration, data processing | APIs, custom tooling, databases |

## Performance Comparison

| Format | Generation Time | File Size | Network Usage |
|--------|-----------------|-----------|---------------|
| `summary` | Fastest | ~100 bytes | Minimal |
| `condensed` | Fast | ~5-10 KB | Low |
| `markdown` | Moderate | ~50-100 KB | High |
| `json` | Moderate | ~20-50 KB | High |

## Filtering Logic

The `condensed` and `summary` formats automatically filter content:

### Files Shown
- Files with flagged patterns
- Files with LLM analysis containing keywords: "concern", "vulnerability", "risk"
- Files with detected security issues

### Dependencies Shown  
- High and Critical risk dependencies only
- Dependencies with security flags
- Recently updated dependencies with suspicious changes

### Content Reduction
- LLM analysis truncated to first sentence + key findings
- Code snippets replaced with pattern summaries
- API errors and empty sections removed
- Metadata condensed to essential information

This approach ensures that condensed reports highlight only actionable security findings while maintaining readability and usefulness.