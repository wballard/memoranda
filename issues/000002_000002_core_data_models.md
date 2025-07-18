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