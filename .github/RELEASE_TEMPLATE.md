# Release v{version}

## What's New

### 🚀 New Features
- 

### 🐛 Bug Fixes
- 

### ⚡ Performance Improvements
- 

### 🔧 Other Changes
- 

## Breaking Changes
> **Note:** List any breaking changes here if this is a major version

## Installation

### Download Pre-built Binaries
Download the appropriate binary for your platform from the [releases page](https://github.com/wballard/memoranda/releases).

### Install via Cargo
```bash
cargo install memoranda
```

### Build from Source
```bash
git clone https://github.com/wballard/memoranda.git
cd memoranda
cargo install --path .
```

## Usage

### Basic Setup
```bash
# Run diagnostics to check your setup
memoranda doctor

# Start the MCP server
memoranda serve
```

### MCP Integration with Claude Code
Add this to your MCP settings:

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

## What's Next
- 

---

**Full Changelog**: https://github.com/wballard/memoranda/compare/v{previous_version}...v{version}