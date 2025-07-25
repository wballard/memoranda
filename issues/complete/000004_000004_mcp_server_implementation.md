# 000004: MCP Server Implementation

## Overview
Implement the core MCP server functionality using the MCP SDK, providing tools for LLMs to interact with the memo system.

## Goals
- Create a robust MCP server using the Rust MCP SDK
- Implement MCP tools for memo operations
- Handle MCP protocol communication over stdio
- Provide comprehensive memo management capabilities

## Tasks
1. Implement MCP server structure:
   ```rust
   struct MemorandaServer {
       memo_store: MemoStore,
   }
   
   impl MemorandaServer {
       pub fn new() -> Self { ... }
       pub async fn run(&self) -> Result<()> { ... }
   }
   ```

2. Create MCP tools for memo operations:
   - `create_memo`: Create new memo with title and content
   - `update_memo`: Update existing memo by ID
   - `list_memos`: List all available memos with metadata
   - `get_memo`: Retrieve specific memo by ID
   - `search_memos`: Search memo content by text
   - `get_all_context`: Combine all memos for LLM context

3. Implement tool parameter schemas:
   - Use proper JSON schema definitions
   - Include validation for required/optional parameters
   - Provide clear descriptions and examples

4. Handle MCP protocol communication:
   - Implement proper request/response handling
   - Use stdio for communication with MCP client
   - Handle connection lifecycle and cleanup
   - Implement proper error responses

5. Add comprehensive error handling:
   - MCP protocol errors
   - File system operation errors
   - JSON parsing errors
   - Tool execution errors

6. Implement logging and debugging:
   - Log all tool invocations
   - Debug information for troubleshooting
   - Performance metrics for operations

## Success Criteria
- MCP server responds to tool discovery requests
- All memo operation tools work correctly
- Proper error handling and responses
- Clean JSON schema for all tools
- Server handles connection lifecycle properly
- Comprehensive logging for debugging

## Implementation Notes
- Follow MCP SDK patterns and best practices
- Use async/await for all I/O operations
- Implement proper JSON schema validation
- Use `tracing` for structured logging
- Handle Unicode content properly in memos
- Ensure thread safety for concurrent operations
- Follow the patterns from successful Rust MCP servers

## Proposed Solution

Based on the current code analysis, I need to implement the following:

1. **Fix the MCP Server Structure:**
   - Use `MemoStore` (file-based) instead of `MemoStorage` (in-memory)
   - Update server to handle `tools/call` requests
   - Add missing tools (update_memo, search_memos, get_all_context)
   - Make tool schemas more robust and complete

2. **Implement Missing Tools:**
   - `update_memo`: Update existing memo content by ID
   - `search_memos`: Search memo content by text pattern
   - `get_all_context`: Combine all memos for LLM context

3. **Improve Tool Execution:**
   - Replace stub implementations with actual memo operations
   - Add proper error handling and JSON responses
   - Implement proper parameter validation

4. **Add Comprehensive Tool Schemas:**
   - Complete JSON schemas for all tools
   - Add proper parameter validation
   - Include descriptions and examples

5. **Test Implementation:**
   - Create comprehensive tests for all tools
   - Test MCP protocol compliance
   - Test error handling scenarios

The implementation will use Test-Driven Development to ensure all functionality works correctly.