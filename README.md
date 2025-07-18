# Memoranda

A memory-augmented note-taking system with MCP server capabilities.

## Overview

Memoranda is a command-line tool and MCP (Model Context Protocol) server that provides intelligent note-taking and memory management capabilities. It allows you to store, search, and retrieve notes with context-aware features.

## Features

- **CLI Interface**: Simple command-line interface for note management
- **MCP Server**: Provides MCP server capabilities for integration with AI systems
- **Context-Aware Search**: Intelligent search and retrieval of notes
- **Health Monitoring**: Built-in doctor command for system health checks

## Installation

```bash
cargo install memoranda
```

## Usage

### Basic Commands

```bash
# Check system health
memoranda doctor

# Start MCP server
memoranda server --port 8080

# Show help
memoranda --help
```

### MCP Server

The MCP server provides endpoints for AI systems to interact with your notes:

```bash
memoranda server --port 8080
```

## Configuration

Configuration is managed through the settings system. Default settings include:

- Data directory: `./data`
- Log level: `info`
- MCP server port: `8080`

## Development

### Prerequisites

- Rust 1.70+
- Cargo

### Building

```bash
cargo build
```

### Running Tests

```bash
cargo test
```

### Running

```bash
cargo run -- doctor
```

## Architecture

The project is structured as follows:

```
src/
├── main.rs           # CLI entry point
├── lib.rs            # Library exports
├── mcp/              # MCP server implementation
│   ├── mod.rs
│   ├── server.rs
│   └── tools.rs
├── cli/              # CLI commands
│   ├── mod.rs
│   ├── doctor.rs
│   └── help.rs
├── memo/             # Memo handling
│   ├── mod.rs
│   ├── storage.rs
│   └── models.rs
└── config/           # Configuration
    ├── mod.rs
    └── settings.rs
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - see LICENSE file for details.