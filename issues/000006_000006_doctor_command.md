# 000006: Doctor Command Implementation

## Overview
Implement a comprehensive `doctor` command that diagnoses issues and provides actionable feedback to users.

## Goals
- Create thorough diagnostic checks for the memoranda system
- Provide clear, actionable feedback on issues
- Help users troubleshoot common problems
- Verify system health and configuration

## Tasks
1. Implement core diagnostic checks:
   - Verify git repository context
   - Check for `.memoranda` directory existence
   - Validate file permissions on memo directories
   - Check memo file format and validity
   - Verify MCP server dependencies

2. Create environment validation:
   - Check Rust version and toolchain
   - Verify required dependencies are available
   - Test file system permissions
   - Validate PATH and execution context

3. Implement memo validation:
   - Check memo file encoding (UTF-8)
   - Validate markdown format
   - Check for corrupted or unreadable files
   - Verify file naming conventions
   - Test memo parsing and serialization

4. Add MCP integration checks:
   - Verify MCP SDK functionality
   - Test stdio communication
   - Check for port conflicts or issues
   - Validate MCP protocol compliance

5. Create detailed reporting:
   - Color-coded status indicators (✅ ❌ ⚠️)
   - Clear problem descriptions
   - Actionable remediation steps
   - Summary of system health

6. Implement fix suggestions:
   - Automatic fixes for common issues
   - Step-by-step manual fix instructions
   - Links to documentation when relevant
   - Command examples for fixes

## Success Criteria
- Comprehensive system health checks
- Clear problem identification and reporting
- Actionable remediation suggestions
- User-friendly output with colors and formatting
- Covers all major failure modes
- Helps users self-diagnose issues

## Implementation Notes
- Use `tracing` for diagnostic logging
- Implement proper error handling for all checks
- Make output readable and scannable
- Use consistent formatting and terminology
- Test against common failure scenarios
- Provide both summary and detailed modes
- Follow patterns from other successful CLI tools

## Proposed Solution

I will enhance the existing doctor command implementation to create a comprehensive diagnostic tool with the following approach:

### 1. Test-Driven Development Strategy
- Create comprehensive tests for all diagnostic checks
- Implement each check incrementally to pass tests
- Use mock directories and files for testing edge cases

### 2. Enhanced Diagnostic Checks

#### Core System Checks
- **Git Repository**: Verify .git directory exists and is valid
- **Memoranda Directory**: Check `.memoranda` exists and has proper permissions
- **File Permissions**: Validate write permissions on critical directories
- **Memo Files**: Validate JSON format, UTF-8 encoding, and naming conventions

#### Environment Validation
- **Rust Toolchain**: Check `rustc --version` and validate minimum version
- **Dependencies**: Verify required system dependencies are available
- **MCP SDK**: Test MCP server functionality and stdio communication
- **File System**: Test file operations and path accessibility

#### Advanced Memo Validation
- **Encoding**: Ensure all memo files are valid UTF-8
- **Format**: Validate JSON structure and required fields
- **Naming**: Check file naming follows ULID conventions
- **Content**: Verify memo content integrity and markdown parsing

#### MCP Integration Checks
- **Server Startup**: Test MCP server can initialize
- **Tool Registration**: Verify all tools are properly registered
- **Communication**: Test stdio communication protocols
- **Protocol Compliance**: Validate MCP protocol adherence

### 3. Enhanced Reporting System

#### Visual Indicators
- ✅ Success (green)
- ❌ Error (red)
- ⚠️ Warning (yellow)
- 📋 Info (blue)

#### Detailed Output
- Clear problem descriptions
- Specific error messages with context
- Actionable remediation steps
- Command examples for fixes
- Links to documentation

#### Report Modes
- **Summary**: High-level health status
- **Detailed**: Full diagnostic information
- **JSON**: Machine-readable output for automation

### 4. Auto-Fix Capabilities
- Create missing `.memoranda` directory
- Fix file permissions where possible
- Repair corrupted JSON files
- Provide commands for manual fixes

### 5. Implementation Structure

```rust
pub struct DoctorCommand {
    verbose: bool,
    auto_fix: bool,
    format: OutputFormat,
}

enum DiagnosticResult {
    Pass,
    Warning(String),
    Error(String),
}

struct DiagnosticCheck {
    name: String,
    description: String,
    check_fn: Box<dyn Fn() -> DiagnosticResult>,
    fix_fn: Option<Box<dyn Fn() -> Result<()>>>,
}
```

### 6. Test Coverage
- Unit tests for each diagnostic check
- Integration tests for complete doctor runs
- Mock file system tests for edge cases
- Error condition testing
- Performance testing for large memo collections

### 7. Implementation Order
1. Create comprehensive test suite
2. Implement core diagnostic infrastructure
3. Add environment validation checks
4. Enhance memo validation capabilities
5. Add MCP integration checks
6. Implement reporting and formatting
7. Add auto-fix capabilities
8. Performance optimization

This approach ensures thorough system health checking while maintaining code quality and testability.