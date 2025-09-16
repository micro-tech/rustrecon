# 🔍 RustRecon - Rust Security Scanner

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Windows](https://img.shields.io/badge/Windows-11-blue.svg)](https://www.microsoft.com/windows/)
[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)

A comprehensive security scanner for Rust projects that leverages Large Language Models (LLMs) to detect malicious code, backdoors, and security vulnerabilities in Rust crates and dependencies.

## ✨ Features

- 🔍 **Deep Code Analysis** - LLM-powered semantic analysis using Google Gemini
- 🔗 **Supply Chain Security** - Comprehensive dependency scanning and risk assessment
- 📊 **Multiple Report Formats** - Summary, condensed, detailed markdown, and JSON outputs
- 🚀 **One-Click Installer** - Automated Windows installation with system integration
- ⚡ **Fast Scanning** - Optimized for quick security assessments
- 🎯 **CI/CD Ready** - Perfect for automated security pipelines

## 🚀 Quick Start

### Windows Installation (Recommended)

```bash
# Download and run the installer
rustrecon-installer.exe --silent

# Configure your API key
rustrecon init

# Your first security scan
rustrecon scan ./my_project --format summary
```

### Manual Installation

```bash
# Clone and build
git clone https://github.com/micro-tech/rustrecon.git
cd rustrecon/rustrecon
cargo build --release

# Run your first scan
./target/release/rustrecon scan ./my_project
```

## 📊 Output Formats

| Format | Use Case | Example Output |
|--------|----------|----------------|
| `summary` | Quick status checks | `📊 my_crate \| Files: 15 \| Patterns: 2 \| ✅ Clean` |
| `condensed` | CI/CD pipelines | Key findings only, ~5KB reports |
| `markdown` | Security reviews | Full analysis with code snippets |
| `json` | Tool integration | Machine-readable structured data |

## 🔧 Usage Examples

```bash
# Quick security overview
rustrecon scan ./my_project --format summary

# Detailed security report
rustrecon scan ./my_project --format condensed -o security_report.md

# Full analysis with dependencies
rustrecon scan ./my_project --format markdown -o full_report.md

# JSON for automation
rustrecon scan ./my_project --format json -o results.json

# Skip dependency scanning for faster results
rustrecon scan ./my_project --skip-dependencies
```

## 🛠️ Installation Options

### Windows Installer Features

- ✅ **One-Click Setup** - Automated installation and configuration
- ✅ **System Integration** - PATH setup, shortcuts, Start Menu entry
- ✅ **Permission Handling** - Works without Administrator privileges
- ✅ **Dependency Management** - Automatic Rust toolchain installation
- ✅ **Clean Uninstall** - Complete removal including registry cleanup

```bash
# Interactive installation
rustrecon-installer.exe

# Silent installation (automation-friendly)
rustrecon-installer.exe --silent

# Force installation (compatibility issues)
rustrecon-installer.exe --force

# Complete uninstall
rustrecon-installer.exe --uninstall
```

## 🔑 Configuration

### API Key Setup

1. Get a free Gemini API key: [Google AI Studio](https://aistudio.google.com/)
2. Configure RustRecon:
   ```bash
   rustrecon init
   ```
3. Edit the config file:
   ```toml
   [llm]
   gemini_api_key = "YOUR_API_KEY_HERE"
   gemini_api_endpoint = "https://generativelanguage.googleapis.com"
   ```
4. Test connection:
   ```bash
   rustrecon test
   ```

## 📁 Project Structure

```
rustrecon/
├── installer/              # Windows installer (Rust-based)
│   ├── src/main.rs         # Installer logic
│   ├── build.bat           # Build script
│   └── README.md           # Installer documentation
├── rustrecon/              # Main scanner application
│   ├── src/                # Core scanner code
│   ├── Cargo.toml          # Dependencies
│   └── README.md           # Scanner documentation
├── INSTALLATION.md         # Detailed installation guide
├── QUICK_INSTALL.md        # Quick start guide
└── README.md               # This file
```

## 🔍 How It Works

1. **File Scanning** - Traverses Rust project files (.rs, Cargo.toml)
2. **Static Analysis** - Detects suspicious patterns and code structures
3. **LLM Analysis** - Deep semantic analysis using Google Gemini
4. **Dependency Check** - Supply chain security assessment
5. **Risk Scoring** - Comprehensive security risk evaluation
6. **Report Generation** - Multiple output formats for different use cases

## 🚨 Security Features

### Code Analysis
- Unsafe block detection
- Suspicious network calls
- File system operations
- Command execution patterns
- Obfuscated code detection
- Backdoor pattern recognition

### Supply Chain Security
- Dependency vulnerability scanning
- Typosquatting detection
- Version analysis
- Maintainer changes
- Unusual update patterns
- License compliance

## 📋 System Requirements

- **Windows**: 10 (build 19041) or later, Windows 11 recommended
- **Rust**: Latest stable toolchain
- **Memory**: 2GB RAM minimum
- **Storage**: 500MB available space
- **Network**: Internet connection for LLM API calls

## 🔧 Development

### Building from Source

```bash
# Clone the repository
git clone https://github.com/micro-tech/rustrecon.git
cd rustrecon

# Build the scanner
cd rustrecon
cargo build --release

# Build the installer
cd ../installer
cargo build --release
```

### Running Tests

```bash
# Run scanner tests
cd rustrecon
cargo test

# Test the installer
cd ../
test_installer.bat
```

## 📚 Documentation

- 📖 **[Installation Guide](INSTALLATION.md)** - Complete setup instructions
- ⚡ **[Quick Install](QUICK_INSTALL.md)** - Get started in 5 minutes
- 📊 **[Report Formats](rustrecon/REPORT_FORMATS.md)** - Output format examples
- 🔧 **[Setup Guide](rustrecon/SETUP_GUIDE.md)** - Advanced configuration
- 💻 **[Installer Guide](installer/README.md)** - Technical installer details

## 🤝 Contributing

We welcome contributions! Please feel free to:

- 🐛 **Report Issues** - [GitHub Issues](https://github.com/micro-tech/rustrecon/issues)
- 💡 **Request Features** - [GitHub Discussions](https://github.com/micro-tech/rustrecon/discussions)
- 🔧 **Submit Pull Requests** - Fork and contribute code
- 📚 **Improve Documentation** - Help others get started

## 📊 Performance

| Scan Type | Time | Output Size | Best For |
|-----------|------|-------------|----------|
| Summary | ~10s | 1 line | Quick status |
| Condensed | ~30s | 5-10KB | Regular reviews |
| Full | ~2min | 50-100KB | Security audits |
| Dependencies | ~5min | Variable | Supply chain |

## 🛡️ Security & Privacy

- 🔒 **API Keys** - Stored locally, never transmitted to third parties
- 🌐 **Network Usage** - Only for LLM API calls and dependency checks  
- 📁 **Local Processing** - All analysis performed on your machine
- 🧹 **Clean Uninstall** - Complete removal of all data

## ⚠️ Limitations

- Requires active internet connection for LLM analysis
- API rate limits may apply (free tier: 50 requests/day)
- Large projects may take time to analyze completely
- Advanced obfuscation techniques may evade detection

## 📞 Support

### Quick Help
- 📚 Check our [documentation](rustrecon/README.md)
- 🚀 Try the [quick install guide](QUICK_INSTALL.md)
- 🔧 Review [troubleshooting](INSTALLATION.md#troubleshooting)

### Community Support
- 💬 [GitHub Discussions](https://github.com/micro-tech/rustrecon/discussions)
- 🐛 [Report Issues](https://github.com/micro-tech/rustrecon/issues)
- 📧 Contact: [security@micro-tech.dev](mailto:security@micro-tech.dev)

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](rustrecon/LICENSE) file for details.

## 🙏 Acknowledgments

- Google Gemini AI for powerful code analysis capabilities
- The Rust community for excellent tooling and libraries
- Security researchers for vulnerability patterns and detection techniques

---

**Secure your Rust projects with confidence! 🚀🔒**

*Made with ❤️ by the RustRecon team*