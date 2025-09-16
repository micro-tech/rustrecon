# RustRecon Setup Guide

This guide will help you set up RustRecon with a real Gemini API key for actual security analysis.

## Quick Start

### Step 1: Get Your Gemini API Key (FREE)

1. **Visit Google AI Studio**: Go to [https://aistudio.google.com](https://aistudio.google.com)
2. **Sign in** with your Google account
3. **Create API Key**:
   - Click "Get API key" in the left sidebar
   - Click "Create API key"
   - Choose "Create API key in new project" (recommended)
   - Copy your API key and save it securely

### Step 2: Configure RustRecon

1. **Initialize configuration** (if you haven't already):
   ```bash
   cargo run -- init
   ```

2. **Edit the config file** `rustrecon_config.toml`:
   ```toml
   [llm]
   gemini_api_key = "paste_your_actual_api_key_here"
   gemini_api_endpoint = "https://generativelanguage.googleapis.com"
   temperature = 0.7
   max_tokens = 2048
   ```

### Step 3: Test Your Setup

Run a scan on a test crate:
```bash
# Scan current directory and save to file
cargo run -- scan . -o security_report.md

# Scan specific crate
cargo run -- scan /path/to/your/rust/project -o report.md

# Generate JSON report
cargo run -- scan . -f json -o report.json

# Print to terminal (no file saved)
cargo run -- scan .
```

## Usage Examples

### Basic Security Scan
```bash
# Scan your project and save as markdown
cargo run -- scan . -o my_security_report.md
```

### Advanced Options
```bash
# JSON format for programmatic processing
cargo run -- scan . -f json -o results.json

# Scan a specific directory
cargo run -- scan /path/to/suspicious/crate -o investigation.md
```

## What RustRecon Analyzes

- **Unsafe Code Blocks**: Identifies potentially dangerous `unsafe` code
- **External Dependencies**: Flags suspicious or unknown dependencies
- **File System Operations**: Detects file read/write operations
- **Network Operations**: Identifies network requests and connections
- **Process Execution**: Flags system command execution
- **Serialization/Deserialization**: Checks data handling patterns
- **Crypto Operations**: Reviews cryptographic implementations
- **Memory Management**: Analyzes manual memory operations

## Understanding the Report

### Report Structure
- **Summary**: Overview with file count and severity breakdown
- **Detailed Findings**: Per-file analysis with:
  - AI-generated security assessment
  - Specific flagged patterns with line numbers
  - Severity levels (High/Medium/Low)
  - Code snippets of problematic areas

### Severity Levels
- **High**: Potential security vulnerabilities or malicious code
- **Medium**: Suspicious patterns that need review
- **Low**: Best practice violations or minor concerns

## API Limits (Free Tier)

- **Requests per minute**: 15
- **Tokens per minute**: 32,000
- **Requests per day**: 1,500
- **Rate limiting**: Automatic retries built-in

For heavy usage, consider upgrading to a paid plan.

## Configuration Options

### LLM Settings
```toml
[llm]
gemini_api_key = "your_key_here"
gemini_api_endpoint = "https://generativelanguage.googleapis.com"
temperature = 0.7        # Creativity level (0.0-1.0)
max_tokens = 2048        # Maximum response length
```

### Advanced Configuration
- **temperature**: Lower values (0.1-0.3) for more focused analysis
- **temperature**: Higher values (0.7-0.9) for more creative interpretation
- **max_tokens**: Increase for longer, more detailed analyses

## Troubleshooting

### Common Issues

**"LLM configuration not found"**
```bash
# Re-run initialization
cargo run -- init
# Then edit rustrecon_config.toml with your API key
```

**"API request failed"**
- Check your API key is correct
- Verify internet connection
- Check rate limits (wait and retry)
- Ensure your Google Cloud project is active

**"No suspicious patterns found"**
- This is good! Your code appears secure
- Try scanning a different project
- Check that .rs files exist in the target directory

### Debug Mode
For verbose output:
```bash
RUST_LOG=debug cargo run -- scan . -o debug_report.md
```

## Security Best Practices

### Protecting Your API Key
- Never commit your API key to version control
- Use environment variables in production
- Rotate keys regularly
- Monitor API usage in Google Cloud Console

### Report Handling
- Review reports carefully - AI can have false positives
- Don't share reports containing proprietary code
- Use reports as a starting point for manual security review

## Alternative LLM Providers

While RustRecon currently supports Gemini, you can extend it to work with:
- OpenAI GPT-4
- Anthropic Claude
- Local models via Ollama
- Azure OpenAI

See the source code in `src/llm_client.rs` for implementation details.

## Contributing

Found a bug or want to add features?
- Submit issues on GitHub
- Contribute new LLM integrations
- Improve analysis prompts
- Add support for more programming languages

## License

Check the LICENSE file in the repository root.