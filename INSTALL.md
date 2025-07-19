# Installation Guide

## Quick Install (Recommended)

### Unix/Linux/macOS
```bash
curl -sSL https://raw.githubusercontent.com/wballard/memoranda/main/scripts/install.sh | bash
```

### Windows (PowerShell)
```powershell
irm https://raw.githubusercontent.com/wballard/memoranda/main/scripts/install.ps1 | iex
```

## Alternative Installation Methods

### Install via Cargo
If you have Rust installed:
```bash
cargo install memoranda
```

### Download Pre-built Binaries
1. Go to the [releases page](https://github.com/wballard/memoranda/releases)
2. Download the appropriate binary for your platform
3. Extract and place it in your PATH

### Build from Source
```bash
git clone https://github.com/wballard/memoranda.git
cd memoranda
cargo install --path .
```

## Platform Support

| Platform | Architecture | Support |
|----------|-------------|---------|
| Linux | x86_64 | ✅ |
| Linux | x86_64 (musl) | ✅ |
| macOS | x86_64 | ✅ |
| macOS | ARM64 | ✅ |
| Windows | x86_64 | ✅ |

## Post-Installation Setup

### 1. Verify Installation
```bash
memoranda --version
memoranda doctor
```

### 2. MCP Integration with Claude Code
Add this configuration to your MCP settings:

```json
{
  "mcpServers": {
    "memoranda": {
      "command": "memoranda",
      "args": ["serve"],
      "env": {}
    }
  }
}
```

### 3. Environment Configuration (Optional)
You can customize memoranda's behavior with environment variables:

```bash
# Set log level (trace, debug, info, warn, error)
export MEMORANDA_LOG_LEVEL=info

# Set custom cache directory
export MEMORANDA_CACHE_DIR="$HOME/.cache/memoranda"
```

## Troubleshooting

### Common Issues

**Command not found after installation**
- Make sure the installation directory is in your PATH
- Restart your terminal/shell session
- On Windows, you may need to restart PowerShell

**Permission denied errors**
- On Unix systems, ensure the binary is executable: `chmod +x memoranda`
- Make sure you have write permissions to the installation directory

**MCP server not connecting**
- Run `memoranda doctor` to diagnose issues
- Check that the MCP configuration matches the installation path
- Verify that memoranda can be found in your PATH

**Getting help**
```bash
memoranda --help
memoranda doctor --help
memoranda serve --help
```

For more help, visit the [GitHub repository](https://github.com/wballard/memoranda) or open an issue.