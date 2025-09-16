# RustRecon: A Malicious Crate Scanner for Rust

RustRecon is a command-line interface (CLI) tool designed to scan Rust crates for suspicious patterns, potential backdoors, and unsafe code, leveraging the power of Large Language Models (LLMs) for semantic analysis. Its primary goal is to enhance the security posture of the Rust ecosystem by helping developers identify and mitigate risks within their dependencies.

## Features

*   **Crate Source Code Parsing**: Integrates with `cargo vendor` or direct repository access to parse Rust crate source code.
*   **Static Pattern Analysis**: Scans Rust files for predefined suspicious patterns (e.g., `unsafe` blocks, unusual file I/O, network calls).
*   **LLM-Powered Semantic Analysis**: Utilizes Google's Gemini LLM (via a direct API client) to perform deeper semantic analysis on code chunks, identifying complex and obfuscated malicious behavior.
*   **Comprehensive Risk Reporting**: Generates detailed risk reports with severity levels, flagged code lines, and explanations from the LLM.
*   **Extensible and Auditable**: Designed with modularity to allow for easy extension of analysis capabilities and transparent auditing of findings.

## How it Works

1.  **Crate Ingestion**: RustRecon takes a path to a local Rust crate (either directly from a repository or a vendored dependency).
2.  **File Traversal & Parsing**: It traverses the crate's directory, identifies all `.rs` files, and parses them into a Concrete Syntax Tree (CST) using `tree-sitter`.
3.  **Initial Static Scan**: A preliminary scan is performed on the CST to quickly identify common suspicious keywords or patterns.
4.  **LLM Analysis (Chunking)**: Code chunks (e.g., functions, modules) are intelligently extracted and sent to the Gemini LLM with relevant context.
5.  **Risk Assessment**: Gemini analyzes the code for malicious intent, unusual behavior, or unsafe practices.
6.  **Report Generation**: All findings, both from static analysis and LLM insights, are compiled into a human-readable risk report (Markdown or JSON).

## Getting Started

### Prerequisites

*   Rust toolchain (latest stable recommended)
*   An API key for Google Gemini (refer to Google Cloud documentation for setup)

### Installation

#### Windows 11 - One-Click Installer (Recommended)

For Windows 11 users, we provide a comprehensive Rust-based installer:

1. **Download the installer**:
   ```bash
   # Build from source
   git clone https://github.com/your-username/RustRecon.git
   cd RustRecon/installer
   cargo build --release
   copy target\release\install.exe ..\rustrecon-installer.exe
   ```

2. **Run the installer**:
   ```bash
   # Interactive installation (recommended)
   rustrecon-installer.exe
   
   # Silent installation for automation
   rustrecon-installer.exe --silent
   ```

The installer will:
- ✅ Check Windows 11 compatibility and system requirements
- ✅ Install Rust toolchain if needed
- ✅ Compile and install RustRecon
- ✅ Add to system PATH
- ✅ Create desktop and Start Menu shortcuts
- ✅ Set up configuration files
- ✅ Test API connectivity

#### Manual Installation

```bash
# Clone the repository
git clone https://github.com/your-username/RustRecon.git
cd RustRecon/rustrecon

# Build the project
cargo build --release
```

### Configuration

RustRecon requires an API key for the Gemini LLM. You should set this as an environment variable or configure it via a `rustrecon_config.toml` file.

```toml
# rustrecon_config.toml
[llm]
gemini_api_key = "YOUR_GEMINI_API_KEY"
gemini_api_endpoint = "https://generativelanguage.googleapis.com"
```

### Usage

#### Quick Start (After Installation)

```bash
# Test installation and API connectivity
rustrecon test

# Scan a local crate with default settings
rustrecon scan ./my_project

# Different output formats
rustrecon scan ./my_project --format summary          # One-line overview
rustrecon scan ./my_project --format condensed        # Key findings only  
rustrecon scan ./my_project --format markdown         # Full detailed report
rustrecon scan ./my_project --format json -o results.json

# Initialize configuration
rustrecon init
```

#### Report Formats

RustRecon supports multiple output formats for different use cases:

- **`summary`**: Ultra-compact one-line status (perfect for dashboards)
- **`condensed`**: Key findings only with reduced verbosity (ideal for CI/CD) 
- **`markdown`**: Full detailed analysis with complete information
- **`json`**: Machine-readable structured data for tool integration

See `REPORT_FORMATS.md` for detailed examples and usage guidance.

#### Manual Usage (Development)

```bash
# Scan a local crate
./target/release/rustrecon scan /path/to/your/crate

# Scan a crate and output a JSON report to a file
./target/release/rustrecon scan /path/to/your/crate --format json --output report.json

# Initialize a default configuration file
./target/release/rustrecon init --config-path ./my_custom_config.toml
```

## Module Structure

*   `src/main.rs`: Entry point and orchestration of the CLI.
*   `src/cli.rs`: Defines and parses command-line arguments using `clap`.
*   `src/scanner.rs`: Core logic for traversing crate files and parsing Rust code using `tree-sitter`.
*   `src/llm_client.rs`: Handles communication with the Gemini LLM, including request formatting and response parsing.
*   `src/report.rs`: Manages the data structures for scan findings and generates reports in various formats.
*   `src/config.rs`: Handles application configuration loading and parsing.
*   `src/utils.rs`: General utility functions (e.g., file operations, code chunking helpers).

## Uninstallation

### Windows 11

```bash
# Using the installer
rustrecon-installer.exe --uninstall

# Or through Windows Settings
# Settings > Apps > Installed apps > RustRecon > Uninstall
```

The uninstaller will completely remove:
- Installation files and directories
- System PATH entries  
- Desktop and Start Menu shortcuts
- Registry entries
- Configuration files

## Documentation

- 📚 **Installation Guide**: `installer/README.md` - Comprehensive installer documentation
- 📊 **Report Formats**: `REPORT_FORMATS.md` - Output format examples and usage
- ⚙️ **Setup Guide**: `SETUP_GUIDE.md` - Manual configuration and setup
- 🔍 **Dependency Scanning**: `DEPENDENCY_SCANNING_DEMO.md` - Supply chain security features

## Troubleshooting

### Common Issues

**API Key Issues**:
- Ensure your Gemini API key is valid and has sufficient quota
- Check `rustrecon_config.toml` for correct API key format

**Installation Issues**:
- Run installer as Administrator for system-wide installation
- Ensure Windows 11 (build 22000+) for compatibility
- Check available disk space (minimum 500MB required)

**Permission Issues**:
- Use `rustrecon-installer.exe --silent` for automated deployments
- Check antivirus software for false positives

## Contributing

We welcome contributions to RustRecon! Please feel free to open issues or submit pull requests.

## License

This project is licensed under the MIT License - see the `LICENSE` file for details.