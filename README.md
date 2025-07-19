# Memoranda

A high-performance, memory-augmented note-taking system with MCP server capabilities for AI-assisted development workflows.

## Overview

Memoranda is a Rust-based command-line tool and MCP (Model Context Protocol) server that provides intelligent note-taking and memory management capabilities. It's designed to seamlessly integrate with AI coding assistants like Claude Code, allowing developers to store, search, and retrieve contextual information during development.

**Key Benefits:**
- **Persistent Memory**: Store and retrieve contextual information across coding sessions
- **AI Integration**: Native MCP server for seamless integration with AI assistants
- **High Performance**: Built in Rust with async I/O and intelligent caching
- **Developer-Friendly**: Simple CLI interface designed for coding workflows
- **Context-Aware**: Intelligent search and retrieval of notes with full-text search

## Features

- **CLI Interface**: Simple, fast command-line interface for memo management
- **MCP Server**: Full MCP protocol support for AI system integration
- **Context-Aware Search**: Intelligent search across memo titles and content
- **Health Monitoring**: Built-in doctor command for system diagnostics
- **Async Performance**: High-performance async I/O with intelligent caching
- **Data Validation**: Robust validation with clear error messages
- **ULID Identifiers**: Sortable, unique identifiers for reliable memo management

## Installation

### Option 1: Install from Git (Recommended)

```bash
# Install latest version from GitHub
cargo install --git https://github.com/wballard/memoranda.git memoranda --force

# Verify installation
memoranda --version
```

### Option 2: Build from Source

```bash
# Clone the repository
git clone https://github.com/wballard/memoranda.git
cd memoranda

# Build and install
cargo build --release
cargo install --path .

# Verify installation
memoranda --version
```

### Requirements

- **Rust**: 1.70.0 or later
- **Cargo**: Latest stable version

## Quick Start

### 1. Verify Installation

```bash
# Check system health
memoranda doctor

# Show help
memoranda --help
```

### 2. Start MCP Server

```bash
# Start server on default port
memoranda serve
```

### 3. Configure Claude Code Integration

Add memoranda to your Claude Code [MCP configuration](https://docs.anthropic.com/en/docs/claude-code/mcp):

```bash
# Add memoranda as MCP server
claude mcp add --scope user memoranda memoranda serve
```

**Alternative manual configuration** in your Claude Code MCP settings:

```json
{
  "mcpServers": {
    "memoranda": {
      "command": "memoranda",
      "args": ["serve"]
    }
  }
}
```

## CLI Commands

### `memoranda doctor`

Perform comprehensive system health checks and diagnostics.

```bash
# Basic health check
memoranda doctor

# Verbose output with detailed diagnostics
memoranda doctor --verbose

# Automatically fix detected issues
memoranda doctor --auto-fix

# Combine verbose output with auto-fix
memoranda doctor --verbose --auto-fix
```

**What it checks:**
- Configuration validity
- Data directory accessibility
- File system permissions  
- MCP server capabilities
- System dependencies

### `memoranda serve`

Start the MCP server for AI integration.

```bash
# Start MCP server (listens on stdin/stdout by default)
memoranda serve
```

**Server capabilities:**
- Handles MCP protocol communications
- Provides memo management tools to AI assistants
- Supports concurrent operations with async I/O
- Includes intelligent caching for performance

### `memoranda help`

Display help information and usage examples.

```bash
# Show general help
memoranda help
memoranda --help
memoranda -h

# Show command-specific help
memoranda doctor --help
memoranda serve --help
```

### `memoranda --version`

Display version and build information.

```bash
memoranda --version
memoranda -V
```

## MCP Tools

When running as an MCP server, memoranda provides the following tools for AI assistants:

### `create_memo`

Create a new memo with title and content.

**Parameters:**
```json
{
  "title": "string (1-255 characters, required)",
  "content": "string (max 1MB, required)"
}
```

**Example:**
```json
{
  "title": "API Authentication Pattern",
  "content": "Use bearer tokens in Authorization header. Implement refresh token rotation for security."
}
```

### `update_memo`

Update the content of an existing memo.

**Parameters:**
```json
{
  "id": "string (26-character ULID, required)",
  "content": "string (max 1MB, required)"
}
```

**Example:**
```json
{
  "id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
  "content": "Updated: Use bearer tokens with 15-minute expiry and implement proper refresh flows."
}
```

### `list_memos`

Retrieve a list of all memos.

**Parameters:**
```json
{}
```

**Returns:** Array of memo objects with id, title, created_at, updated_at, and tags.

### `get_memo`

Retrieve a specific memo by its ID.

**Parameters:**
```json
{
  "id": "string (26-character ULID, required)"
}
```

**Example:**
```json
{
  "id": "01ARZ3NDEKTSV4RRFFQ69G5FAV"
}
```

### `delete_memo`

Delete a memo by its ID.

**Parameters:**
```json
{
  "id": "string (26-character ULID, required)"
}
```

**Example:**
```json
{
  "id": "01ARZ3NDEKTSV4RRFFQ69G5FAV"
}
```

### `search_memos`

Search memos by query string (searches both title and content).

**Parameters:**
```json
{
  "query": "string (1-1000 characters, required)"
}
```

**Example:**
```json
{
  "query": "authentication API bearer"
}
```

### `get_all_context`

Retrieve all memos as context for AI processing.

**Parameters:**
```json
{}
```

**Returns:** All memo content formatted for AI context understanding.

## Configuration

Configuration is managed through the settings system. Default settings include:

- Data directory: `~/.memoranda/data`
- Log level: `info`
- Cache size: Adaptive based on system memory
- File validation: Enabled with comprehensive checks

## Usage Examples

### Basic Memo Management via AI Assistant

Once memoranda is configured with Claude Code, you can manage memos through natural language:

```
"Create a memo about REST API best practices"
"Search for memos about authentication"
"Update the memo about API patterns with new security information"
"Show me all memos related to database design"
```

### Development Workflow Integration

**Storing Code Patterns:**
```
"Create a memo titled 'Error Handling Pattern' with the content: 
'Use Result<T, E> for recoverable errors. Implement Display and Error traits. 
Use anyhow for application errors with context.'"
```

**Documenting Decisions:**
```
"Store this architectural decision: We're using async/await for I/O operations 
to improve performance. All database calls should be non-blocking."
```

**Capturing Learning:**
```
"Save a memo about Rust ownership rules: 
'Each value has exactly one owner. When owner goes out of scope, value is dropped. 
References don't take ownership.'"
```

### Real-World Scenarios

#### 1. Code Review Notes
Store insights from code reviews for future reference:
- Performance optimization techniques
- Security considerations
- Code style guidelines
- Common mistakes to avoid

#### 2. Project Documentation
Maintain living documentation that evolves with your project:
- API endpoint specifications
- Database schema decisions
- Third-party integration notes
- Deployment procedures

#### 3. Learning Journal
Track your learning progress and insights:
- New language features discovered
- Library comparisons and decisions
- Problem-solving approaches
- Best practices learned

## Advanced Integration

### Environment Variables

Configure memoranda behavior through environment variables:

```bash
# Set log level (trace, debug, info, warn, error)
export MEMORANDA_LOG_LEVEL=debug

# Set custom data directory
export MEMORANDA_DATA_DIR=/custom/path

# Enable detailed error reporting
export MEMORANDA_VERBOSE_ERRORS=true
```

### Claude Code Advanced Configuration

For advanced Claude Code integration, configure in your MCP settings:

```json
{
  "mcpServers": {
    "memoranda": {
      "command": "memoranda",
      "args": ["serve"],
      "env": {
        "MEMORANDA_LOG_LEVEL": "info",
        "MEMORANDA_DATA_DIR": "/path/to/your/memos"
      }
    }
  }
}
```

### Using with Other MCP Clients

Memoranda works with any MCP-compatible client:

```python
# Example Python client integration
from mcp import MCPClient

client = MCPClient("memoranda serve")
result = await client.call_tool("create_memo", {
    "title": "Python Integration Example",
    "content": "Successfully integrated memoranda with Python MCP client"
})
```

## Troubleshooting

### Common Issues

#### MCP Server Won't Start
```bash
# Check if memoranda is installed and accessible
which memoranda

# Run doctor command for diagnostics
memoranda doctor --verbose

# Check for permission issues
memoranda doctor --auto-fix
```

#### Claude Code Integration Problems
1. **Server not recognized**: Ensure memoranda is in your PATH
2. **Connection issues**: Try restarting Claude Code after configuration
3. **Permission errors**: Check file system permissions for data directory

#### Performance Issues
```bash
# Check system health
memoranda doctor --verbose

# Monitor resource usage
top | grep memoranda
```

#### Data Directory Issues
```bash
# Check current configuration
memoranda doctor

# Fix permission issues automatically
memoranda doctor --auto-fix

# Manual permission fix
chmod -R 755 ~/.memoranda/
```

### Getting Help

- **CLI help**: `memoranda --help`
- **Command-specific help**: `memoranda doctor --help`
- **System diagnostics**: `memoranda doctor --verbose`
- **GitHub Issues**: Report bugs and feature requests
- **Documentation**: Check this README for detailed information

## Development

### Prerequisites

- **Rust**: 1.70.0 or later (specified in `rust-toolchain.toml`)
- **Cargo**: Latest stable version
- **Git**: For version control

### Development Setup

```bash
# Clone repository
git clone https://github.com/wballard/memoranda.git
cd memoranda

# Build project
cargo build

# Run tests
cargo test

# Run with debug output
MEMORANDA_LOG_LEVEL=debug cargo run -- doctor --verbose

# Run benchmarks
cargo bench

# Check code formatting
cargo fmt --check

# Run lints
cargo clippy -- -D warnings
```

### Testing

Memoranda includes comprehensive test coverage:

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run integration tests
cargo test --test '*'

# Run doctests
cargo test --doc

# Run stress tests (if feature enabled)
cargo test --features stress_tests
```

### Project Architecture

```
src/
├── main.rs              # CLI entry point and error handling
├── lib.rs               # Library exports and public API
├── cli/                 # Command-line interface
│   ├── mod.rs           # CLI module exports
│   ├── doctor.rs        # System health and diagnostics
│   └── help.rs          # Help command implementation
├── mcp/                 # MCP server implementation
│   ├── mod.rs           # MCP module exports
│   ├── server.rs        # MCP protocol server
│   ├── tools.rs         # Tool definitions and schemas
│   └── tests.rs         # MCP protocol compliance tests
├── memo/                # Core memo functionality
│   ├── mod.rs           # Memo module exports
│   ├── models.rs        # Data models (Memo, MemoId)
│   ├── storage.rs       # File system operations
│   ├── search.rs        # Search and retrieval logic
│   └── cache.rs         # Intelligent caching system
├── config/              # Configuration management
│   ├── mod.rs           # Config module exports
│   └── settings.rs      # Application settings
├── error.rs             # Error types and handling
├── logging.rs           # Structured logging setup
└── utils.rs             # Shared utilities
```

### Data Models

#### Memo Structure
```rust
pub struct Memo {
    pub id: MemoId,              // ULID identifier
    pub title: String,           // 1-255 characters
    pub content: String,         // Up to 1MB
    pub created_at: DateTime<Utc>, // Creation timestamp
    pub updated_at: DateTime<Utc>, // Last modification
    pub tags: Vec<String>,       // Searchable tags
    pub file_path: Option<PathBuf>, // Optional file association
}
```

#### Key Design Decisions
- **ULID over UUID**: Sortable identifiers for better performance
- **Async I/O**: Non-blocking operations for scalability  
- **Strong Typing**: MemoId wrapper prevents ID mixing
- **Validation**: Content and title length limits
- **Caching**: Intelligent caching with memory management

### Performance Characteristics

- **Memory Usage**: Adaptive caching based on system resources
- **File I/O**: Async operations with proper error handling
- **Search**: Full-text search across titles and content
- **Concurrency**: Thread-safe operations with proper locking
- **Scalability**: Handles large memo collections efficiently

### Contributing

#### Code Standards
- Follow Rust idioms and best practices
- Use `rustfmt` for consistent formatting
- Pass all `clippy` lints without warnings
- Maintain test coverage above 90%
- Document public APIs with rustdoc

#### Pull Request Process
1. Fork the repository
2. Create a feature branch
3. Make changes with comprehensive tests
4. Ensure all tests pass: `cargo test`
5. Format code: `cargo fmt`
6. Check lints: `cargo clippy`
7. Submit pull request with clear description

#### Testing Requirements
- Unit tests for all new functions
- Integration tests for CLI commands
- MCP protocol compliance tests
- Performance benchmarks for critical paths
- Documentation tests for examples

#### Issue Reporting
- Use GitHub Issues for bugs and feature requests
- Include reproduction steps for bugs
- Provide system information for compatibility issues
- Check existing issues before creating new ones

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Rust](https://rust-lang.org/) for performance and safety
- Uses [MCP SDK](https://github.com/modelcontextprotocol/rust-sdk) for protocol implementation
- Inspired by modern note-taking and knowledge management systems