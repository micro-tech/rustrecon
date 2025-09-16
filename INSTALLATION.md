# RustRecon Installation Guide

Complete installation guide for RustRecon Security Scanner on Windows systems.

## 🚀 Quick Start (Windows 11)

### Option 1: One-Click Installer (Recommended)

```bash
# Download and run the installer
rustrecon-installer.exe
```

### Option 2: Silent Installation (For Automation)

```bash
# Automated installation with default settings
rustrecon-installer.exe --silent
```

### Option 3: Force Installation (Compatibility Issues)

```bash
# Bypass version checks if needed
rustrecon-installer.exe --force
```

## 📋 System Requirements

### Minimum Requirements
- **Operating System**: Windows 10 (build 19041) or later
- **Recommended**: Windows 11 (build 22000+)
- **Disk Space**: 500 MB available
- **Memory**: 2 GB RAM minimum
- **Network**: Internet connection for dependencies and API calls

### Prerequisites
- **Rust Toolchain**: Latest stable (installer can auto-install)
- **Git**: Optional for source compilation
- **Gemini API Key**: Required for LLM analysis

## 🔧 Installation Methods

### Method 1: Windows Installer (Recommended)

#### Step 1: Build the Installer

```bash
# Clone the repository
git clone https://github.com/yourusername/RustRecon.git
cd RustRecon/installer

# Build the installer
cargo build --release

# Copy to main directory
copy target\release\install.exe ..\rustrecon-installer.exe
```

#### Step 2: Run Installation

**Interactive Installation:**
```bash
cd ..
.\rustrecon-installer.exe
```

The installer will guide you through:
1. ✅ System requirements check
2. ⚙️ Installation preferences
3. 📁 Directory selection
4. 🔑 API key configuration
5. 🚀 Installation and setup

**Silent Installation:**
```bash
.\rustrecon-installer.exe --silent
```

Uses these defaults:
- **Install Path**: `C:\Program Files\RustRecon`
- **Add to PATH**: Yes
- **Desktop Shortcut**: Yes
- **Start Menu Entry**: Yes
- **API Key**: Not configured (manual setup required)

#### Step 3: Configure API Key

If not configured during installation:

```bash
# Initialize configuration
rustrecon init

# Edit the config file
notepad "C:\Program Files\RustRecon\rustrecon_config.toml"
```

Add your Gemini API key:
```toml
[llm]
gemini_api_key = "YOUR_ACTUAL_API_KEY_HERE"
gemini_api_endpoint = "https://generativelanguage.googleapis.com"
```

#### Step 4: Test Installation

```bash
# Test API connectivity
rustrecon test

# Quick scan
rustrecon scan ./my_project --format summary
```

### Method 2: Manual Installation

#### Prerequisites Check

```bash
# Check Rust installation
rustc --version
cargo --version

# If not installed, get Rust from: https://rustup.rs/
```

#### Build from Source

```bash
# Clone repository
git clone https://github.com/yourusername/RustRecon.git
cd RustRecon/rustrecon

# Build in release mode
cargo build --release

# The binary will be at: target/release/rustrecon.exe
```

#### Manual Setup

1. **Copy binary to desired location**:
   ```bash
   mkdir "C:\Program Files\RustRecon"
   copy target\release\rustrecon.exe "C:\Program Files\RustRecon\"
   copy rustrecon_config.toml "C:\Program Files\RustRecon\"
   ```

2. **Add to PATH** (as Administrator):
   ```bash
   setx /M PATH "%PATH%;C:\Program Files\RustRecon"
   ```

3. **Create shortcuts** (optional):
   - Desktop: Create shortcut to `rustrecon.exe`
   - Start Menu: Add to Programs folder

## 🔑 API Key Setup

### Getting a Gemini API Key

1. Go to [Google AI Studio](https://aistudio.google.com/)
2. Sign in with your Google account
3. Click "Get API Key" → "Create API Key"
4. Copy the generated key

### Configuration

Edit the configuration file:
```toml
# rustrecon_config.toml
[llm]
gemini_api_key = "AIzaSyD_YourActualAPIKeyHere_1234567890"
gemini_api_endpoint = "https://generativelanguage.googleapis.com"

[scanning]
max_file_size = 1048576
timeout_seconds = 30
concurrent_requests = 3

[output]
default_format = "condensed"
include_source_code = false
```

### Test API Connection

```bash
rustrecon test
```

Expected output:
```
🔍 Testing LLM API connection...
✅ API connection successful!
📋 Test response: API test successful
🎉 Your Gemini API is configured correctly!
```

## 📊 Usage Examples

### Basic Scanning

```bash
# Quick status check
rustrecon scan ./my_project --format summary

# Detailed analysis
rustrecon scan ./my_project --format condensed -o security_report.md

# Full comprehensive scan
rustrecon scan ./my_project --format markdown -o detailed_report.md

# JSON output for tools
rustrecon scan ./my_project --format json -o results.json
```

### Advanced Options

```bash
# Skip dependency scanning (faster)
rustrecon scan ./my_project --skip-dependencies

# Custom configuration file
rustrecon scan ./my_project --config ./custom_config.toml

# Multiple output formats
rustrecon scan ./my_project --format condensed -o report.md
rustrecon scan ./my_project --format json -o report.json
```

## 🚨 Troubleshooting

### Common Installation Issues

**Error: "Windows 11 or later is required"**
```bash
# Solution: Force installation
.\rustrecon-installer.exe --force
```

**Error: "Rust compiler not found"**
```bash
# Solution: Install Rust automatically
# The installer will prompt to install Rust
# Or install manually from: https://rustup.rs/
```

**Error: "Insufficient permissions"**
```bash
# Solution: Run as Administrator
# Right-click installer → "Run as administrator"
```

**Error: "API key validation failed"**
```bash
# Check API key in config file
notepad "C:\Program Files\RustRecon\rustrecon_config.toml"

# Verify API key format (starts with AIzaSy...)
# Test with: rustrecon test
```

### Runtime Issues

**Error: "Command not found"**
```bash
# Check PATH variable
echo %PATH%

# Should contain: C:\Program Files\RustRecon
# If not, re-run installer or add manually
```

**Error: "LLM API request failed"**
```bash
# Check internet connectivity
# Verify API key and quotas at: https://aistudio.google.com/
# Check rate limits (50 requests/day for free tier)
```

**Error: "Permission denied"**
```bash
# Run command prompt as Administrator
# Or change output directory to user folder
```

### Performance Issues

**Slow scanning:**
- Use `--skip-dependencies` for code-only scans
- Use `--format summary` for quick overviews
- Reduce `concurrent_requests` in config

**High API usage:**
- Use condensed format to reduce token usage
- Enable caching in configuration
- Consider upgrading API plan

## 🔄 Updating RustRecon

### Automatic Updates (If Enabled)
```bash
# Check for updates
rustrecon --version

# Updates will be applied automatically if configured
```

### Manual Updates
```bash
# Re-run installer to update
.\rustrecon-installer.exe

# Or rebuild from source
cd RustRecon/rustrecon
git pull
cargo build --release
```

## ❌ Uninstallation

### Using the Installer
```bash
.\rustrecon-installer.exe --uninstall
```

### Windows Settings
1. Go to **Settings** → **Apps** → **Installed apps**
2. Find **RustRecon Security Scanner**
3. Click **Uninstall**

### Manual Removal
If automated uninstall fails:

1. **Remove files**:
   ```bash
   rmdir /s "C:\Program Files\RustRecon"
   ```

2. **Remove from PATH**:
   - System Properties → Environment Variables
   - Remove RustRecon path from PATH variable

3. **Remove shortcuts**:
   - Delete desktop shortcut
   - Remove from Start Menu

4. **Registry cleanup** (optional):
   ```bash
   reg delete "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\RustRecon" /f
   ```

## 📞 Support

### Getting Help

- 📚 **Documentation**: Check `REPORT_FORMATS.md` for output options
- 🐛 **Bug Reports**: [GitHub Issues](https://github.com/yourusername/RustRecon/issues)
- 💬 **Discussions**: [GitHub Discussions](https://github.com/yourusername/RustRecon/discussions)

### Logs and Diagnostics

**Installation logs**:
```
%TEMP%\rustrecon-installer.log
```

**Runtime logs**:
```bash
# Enable verbose logging
set RUST_LOG=debug
rustrecon scan ./my_project
```

**Configuration check**:
```bash
rustrecon init --show-config
```

## 🔐 Security Considerations

### API Key Security
- Store API keys securely
- Use environment variables in CI/CD
- Rotate keys regularly
- Monitor usage and quotas

### Network Security
- Installer requires internet for dependencies
- Runtime needs internet for LLM API calls
- Consider proxy settings if needed

### Permission Requirements
- Installation requires Administrator privileges
- Runtime works with standard user permissions
- Output files respect system permissions

---

**Happy scanning with RustRecon! 🔍**

For additional help, see our documentation at: https://github.com/yourusername/RustRecon/wiki