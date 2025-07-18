# 000002: Core Data Models and Storage

## Overview
Implement the core data structures and file system storage for managing memos in `.memoranda` directories.

## Goals
- Define clean data models for memos
- Implement file system operations for memo storage
- Create directory traversal for finding `.memoranda` folders
- Establish storage conventions

## Tasks
1. Create `Memo` struct with fields:
   - `id`: ULID for unique identification
   - `title`: String title extracted from filename
   - `content`: String markdown content
   - `created_at`: DateTime timestamp
   - `updated_at`: DateTime timestamp
   - `file_path`: PathBuf to the memo file

2. Create `MemoStore` struct for storage operations:
   - `find_memoranda_dirs()`: Find all `.memoranda` directories in current git repo
   - `list_memos()`: List all memos in scope
   - `get_memo(id)`: Get specific memo by ID
   - `create_memo(title, content)`: Create new memo file
   - `update_memo(id, content)`: Update existing memo
   - `delete_memo(id)`: Delete memo file

3. Implement file system operations:
   - Traverse directory tree to find `.memoranda` folders
   - Read/write markdown files with proper error handling
   - Handle file naming conventions (sanitize titles for filenames)
   - Support both absolute and relative paths

4. Create utility functions:
   - `sanitize_filename()`: Convert title to safe filename
   - `extract_title_from_filename()`: Get title from filename
   - `find_git_root()`: Find git repository root
   - `generate_ulid()`: Create unique identifiers

5. Add comprehensive error handling:
   - File I/O errors
   - Permission errors
   - Invalid UTF-8 content
   - Directory traversal errors

## Success Criteria
- Can find `.memoranda` directories in a git repository
- Can create, read, update, and delete memo files
- Proper error handling for all file operations
- Clean separation of concerns between models and storage
- All functions have comprehensive unit tests

## Implementation Notes
- Use `ULID` for memo IDs to ensure sortable, unique identifiers
- Follow Rust naming conventions for all types
- Use `PathBuf` for all file paths
- Implement proper error types using `anyhow`
- Store memos as markdown files with meaningful filenames
- Support Unicode in memo content and titles

---

## Update: 2025-07-18 09:48:10


## Proposed Solution

Based on the existing codebase analysis, I'll implement the following:

### 1. Update Memo struct
- Add `file_path: Option<PathBuf>` field to the existing `Memo` struct
- Update constructor and methods to handle file paths
- Keep existing fields and functionality intact

### 2. Create MemoStore struct
- Implement file system-based storage alongside existing in-memory storage
- Add methods: `find_memoranda_dirs()`, `list_memos()`, `get_memo(id)`, `create_memo()`, `update_memo()`, `delete_memo()`
- Use `.memoranda` directories as storage locations within git repositories

### 3. Implement utility functions
- `sanitize_filename()`: Convert titles to filesystem-safe filenames
- `extract_title_from_filename()`: Parse titles from markdown filenames
- `find_git_root()`: Locate git repository root using `.git` directory
- Directory traversal using existing `walkdir` dependency

### 4. File system operations
- Read/write markdown files with frontmatter metadata
- Handle Unicode content properly
- Implement proper error handling for I/O operations
- Support both absolute and relative paths

### 5. Error handling
- Use `anyhow` for error types (already in dependencies)
- Handle file I/O, permissions, and UTF-8 conversion errors
- Provide meaningful error messages

### 6. Testing approach
- Unit tests for all utility functions
- Integration tests for file operations using `tempfile`
- Tests for error conditions and edge cases
- Tests for directory traversal and memo file discovery

This solution maintains compatibility with existing code while adding the required filesystem functionality.