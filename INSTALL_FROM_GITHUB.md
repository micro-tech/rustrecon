# Install RustRecon from GitHub

Complete installation guide for downloading and installing RustRecon Security Scanner directly from GitHub.

## 🚀 Quick Installation (Recommended)

### One-Command Install

```bash
# Download installer and run
curl -L https://github.com/micro-tech/rustrecon/raw/main/rustrecon-installer.exe -o rustrecon-installer.exe
rustrecon-installer.exe --silent
```

### Manual Download

1. Go to: https://github.com/micro-tech/rustrecon
2. Click "Code" → "Download ZIP" or clone:
   ```bash
   git clone https://github.com/micro-tech/rustrecon.git
   cd rustrecon
   ```
3. Run the installer:
   ```bash
   rustrecon-installer.exe
   ```

## 📋 What the Installer Does

✅ **Downloads Latest Code** - Clones directly from GitHub repository  
✅ **Compiles from Source** - Builds RustRecon using your Rust toolchain  
✅ **System Integration** - Adds to PATH, creates shortcuts  
✅ **User-Friendly Install** - No Administrator privileges required  
✅ **Clean Setup** - Installs to `%LOCALAPPDATA%\RustRecon`  

## 🛠️ Installation Options

| Command | Description | Best For |
|---------|-------------|----------|
| `rustrecon-installer.exe` | Interactive setup | First-time users |
| `rustrecon-installer.exe --silent` | Automated install | Scripts, CI/CD |
| `rustrecon-installer.exe --force` | Bypass version checks | Older systems |

## 📍 Installation Location

**Default**: `C:\Users\YourName\AppData\Local\RustRecon\`

**Contains**:
- `rustrecon.exe` - Main scanner
- `rustrecon_config.toml.example` - Configuration template
- Documentation and guides

## ⚙️ Post-Installation Setup

### 1. Configure API Key

```bash
# Initialize configuration
rustrecon init

# Edit config file (opens in notepad)
notepad "%LOCALAPPDATA%\RustRecon\rustrecon_config.toml"
```

Add your Gemini API key:
```toml
[llm]
gemini_api_key = "YOUR_ACTUAL_API_KEY_HERE"
gemini_api_endpoint = "https://generativelanguage.googleapis.com"
```

**Get API Key**: https://aistudio.google.com/

### 2. Test Installation

```bash
# Verify installation
rustrecon --help

# Test API connection
rustrecon test

# First security scan
rustrecon scan . --format summary
```

## 📊 Usage Examples

```bash
# Quick project health check
rustrecon scan ./my_project --format summary

# Detailed security report
rustrecon scan ./my_project --format condensed -o report.md

# Full analysis with dependencies
rustrecon scan ./my_project --format markdown -o full_report.md

# JSON for automation
rustrecon scan ./my_project --format json -o results.json

# Code-only scan (faster)
rustrecon scan ./my_project --skip-dependencies
```

## 🔧 System Requirements

- **Windows**: 10 (build 19041) or later, Windows 11 recommended
- **Internet**: Required for installation and scanning
- **Disk Space**: 500MB minimum
- **Dependencies**: Rust toolchain (auto-installed by installer)

## 🚨 Common Issues & Solutions

### "Git not found" or "Clone failed"
```bash
# Install Git from: https://git-scm.com/
# Or use local installation:
rustrecon-installer.exe
# The installer will use local source if available
```

### "Access denied" during installation
```bash
# Use default user directory (no admin needed):
rustrecon-installer.exe --silent
```

### "Windows version not supported"
```bash
# Bypass compatibility check:
rustrecon-installer.exe --force
```

### "API key validation failed"
1. Get a free key from: https://aistudio.google.com/
2. Edit config: `%LOCALAPPDATA%\RustRecon\rustrecon_config.toml`
3. Test: `rustrecon test`

## 🔄 Updating RustRecon

```bash
# Re-run installer to get latest version
rustrecon-installer.exe --silent

# Or manually:
cd %LOCALAPPDATA%\RustRecon
git pull
cargo build --release
```

## ❌ Uninstallation

### Automatic
```bash
rustrecon-installer.exe --uninstall
```

### Manual
1. Delete folder: `%LOCALAPPDATA%\RustRecon`
2. Remove from PATH (if manually added)
3. Delete desktop/Start Menu shortcuts

## 📈 Report Formats

| Format | Size | Best For | Example |
|--------|------|----------|---------|
| `summary` | 1 line | Dashboards | `📊 project \| Files: 15 \| ✅ Clean` |
| `condensed` | ~5KB | Regular reviews | Key findings only |
| `markdown` | ~50KB | Security audits | Complete analysis |
| `json` | ~20KB | Tool integration | Machine-readable |

## 🆘 Need Help?

**Quick Links**:
- 📖 [Full Documentation](https://github.com/micro-tech/rustrecon/blob/main/README.md)
- ⚡ [Quick Install Guide](https://github.com/micro-tech/rustrecon/blob/main/QUICK_INSTALL.md)
- 🔧 [Detailed Setup](https://github.com/micro-tech/rustrecon/blob/main/INSTALLATION.md)

**Community Support**:
- 🐛 [Report Issues](https://github.com/micro-tech/rustrecon/issues)
- 💬 [Discussions](https://github.com/micro-tech/rustrecon/discussions)
- 📧 Email: security@micro-tech.dev

## ✅ Verification

After installation, verify everything works:

```bash
# Check installation
rustrecon --version
# Should output: rustrecon x.x.x

# Verify PATH
where rustrecon
# Should show: C:\Users\YourName\AppData\Local\RustRecon\rustrecon.exe

# Test scan capability
rustrecon scan . --format summary
# Should output: 📊 [project] | Files: X | ...
```

## 🎯 What's Next?

1. **Configure your API key** (required for scanning)
2. **Run your first security scan** on a Rust project
3. **Explore different report formats** for your use case
4. **Integrate into your development workflow**
5. **Share feedback** and contribute to the project

---

**Total setup time: ~5 minutes** ⏱️

**Repository**: https://github.com/micro-tech/rustrecon

**Happy scanning! 🔍🛡️**