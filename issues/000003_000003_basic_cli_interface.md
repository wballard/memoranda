# 000003: Basic CLI Interface

## Overview
Create a clean CLI interface inspired by modern Rust tools like `uv`, with helpful commands and excellent user experience.

## Goals
- Implement a modern CLI with `clap` following `uv` patterns
- Create helpful standalone usage information
- Provide MCP server integration instructions
- Build foundation for future commands

## Tasks
1. Create main CLI structure using `clap` derive macros:
   ```rust
   #[derive(Parser)]
   #[command(name = "memoranda")]
   #[command(about = "A note-taking MCP server for coding agents")]
   struct Cli {
       #[command(subcommand)]
       command: Option<Commands>,
   }
   
   #[derive(Subcommand)]
   enum Commands {
       Doctor,
       Serve,
   }
   ```

2. Implement standalone help that shows MCP integration instructions:
   - When run without arguments, show help for adding to Claude Code
   - Provide copy-pasteable configuration for MCP settings
   - Include example usage scenarios

3. Create `doctor` command stub:
   - Check if `.memoranda` directory exists
   - Verify git repository context
   - Check file permissions
   - Validate memo file formats
   - Report any issues found

4. Create `serve` command for MCP server mode:
   - Start MCP server on stdio
   - Handle graceful shutdown
   - Proper error handling and logging

5. Add rich help text and examples:
   - Clear command descriptions
   - Usage examples for each command
   - Integration instructions for Claude Code
   - Troubleshooting guidance

6. Implement proper error handling:
   - User-friendly error messages
   - Exit codes following Unix conventions
   - Helpful suggestions for common issues

## Success Criteria
- `memoranda` shows helpful standalone information
- `memoranda doctor` runs basic diagnostics
- `memoranda serve` starts MCP server mode
- All commands have comprehensive help text
- Error messages are clear and actionable
- CLI follows modern Rust CLI patterns

## Implementation Notes
- Use `clap` with derive macros for clean command definitions
- Follow `uv` patterns for CLI structure and help text
- Use `tracing` for all logging, not `println!`
- Implement proper signal handling for graceful shutdown
- Make help text copy-pasteable for MCP configuration
- Use colors and formatting for better UX (when appropriate)