# 000005: Memo Search and Context Features

## Overview
Implement advanced memo search capabilities and context aggregation features for optimal LLM integration.

## Goals
- Add powerful search functionality across all memos
- Implement context aggregation for LLM consumption
- Create efficient indexing and retrieval systems
- Support various search patterns and filters

## Tasks
1. Implement search functionality:
   - Full-text search across memo content
   - Title-based search with fuzzy matching
   - Tag-based search (if memos contain tags)
   - Date range filtering
   - Regular expression search support

2. Create context aggregation system:
   - `get_all_context()`: Combine all memos into single context
   - Smart formatting for LLM consumption
   - Include metadata (creation date, update date, etc.)
   - Optimize for token efficiency

3. Add search indexing:
   - Build in-memory index for fast searches
   - Support incremental index updates
   - Handle large numbers of memos efficiently

4. Implement advanced search features:
   - Boolean search operators (AND, OR, NOT)
   - Phrase searching with quotes
   - Wildcard pattern matching
   - Case-insensitive search by default

5. Create search result ranking:
   - Relevance scoring based on term frequency
   - Boost results by recency
   - Consider title matches vs content matches
   - Return results with snippets and highlights

6. Add filtering and sorting options:
   - Sort by creation date, update date, title
   - Filter by file path patterns
   - Limit result count and pagination
   - Include/exclude specific directories

## Success Criteria
- Fast full-text search across all memos
- Context aggregation produces well-formatted output
- Search results are ranked by relevance
- Advanced search operators work correctly
- Efficient performance with large memo collections
- Clean API for search and context operations

## Implementation Notes
- Use efficient string search algorithms
- Consider using a lightweight search library if needed
- Implement proper Unicode handling for search
- Cache search indices for performance
- Use async I/O for file operations
- Provide clear search syntax documentation
- Handle edge cases (empty results, malformed queries)

## Proposed Solution

After analyzing the existing codebase, I will implement the search and context features by:

1. **Create a new `search.rs` module** with:
   - `MemoSearcher` struct with in-memory indexing
   - `SearchQuery` struct for parsing search queries
   - `SearchResult` struct for ranked results with snippets
   - `SearchIndex` for efficient text matching

2. **Implement search functionality**:
   - Full-text search using tokenization and term matching
   - Title-based search with fuzzy matching (Levenshtein distance)
   - Tag-based search using exact matching
   - Date range filtering using memo created_at/updated_at
   - Regular expression search support

3. **Add context aggregation**:
   - `get_all_context()` method that combines all memos
   - Smart formatting optimized for LLM consumption
   - Include metadata (dates, tags, file paths)
   - Token-efficient formatting with clear delimiters

4. **Implement advanced search features**:
   - Boolean operators (AND, OR, NOT) with precedence
   - Phrase searching with quoted strings
   - Wildcard pattern matching (* and ?)
   - Case-insensitive search by default

5. **Add search result ranking**:
   - TF-IDF scoring for relevance
   - Boost results by recency (newer memos rank higher)
   - Title matches get higher scores than content matches
   - Return results with context snippets and highlights

6. **Extend existing `MemoStore`**:
   - Add search methods to `MemoStore`
   - Integrate with existing memo loading/listing
   - Maintain compatibility with current API

7. **Add supporting utilities**:
   - Text tokenization and normalization
   - Snippet extraction with highlighted terms
   - Query parsing and validation
   - Search result formatting

The implementation will be built on top of the existing `MemoStore` and `Memo` structures, ensuring full compatibility with the current system while adding powerful search capabilities.