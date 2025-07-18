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

---

## Update: 2025-07-18 10:04:03

## Proposed Solution

Based on my analysis of the existing codebase, I found that there's already a basic CLI structure in place, but it needs to be enhanced to follow the patterns specified in the issue. Here's my implementation plan:

### Current State Analysis
- `main.rs` already has clap CLI structure with Doctor and Server commands
- `HelpCommand` exists but is basic and doesn't include MCP integration instructions
- `DoctorCommand` exists but just prints "All systems operational"
- Server command uses port-based networking instead of stdio as specified for MCP
- Missing comprehensive help text and MCP integration instructions

### Implementation Steps

1. **Update CLI Structure**: 
   - Rename `Server` command to `Serve` to match the issue specification
   - Change serve command to use stdio instead of port-based networking
   - Update command structure to match the exact specification in the issue

2. **Enhance Help System**:
   - Update `HelpCommand` to display comprehensive MCP integration instructions
   - Include copy-pasteable configuration for Claude Code
   - Add example usage scenarios
   - Show help when no arguments are provided

3. **Improve Doctor Command**:
   - Check if `.memoranda` directory exists
   - Verify git repository context
   - Check file permissions
   - Validate memo file formats
   - Report any issues found with actionable suggestions

4. **Update Serve Command**:
   - Ensure MCP server starts on stdio (not port-based)
   - Handle graceful shutdown with proper signal handling
   - Implement proper error handling and logging

5. **Add Rich Help Text**:
   - Clear command descriptions with examples
   - Integration instructions for Claude Code
   - Troubleshooting guidance
   - Follow modern Rust CLI patterns similar to `uv`

6. **Implement Proper Error Handling**:
   - User-friendly error messages
   - Exit codes following Unix conventions
   - Helpful suggestions for common issues

7. **Testing**:
   - Write comprehensive tests for all CLI functionality
   - Test error scenarios and edge cases
   - Verify MCP integration works correctly

### Key Changes Required

1. Update `main.rs` to use exact CLI structure from issue specification
2. Enhance `HelpCommand` to include MCP integration instructions
3. Improve `DoctorCommand` with comprehensive diagnostics
4. Update `McpServer` to use stdio properly for MCP protocol
5. Add proper error handling and exit codes throughout
6. Write tests for all functionality

This approach builds on the existing foundation while implementing all the requirements specified in the issue.