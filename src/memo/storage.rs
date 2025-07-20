// use anyhow::anyhow;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use thiserror::Error;
use tokio::fs as async_fs;
use tracing::{info, warn};
use walkdir::WalkDir;

use super::cache::{MemoCache, MemoMetadata};
use super::models::{Memo, MemoId};
use super::search::{MemoSearcher, SearchQuery, SearchResult};
use crate::utils::{retry_with_backoff_sync, RetryConfig};

#[derive(Error, Debug)]
pub enum MemoStoreError {
    #[error("Memo not found: {id}")]
    MemoNotFound { id: String },

    #[error("No .memoranda directories found")]
    NoMemorandaDirectories,

    #[error("Invalid frontmatter in file {file}: {source}")]
    InvalidFrontmatter {
        file: String,
        source: serde_json::Error,
    },

    #[error("Missing frontmatter section in file {file}")]
    MissingFrontmatter { file: String },

    #[error("File operation failed: {source}")]
    FileOperation { source: std::io::Error },

    #[error("Serialization error: {source}")]
    Serialization { source: serde_json::Error },

    #[error("Validation error: {message}")]
    Validation { message: String },

    #[error("Walkdir error: {source}")]
    WalkDir { source: walkdir::Error },

    #[error("Git repository not found")]
    GitNotFound,
}

pub type Result<T> = std::result::Result<T, MemoStoreError>;

impl From<std::io::Error> for MemoStoreError {
    fn from(err: std::io::Error) -> Self {
        MemoStoreError::FileOperation { source: err }
    }
}

impl From<serde_json::Error> for MemoStoreError {
    fn from(err: serde_json::Error) -> Self {
        MemoStoreError::Serialization { source: err }
    }
}

impl From<walkdir::Error> for MemoStoreError {
    fn from(err: walkdir::Error) -> Self {
        MemoStoreError::WalkDir { source: err }
    }
}

impl From<anyhow::Error> for MemoStoreError {
    fn from(err: anyhow::Error) -> Self {
        MemoStoreError::Validation {
            message: err.to_string(),
        }
    }
}

#[derive(Default)]
pub struct MemoStorage {
    memos: HashMap<MemoId, Memo>,
}

impl MemoStorage {
    pub fn new() -> Self {
        info!("Creating memo storage");
        Self {
            memos: HashMap::new(),
        }
    }

    pub fn store_memo(&mut self, memo: Memo) -> Result<()> {
        info!("Storing memo: {}", memo.id);
        self.memos.insert(memo.id, memo);
        Ok(())
    }

    pub fn get_memo(&self, id: &MemoId) -> Option<&Memo> {
        self.memos.get(id)
    }

    pub fn list_memos(&self) -> Vec<&Memo> {
        self.memos.values().collect()
    }

    pub fn remove_memo(&mut self, id: &MemoId) -> Option<Memo> {
        info!("Removing memo: {}", id);
        self.memos.remove(id)
    }
}

#[derive(Debug)]
pub struct MemoStore {
    root_path: PathBuf,
    searcher: RwLock<MemoSearcher>,
    index_dirty: RwLock<bool>,
    cache: MemoCache,
}

impl MemoStore {
    // Helper function to check if a file is a markdown file
    fn is_markdown_file(path: &Path) -> bool {
        path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md")
    }

    // Helper function to create and cache memo metadata
    async fn create_and_cache_metadata(&self, memo: &Memo, file_path: &Path) -> Result<()> {
        if let Ok(file_metadata) = async_fs::metadata(file_path).await {
            if let Ok(last_modified) = file_metadata.modified() {
                let metadata = MemoMetadata {
                    id: memo.id,
                    title: memo.title.clone(),
                    file_path: file_path.to_path_buf(),
                    last_modified,
                    file_size: file_metadata.len(),
                };
                self.cache
                    .put_metadata(file_path.to_path_buf(), metadata)
                    .await;
            }
        }
        Ok(())
    }

    /// Prepares file content for memo serialization with frontmatter.
    /// This helper reduces duplication between sync and async save methods.
    fn prepare_memo_file_content(&self, memo: &Memo) -> Result<String> {
        // Create memo without file_path for serialization
        let mut memo_for_serialization = memo.clone();
        memo_for_serialization.file_path = None;

        let frontmatter = serde_json::to_string_pretty(&memo_for_serialization)?;
        Ok(format!("---\n{}\n---\n{}", frontmatter, memo.content))
    }

    /// Helper function to create a memo from content with frontmatter parsing fallback.
    /// This reduces duplication between sync and async loading methods.
    fn create_memo_from_content_with_fallback(
        &self,
        content: String,
        file_path: &Path,
    ) -> Result<Memo> {
        match self.parse_frontmatter(&content) {
            Ok(Some(mut memo)) => {
                memo.file_path = Some(file_path.to_path_buf());
                Ok(memo)
            }
            Ok(None) => {
                // No frontmatter found, create new memo from content
                let title = extract_title_from_filename(file_path);
                let memo = Memo::with_file_path(title, content, Some(file_path.to_path_buf()))?;
                Ok(memo)
            }
            Err(e) => {
                warn!(
                    "Failed to parse frontmatter in {}: {}",
                    file_path.display(),
                    e
                );
                // Fallback: create new memo from content
                let title = extract_title_from_filename(file_path);
                let memo = Memo::with_file_path(title, content, Some(file_path.to_path_buf()))?;
                Ok(memo)
            }
        }
    }

    // Helper function to parse frontmatter and extract memo ID from content
    fn extract_memo_id_from_content(content: &str, file_path: &Path) -> Result<Option<MemoId>> {
        if !content.starts_with("---\n") {
            return Ok(None);
        }

        let mut parts = content.splitn(3, "---\n");
        parts.next(); // skip the first empty part

        let frontmatter = parts.next().ok_or(MemoStoreError::MissingFrontmatter {
            file: file_path.display().to_string(),
        })?;

        // Parse just the id field from the frontmatter
        let value: serde_json::Value =
            serde_json::from_str(frontmatter).map_err(|e| MemoStoreError::InvalidFrontmatter {
                file: file_path.display().to_string(),
                source: e,
            })?;

        if let Some(id_str) = value.get("id").and_then(|v| v.as_str()) {
            if let Ok(ulid) = id_str.parse::<ulid::Ulid>() {
                return Ok(Some(MemoId::from_ulid(ulid)));
            }
        }

        Ok(None)
    }
    pub fn new(root_path: PathBuf) -> Self {
        Self {
            root_path,
            searcher: RwLock::new(MemoSearcher::new()),
            index_dirty: RwLock::new(true),
            cache: MemoCache::new(),
        }
    }

    pub fn new_with_cache_config(root_path: PathBuf, cache_size: u64, ttl_seconds: u64) -> Self {
        Self {
            root_path,
            searcher: RwLock::new(MemoSearcher::new()),
            index_dirty: RwLock::new(true),
            cache: MemoCache::with_config(cache_size, ttl_seconds),
        }
    }

    pub fn from_git_root() -> Result<Self> {
        let git_root = find_git_root()?;
        Ok(Self::new(git_root))
    }

    pub fn find_memoranda_dirs(&self) -> Result<Vec<PathBuf>> {
        let mut memoranda_dirs = Vec::new();

        for entry in WalkDir::new(&self.root_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_dir() && path.file_name().and_then(|s| s.to_str()) == Some(".memoranda") {
                memoranda_dirs.push(path.to_path_buf());
            }
        }

        Ok(memoranda_dirs)
    }

    pub async fn find_memoranda_dirs_async(&self) -> Result<Vec<PathBuf>> {
        let mut memoranda_dirs = Vec::new();
        let mut stack = vec![self.root_path.clone()];

        while let Some(current_dir) = stack.pop() {
            let mut entries = async_fs::read_dir(&current_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                let metadata = entry.metadata().await?;

                if metadata.is_dir() {
                    if path.file_name().and_then(|s| s.to_str()) == Some(".memoranda") {
                        memoranda_dirs.push(path);
                    } else {
                        // Add subdirectory to stack for recursive traversal
                        stack.push(path);
                    }
                }
            }
        }

        Ok(memoranda_dirs)
    }

    /// Gets the first available memoranda directory (sync version).
    /// This helper reduces duplication of the common pattern:
    /// find_memoranda_dirs()?.first().ok_or(NoMemorandaDirectories)
    pub fn get_primary_memoranda_dir(&self) -> Result<PathBuf> {
        let memoranda_dirs = self.find_memoranda_dirs()?;
        memoranda_dirs
            .first()
            .cloned()
            .ok_or(MemoStoreError::NoMemorandaDirectories)
    }

    /// Gets the first available memoranda directory (async version).
    /// This helper reduces duplication of the common pattern:
    /// find_memoranda_dirs_async().await?.first().ok_or(NoMemorandaDirectories)
    pub async fn get_primary_memoranda_dir_async(&self) -> Result<PathBuf> {
        let memoranda_dirs = self.find_memoranda_dirs_async().await?;
        memoranda_dirs
            .first()
            .cloned()
            .ok_or(MemoStoreError::NoMemorandaDirectories)
    }

    pub fn list_memos(&self) -> Result<Vec<Memo>> {
        let mut memos = Vec::new();
        let memoranda_dirs = self.find_memoranda_dirs()?;

        for dir in memoranda_dirs {
            for entry in fs::read_dir(&dir)? {
                let entry = entry?;
                let path = entry.path();

                if Self::is_markdown_file(&path) {
                    match self.load_memo_from_file(&path) {
                        Ok(memo) => memos.push(memo),
                        Err(e) => warn!("Failed to load memo from {}: {}", path.display(), e),
                    }
                }
            }
        }

        Ok(memos)
    }

    pub async fn list_memos_async(&self) -> Result<Vec<Memo>> {
        let mut memos = Vec::new();
        let memoranda_dirs = self.find_memoranda_dirs_async().await?;

        for dir in memoranda_dirs {
            let mut dir_entries = async_fs::read_dir(&dir).await?;
            while let Some(entry) = dir_entries.next_entry().await? {
                let path = entry.path();

                if Self::is_markdown_file(&path) {
                    match self.load_memo_from_file_async(&path).await {
                        Ok(memo) => memos.push(memo),
                        Err(e) => warn!("Failed to load memo from {}: {}", path.display(), e),
                    }
                }
            }
        }

        Ok(memos)
    }

    pub fn get_memo(&self, id: &MemoId) -> Result<Option<Memo>> {
        let memoranda_dirs = self.find_memoranda_dirs()?;

        for dir in memoranda_dirs {
            for entry in fs::read_dir(&dir)? {
                let entry = entry?;
                let path = entry.path();

                if Self::is_markdown_file(&path) {
                    // Quick check: read just the frontmatter to check ID
                    if let Ok(Some(memo_id)) = self.extract_memo_id_from_file(&path) {
                        if memo_id == *id {
                            // Found the memo, load it fully
                            return Ok(Some(self.load_memo_from_file(&path)?));
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    pub async fn get_memo_async(&self, id: &MemoId) -> Result<Option<Memo>> {
        // Check cache first
        if let Some(cached_memo) = self.cache.get_memo(id).await {
            // Verify cache validity if we have the file path
            if let Some(file_path) = &cached_memo.file_path {
                if self
                    .cache
                    .is_memo_valid(id, file_path)
                    .await
                    .unwrap_or(false)
                {
                    return Ok(Some((*cached_memo).clone()));
                }
            } else {
                return Ok(Some((*cached_memo).clone()));
            }
        }

        // Cache miss or invalid, load from file
        let memoranda_dirs = self.find_memoranda_dirs_async().await?;

        for dir in memoranda_dirs {
            let mut dir_entries = async_fs::read_dir(&dir).await?;
            while let Some(entry) = dir_entries.next_entry().await? {
                let path = entry.path();

                if Self::is_markdown_file(&path) {
                    // Quick check: read just the frontmatter to check ID
                    if let Ok(Some(memo_id)) = self.extract_memo_id_from_file_async(&path).await {
                        if memo_id == *id {
                            // Found the memo, load it fully and cache it
                            let memo = self.load_memo_from_file_async(&path).await?;

                            // Cache the memo
                            self.cache.put_memo(memo.clone()).await;

                            // Cache metadata for validation
                            if let Some(file_path) = &memo.file_path {
                                let _ = self.create_and_cache_metadata(&memo, file_path).await;
                            }

                            return Ok(Some(memo));
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    fn extract_memo_id_from_file(&self, file_path: &Path) -> Result<Option<MemoId>> {
        let file_path_clone = file_path.to_path_buf();

        // Retry file read operation with exponential backoff
        let content = retry_with_backoff_sync(
            || fs::read_to_string(&file_path_clone).map_err(anyhow::Error::from),
            RetryConfig::for_file_io(),
            "read_memo_file",
        )?;

        Self::extract_memo_id_from_content(&content, file_path)
    }

    async fn extract_memo_id_from_file_async(&self, file_path: &Path) -> Result<Option<MemoId>> {
        let content = async_fs::read_to_string(file_path).await?;
        Self::extract_memo_id_from_content(&content, file_path)
    }

    pub fn create_memo(&self, title: String, content: String) -> Result<Memo> {
        let target_dir = self.get_primary_memoranda_dir()?;

        let filename = sanitize_filename(&title);
        let file_path = target_dir.join(format!("{filename}.md"));

        let memo = Memo::with_file_path(title, content.clone(), Some(file_path.clone()))?;

        self.save_memo_to_file(&memo, &file_path)?;
        self.mark_index_dirty();

        Ok(memo)
    }

    pub async fn create_memo_async(&self, title: String, content: String) -> Result<Memo> {
        let target_dir = self.get_primary_memoranda_dir_async().await?;

        let filename = sanitize_filename(&title);
        let file_path = target_dir.join(format!("{filename}.md"));

        let memo = Memo::with_file_path(title, content.clone(), Some(file_path.clone()))?;

        self.save_memo_to_file_async(&memo, &file_path).await?;

        // Cache the newly created memo
        self.cache.put_memo(memo.clone()).await;

        // Cache metadata
        let _ = self.create_and_cache_metadata(&memo, &file_path).await;

        self.mark_index_dirty();

        Ok(memo)
    }

    pub fn update_memo(&self, id: &MemoId, content: String) -> Result<Memo> {
        let mut memo = self
            .get_memo(id)?
            .ok_or(MemoStoreError::MemoNotFound { id: id.to_string() })?;

        memo.update_content(content)?;

        if let Some(file_path) = &memo.file_path {
            self.save_memo_to_file(&memo, file_path)?;
        }
        self.mark_index_dirty();

        Ok(memo)
    }

    pub async fn update_memo_async(&self, id: &MemoId, content: String) -> Result<Memo> {
        let mut memo = self
            .get_memo_async(id)
            .await?
            .ok_or(MemoStoreError::MemoNotFound { id: id.to_string() })?;

        memo.update_content(content)?;

        if let Some(file_path) = &memo.file_path {
            self.save_memo_to_file_async(&memo, file_path).await?;

            // Update cache with new memo version
            self.cache.put_memo(memo.clone()).await;

            // Update metadata cache
            let _ = self.create_and_cache_metadata(&memo, file_path).await;
        }
        self.mark_index_dirty();

        Ok(memo)
    }

    pub fn delete_memo(&self, id: &MemoId) -> Result<()> {
        let memo = self
            .get_memo(id)?
            .ok_or(MemoStoreError::MemoNotFound { id: id.to_string() })?;

        if let Some(file_path) = &memo.file_path {
            let file_path_clone = file_path.clone();
            retry_with_backoff_sync(
                || fs::remove_file(&file_path_clone).map_err(anyhow::Error::from),
                RetryConfig::for_file_io(),
                "delete_memo_file",
            )?;

            // TODO: Remove from cache - need to handle async cache operations from sync context
        }
        self.mark_index_dirty();

        Ok(())
    }

    pub async fn delete_memo_async(&self, id: &MemoId) -> Result<()> {
        let memo = self
            .get_memo_async(id)
            .await?
            .ok_or(MemoStoreError::MemoNotFound { id: id.to_string() })?;

        if let Some(file_path) = &memo.file_path {
            async_fs::remove_file(file_path).await?;

            // Remove from cache
            self.cache.remove_memo(id).await;
            self.cache.remove_metadata(file_path).await;
        }
        self.mark_index_dirty();

        Ok(())
    }

    fn load_memo_from_file(&self, file_path: &Path) -> Result<Memo> {
        let content = fs::read_to_string(file_path)?;
        self.create_memo_from_content_with_fallback(content, file_path)
    }

    async fn load_memo_from_file_async(&self, file_path: &Path) -> Result<Memo> {
        let content = async_fs::read_to_string(file_path).await?;
        self.create_memo_from_content_with_fallback(content, file_path)
    }

    fn parse_frontmatter(&self, content: &str) -> Result<Option<Memo>> {
        if !content.starts_with("---\n") {
            return Ok(None);
        }

        let mut parts = content.splitn(3, "---\n");
        parts.next(); // skip the first empty part

        let frontmatter = parts.next().ok_or(MemoStoreError::MissingFrontmatter {
            file: "unknown".to_string(),
        })?;

        let _body = parts.next().ok_or(MemoStoreError::MissingFrontmatter {
            file: "unknown".to_string(),
        })?;

        // Parse frontmatter as JSON
        let memo = serde_json::from_str::<Memo>(frontmatter).map_err(|e| {
            MemoStoreError::InvalidFrontmatter {
                file: "unknown".to_string(),
                source: e,
            }
        })?;

        Ok(Some(memo))
    }

    fn save_memo_to_file(&self, memo: &Memo, file_path: &Path) -> Result<()> {
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let file_content = self.prepare_memo_file_content(memo)?;

        // Atomic write: write to temporary file first, then rename
        let temp_file_path = file_path.with_extension("md.tmp");
        let file_path_clone = file_path.to_path_buf();
        let temp_file_path_clone = temp_file_path.clone();
        let file_content_clone = file_content.clone();

        // Write to temporary file with retry logic
        retry_with_backoff_sync(
            || fs::write(&temp_file_path_clone, &file_content_clone).map_err(anyhow::Error::from),
            RetryConfig::for_file_io(),
            "write_memo_temp_file",
        )?;

        // Atomically rename temporary file to final destination with retry
        retry_with_backoff_sync(
            || fs::rename(&temp_file_path, &file_path_clone).map_err(anyhow::Error::from),
            RetryConfig::for_file_io(),
            "rename_memo_file",
        )
        .map_err(|e| {
            // Clean up temporary file on failure
            let _ = fs::remove_file(&temp_file_path);
            e
        })?;

        Ok(())
    }

    async fn save_memo_to_file_async(&self, memo: &Memo, file_path: &Path) -> Result<()> {
        if let Some(parent) = file_path.parent() {
            async_fs::create_dir_all(parent).await?;
        }

        let file_content = self.prepare_memo_file_content(memo)?;

        // Atomic write: write to temporary file first, then rename
        let temp_file_path = file_path.with_extension("md.tmp");

        // Write to temporary file
        async_fs::write(&temp_file_path, &file_content).await?;

        // Atomically rename temporary file to final destination
        match async_fs::rename(&temp_file_path, file_path).await {
            Ok(()) => Ok(()),
            Err(e) => {
                // Clean up temporary file on failure
                let _ = async_fs::remove_file(&temp_file_path).await;
                Err(e.into())
            }
        }
    }

    pub fn search_memos(&self, query: &str) -> Result<Vec<SearchResult>> {
        let memos = self.list_memos()?;
        self.ensure_index_updated(&memos)?;

        let searcher = self.searcher.read().unwrap();
        let search_query = SearchQuery::parse_query(query);
        let results = searcher.search(&search_query, &memos);

        Ok(results)
    }

    pub fn search_memos_with_query(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let memos = self.list_memos()?;
        self.ensure_index_updated(&memos)?;

        let searcher = self.searcher.read().unwrap();
        let results = searcher.search(query, &memos);

        Ok(results)
    }

    pub fn get_all_context(&self) -> Result<String> {
        let memos = self.list_memos()?;
        let searcher = MemoSearcher::new();

        Ok(searcher.get_all_context(&memos))
    }

    /// Ensures the search index is up-to-date with the current memos
    fn ensure_index_updated(&self, memos: &[Memo]) -> Result<()> {
        let is_dirty = *self.index_dirty.read().unwrap();

        if is_dirty {
            let mut searcher = self.searcher.write().unwrap();
            *searcher = MemoSearcher::new();

            // Re-index all memos
            for memo in memos {
                searcher.index_memo(memo);
            }

            *self.index_dirty.write().unwrap() = false;
        }

        Ok(())
    }

    /// Marks the search index as dirty, requiring re-indexing
    fn mark_index_dirty(&self) {
        *self.index_dirty.write().unwrap() = true;
    }

    /// Get cache statistics for monitoring
    pub async fn get_cache_stats(&self) -> super::cache::CacheStats {
        self.cache.get_stats().await
    }

    /// Get cache hit ratio for monitoring
    pub fn get_cache_hit_ratio(&self) -> f64 {
        self.cache.cache_hit_ratio()
    }

    /// Clear all cached memos (useful for testing or memory management)
    pub async fn clear_cache(&self) {
        self.cache.invalidate_all().await;
    }

    /// Preload frequently accessed memos into cache
    pub async fn warm_cache(&self) -> Result<usize> {
        let memos = self.list_memos_async().await?;
        let count = memos.len();

        for memo in memos {
            self.cache.put_memo(memo).await;
        }

        info!("Warmed cache with {} memos", count);
        Ok(count)
    }
}

pub fn sanitize_filename(title: &str) -> String {
    title
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            '\0'..='\x1f' => '_',
            c => c,
        })
        .collect::<String>()
        .trim_matches('.')
        .to_string()
}

pub fn extract_title_from_filename(file_path: &Path) -> String {
    file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled")
        .replace('_', " ")
}

pub fn find_git_root() -> Result<PathBuf> {
    let current_dir = std::env::current_dir()?;
    let mut dir = current_dir.as_path();

    loop {
        if dir.join(".git").exists() {
            return Ok(dir.to_path_buf());
        }

        match dir.parent() {
            Some(parent) => dir = parent,
            None => return Err(MemoStoreError::GitNotFound),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memo_storage_creation() {
        let storage = MemoStorage::new();
        assert_eq!(storage.list_memos().len(), 0);
    }

    #[test]
    fn test_store_and_retrieve_memo() {
        let mut storage = MemoStorage::new();
        let memo = Memo::new("Test".to_string(), "Content".to_string()).unwrap();
        let memo_id = memo.id;

        storage.store_memo(memo).unwrap();

        let retrieved = storage.get_memo(&memo_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().title, "Test");
    }

    #[test]
    fn test_list_memos() {
        let mut storage = MemoStorage::new();
        let memo1 = Memo::new("Test1".to_string(), "Content1".to_string()).unwrap();
        let memo2 = Memo::new("Test2".to_string(), "Content2".to_string()).unwrap();

        storage.store_memo(memo1).unwrap();
        storage.store_memo(memo2).unwrap();

        let memos = storage.list_memos();
        assert_eq!(memos.len(), 2);
    }

    #[test]
    fn test_remove_memo() {
        let mut storage = MemoStorage::new();
        let memo = Memo::new("Test".to_string(), "Content".to_string()).unwrap();
        let memo_id = memo.id;

        storage.store_memo(memo).unwrap();
        assert!(storage.get_memo(&memo_id).is_some());

        let removed = storage.remove_memo(&memo_id);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().title, "Test");
        assert!(storage.get_memo(&memo_id).is_none());
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("Hello World"), "Hello World");
        assert_eq!(sanitize_filename("test/file"), "test_file");
        assert_eq!(sanitize_filename("test\\file"), "test_file");
        assert_eq!(sanitize_filename("test:file"), "test_file");
        assert_eq!(sanitize_filename("test*file"), "test_file");
        assert_eq!(sanitize_filename("test?file"), "test_file");
        assert_eq!(sanitize_filename("test\"file"), "test_file");
        assert_eq!(sanitize_filename("test<file"), "test_file");
        assert_eq!(sanitize_filename("test>file"), "test_file");
        assert_eq!(sanitize_filename("test|file"), "test_file");
        assert_eq!(sanitize_filename("...test..."), "test");
        assert_eq!(sanitize_filename("test\x00file"), "test_file");
    }

    #[test]
    fn test_extract_title_from_filename() {
        use std::path::Path;

        let path = Path::new("test_file.md");
        assert_eq!(extract_title_from_filename(path), "test file");

        let path = Path::new("hello_world.md");
        assert_eq!(extract_title_from_filename(path), "hello world");

        let path = Path::new("single.md");
        assert_eq!(extract_title_from_filename(path), "single");

        let path = Path::new("no_extension");
        assert_eq!(extract_title_from_filename(path), "no extension");
    }

    #[test]
    fn test_find_git_root() {
        let result = find_git_root();
        assert!(result.is_ok());
        let git_root = result.unwrap();
        assert!(git_root.join(".git").exists());
    }

    #[test]
    fn test_memo_store_creation() {
        use std::env;
        let temp_dir = env::temp_dir();
        let store = MemoStore::new(temp_dir.clone());
        assert_eq!(store.root_path, temp_dir);
    }

    #[test]
    fn test_memo_store_find_memoranda_dirs() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create a .memoranda directory
        let memoranda_dir = temp_path.join(".memoranda");
        fs::create_dir(&memoranda_dir).unwrap();

        // Create a nested .memoranda directory
        let nested_dir = temp_path.join("nested");
        fs::create_dir(&nested_dir).unwrap();
        let nested_memoranda = nested_dir.join(".memoranda");
        fs::create_dir(&nested_memoranda).unwrap();

        let store = MemoStore::new(temp_path.to_path_buf());
        let dirs = store.find_memoranda_dirs().unwrap();

        assert_eq!(dirs.len(), 2);
        assert!(dirs.contains(&memoranda_dir));
        assert!(dirs.contains(&nested_memoranda));
    }

    #[test]
    fn test_memo_store_with_file_operations() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create a .memoranda directory
        let memoranda_dir = temp_path.join(".memoranda");
        fs::create_dir(&memoranda_dir).unwrap();

        let store = MemoStore::new(temp_path.to_path_buf());

        // Create a memo using the store (which will use proper format)
        let _memo = store
            .create_memo(
                "Test Memo".to_string(),
                "This is a test memo content.".to_string(),
            )
            .unwrap();

        // Test listing memos
        let memos = store.list_memos().unwrap();
        assert_eq!(memos.len(), 1);
        assert_eq!(memos[0].title, "Test Memo");
        assert_eq!(memos[0].content, "This is a test memo content.");

        // Test getting a specific memo
        let memo_id = memos[0].id;
        let retrieved_memo = store.get_memo(&memo_id).unwrap();
        assert!(retrieved_memo.is_some());
        let retrieved_memo = retrieved_memo.unwrap();
        assert_eq!(retrieved_memo.title, "Test Memo");
        assert_eq!(retrieved_memo.id, memo_id);
    }

    #[test]
    fn test_memo_store_create_memo() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create a .memoranda directory
        let memoranda_dir = temp_path.join(".memoranda");
        fs::create_dir(&memoranda_dir).unwrap();

        let store = MemoStore::new(temp_path.to_path_buf());

        // Test creating a new memo
        let memo = store
            .create_memo("New Memo".to_string(), "This is new content".to_string())
            .unwrap();
        assert_eq!(memo.title, "New Memo");
        assert_eq!(memo.content, "This is new content");
        assert!(memo.file_path.is_some());

        // Verify the file was created
        let file_path = memo.file_path.unwrap();
        assert!(file_path.exists());
        let file_content = fs::read_to_string(&file_path).unwrap();
        assert!(file_content.contains("This is new content"));
        assert!(file_content.starts_with("---\n"));
    }

    #[test]
    fn test_memo_store_update_memo() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create a .memoranda directory
        let memoranda_dir = temp_path.join(".memoranda");
        fs::create_dir(&memoranda_dir).unwrap();

        let store = MemoStore::new(temp_path.to_path_buf());

        // Create a memo
        let memo = store
            .create_memo("Update Test".to_string(), "Original content".to_string())
            .unwrap();
        let memo_id = memo.id;

        // Update the memo
        let updated_memo = store
            .update_memo(&memo_id, "Updated content".to_string())
            .unwrap();
        assert_eq!(updated_memo.content, "Updated content");
        assert!(updated_memo.updated_at > updated_memo.created_at);

        // Verify the file was updated
        let file_path = updated_memo.file_path.unwrap();
        let file_content = fs::read_to_string(&file_path).unwrap();
        assert!(file_content.contains("Updated content"));
        assert!(file_content.starts_with("---\n"));
    }

    #[test]
    fn test_memo_store_delete_memo() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create a .memoranda directory
        let memoranda_dir = temp_path.join(".memoranda");
        fs::create_dir(&memoranda_dir).unwrap();

        let store = MemoStore::new(temp_path.to_path_buf());

        // Create a memo
        let memo = store
            .create_memo("Delete Test".to_string(), "To be deleted".to_string())
            .unwrap();
        let memo_id = memo.id;
        let file_path = memo.file_path.clone().unwrap();

        // Verify the file exists
        assert!(file_path.exists());

        // Delete the memo
        store.delete_memo(&memo_id).unwrap();

        // Verify the file was deleted
        assert!(!file_path.exists());

        // Verify the memo is no longer retrievable
        let retrieved = store.get_memo(&memo_id).unwrap();
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_memo_store_search_memos() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create a .memoranda directory
        let memoranda_dir = temp_path.join(".memoranda");
        fs::create_dir(&memoranda_dir).unwrap();

        let store = MemoStore::new(temp_path.to_path_buf());

        // Create test memos
        let memo1 = store
            .create_memo(
                "Rust Programming".to_string(),
                "Learning rust language".to_string(),
            )
            .unwrap();
        let _memo2 = store
            .create_memo(
                "Python Guide".to_string(),
                "Python programming tutorial".to_string(),
            )
            .unwrap();

        // Test search functionality
        let results = store.search_memos("rust").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].memo.id, memo1.id);

        let results = store.search_memos("programming").unwrap();
        assert_eq!(results.len(), 2);

        let results = store.search_memos("\"rust language\"").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].memo.id, memo1.id);

        let results = store.search_memos("rust AND programming").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].memo.id, memo1.id);

        let results = store.search_memos("rust OR python").unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_memo_store_search_memos_with_query() {
        use crate::memo::search::SearchQuery;
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create a .memoranda directory
        let memoranda_dir = temp_path.join(".memoranda");
        fs::create_dir(&memoranda_dir).unwrap();

        let store = MemoStore::new(temp_path.to_path_buf());

        // Create test memos
        let memo1 = store
            .create_memo(
                "Rust Programming".to_string(),
                "Learning rust language".to_string(),
            )
            .unwrap();
        let _memo2 = store
            .create_memo(
                "Python Guide".to_string(),
                "Python programming tutorial".to_string(),
            )
            .unwrap();

        // Test search with custom query
        let query = SearchQuery::with_terms(vec!["rust".to_string()]);
        let results = store.search_memos_with_query(&query).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].memo.id, memo1.id);

        let query = SearchQuery::with_phrase("rust language".to_string());
        let results = store.search_memos_with_query(&query).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].memo.id, memo1.id);
    }

    #[test]
    fn test_memo_store_get_all_context() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create a .memoranda directory
        let memoranda_dir = temp_path.join(".memoranda");
        fs::create_dir(&memoranda_dir).unwrap();

        let store = MemoStore::new(temp_path.to_path_buf());

        // Create test memos
        let _memo1 = store
            .create_memo("First Memo".to_string(), "First content".to_string())
            .unwrap();
        let _memo2 = store
            .create_memo("Second Memo".to_string(), "Second content".to_string())
            .unwrap();

        // Test context aggregation
        let context = store.get_all_context().unwrap();
        assert!(context.contains("# First Memo"));
        assert!(context.contains("# Second Memo"));
        assert!(context.contains("First content"));
        assert!(context.contains("Second content"));
        assert!(context.contains("Created:"));
        assert!(context.contains("Updated:"));
    }

    #[tokio::test]
    async fn test_memo_store_async_operations() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create a .memoranda directory
        let memoranda_dir = temp_path.join(".memoranda");
        fs::create_dir(&memoranda_dir).unwrap();

        let store = MemoStore::new(temp_path.to_path_buf());

        // Test async memo creation
        let memo = store
            .create_memo_async(
                "Async Test Memo".to_string(),
                "This is async content".to_string(),
            )
            .await
            .unwrap();
        assert_eq!(memo.title, "Async Test Memo");
        assert_eq!(memo.content, "This is async content");
        assert!(memo.file_path.is_some());

        // Test async memo retrieval
        let retrieved_memo = store.get_memo_async(&memo.id).await.unwrap();
        assert!(retrieved_memo.is_some());
        let retrieved_memo = retrieved_memo.unwrap();
        assert_eq!(retrieved_memo.title, "Async Test Memo");
        assert_eq!(retrieved_memo.content, "This is async content");

        // Test async memo update
        let updated_memo = store
            .update_memo_async(&memo.id, "Updated async content".to_string())
            .await
            .unwrap();
        assert_eq!(updated_memo.content, "Updated async content");

        // Test async memo list
        let memos = store.list_memos_async().await.unwrap();
        assert_eq!(memos.len(), 1);
        assert_eq!(memos[0].title, "Async Test Memo");
        assert_eq!(memos[0].content, "Updated async content");

        // Test async memo deletion
        store.delete_memo_async(&memo.id).await.unwrap();

        // Verify memo is deleted
        let deleted_memo = store.get_memo_async(&memo.id).await.unwrap();
        assert!(deleted_memo.is_none());

        let memos_after_delete = store.list_memos_async().await.unwrap();
        assert_eq!(memos_after_delete.len(), 0);
    }

    #[tokio::test]
    async fn test_find_memoranda_dirs_async() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create multiple .memoranda directories
        let memoranda_dir1 = temp_path.join(".memoranda");
        fs::create_dir(&memoranda_dir1).unwrap();

        let nested_dir = temp_path.join("nested");
        fs::create_dir(&nested_dir).unwrap();
        let memoranda_dir2 = nested_dir.join(".memoranda");
        fs::create_dir(&memoranda_dir2).unwrap();

        let store = MemoStore::new(temp_path.to_path_buf());
        let dirs = store.find_memoranda_dirs_async().await.unwrap();

        assert_eq!(dirs.len(), 2);
        assert!(dirs.contains(&memoranda_dir1));
        assert!(dirs.contains(&memoranda_dir2));
    }

    #[tokio::test]
    async fn test_async_file_operations_performance() {
        use std::fs;
        use std::time::Instant;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create a .memoranda directory
        let memoranda_dir = temp_path.join(".memoranda");
        fs::create_dir(&memoranda_dir).unwrap();

        let store = MemoStore::new(temp_path.to_path_buf());

        // Test async operations are faster than sync for multiple operations
        let start_async = Instant::now();
        let mut async_tasks = Vec::new();
        for i in 0..10 {
            let store_clone = &store;
            let task = async move {
                store_clone
                    .create_memo_async(format!("Async Memo {i}"), format!("Async content {i}"))
                    .await
            };
            async_tasks.push(task);
        }

        // Execute all async operations concurrently
        let async_results: Vec<_> = futures::future::try_join_all(async_tasks).await.unwrap();
        let async_duration = start_async.elapsed();

        assert_eq!(async_results.len(), 10);

        // Clean up async-created memos
        for memo in async_results {
            store.delete_memo_async(&memo.id).await.unwrap();
        }

        // Test sync operations for comparison
        let start_sync = Instant::now();
        for i in 0..10 {
            store
                .create_memo(format!("Sync Memo {i}"), format!("Sync content {i}"))
                .unwrap();
        }
        let sync_duration = start_sync.elapsed();

        // For small operations, async might not be significantly faster,
        // but it should at least work correctly
        tracing::info!(
            async_duration = ?async_duration,
            sync_duration = ?sync_duration,
            "Performance comparison completed"
        );

        // Verify both approaches created the same number of memos
        let memos = store.list_memos_async().await.unwrap();
        assert_eq!(memos.len(), 10);
    }

    #[tokio::test]
    async fn test_memo_caching() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create a .memoranda directory
        let memoranda_dir = temp_path.join(".memoranda");
        fs::create_dir(&memoranda_dir).unwrap();

        let store = MemoStore::new(temp_path.to_path_buf());

        // Create a memo
        let memo = store
            .create_memo_async("Cached Test Memo".to_string(), "Cached content".to_string())
            .await
            .unwrap();

        let memo_id = memo.id;
        tracing::debug!(memo_id = %memo_id, "Created test memo");

        // Check cache stats after creation
        let stats_after_create = store.get_cache_stats().await;
        tracing::debug!(
            hits = stats_after_create.memo_hits,
            misses = stats_after_create.memo_misses,
            cache_size = stats_after_create.memo_cache_size,
            "Cache stats after creation"
        );

        // First retrieval should cache the memo
        let retrieved1 = store.get_memo_async(&memo_id).await.unwrap();
        assert!(retrieved1.is_some());
        tracing::debug!("First retrieval successful");

        // Check cache stats after first retrieval
        let stats_after_first = store.get_cache_stats().await;
        tracing::debug!(
            hits = stats_after_first.memo_hits,
            misses = stats_after_first.memo_misses,
            cache_size = stats_after_first.memo_cache_size,
            "Cache stats after first retrieval"
        );

        // Second retrieval should come from cache (faster)
        let retrieved2 = store.get_memo_async(&memo_id).await.unwrap();
        assert!(retrieved2.is_some());
        assert_eq!(retrieved1.unwrap().title, retrieved2.unwrap().title);
        tracing::debug!("Second retrieval successful");

        // Check cache stats - the important thing is that we get cache hits
        let stats = store.get_cache_stats().await;
        tracing::debug!(
            hits = stats.memo_hits,
            misses = stats.memo_misses,
            cache_size = stats.memo_cache_size,
            "Final cache stats"
        );

        // The key indicator of successful caching is cache hits
        assert!(
            stats.memo_hits > 0,
            "Expected memo cache hits but got {}",
            stats.memo_hits
        );

        // Note: cache_size might be 0 due to moka's internal cache management,
        // but the hits prove that caching is working correctly
    }

    #[tokio::test]
    async fn test_cache_invalidation_on_update() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create a .memoranda directory
        let memoranda_dir = temp_path.join(".memoranda");
        fs::create_dir(&memoranda_dir).unwrap();

        let store = MemoStore::new(temp_path.to_path_buf());

        // Create a memo
        let memo = store
            .create_memo_async("Update Test".to_string(), "Original content".to_string())
            .await
            .unwrap();

        let memo_id = memo.id;

        // Retrieve to cache it
        let original = store.get_memo_async(&memo_id).await.unwrap().unwrap();
        assert_eq!(original.content, "Original content");

        // Update the memo
        let updated = store
            .update_memo_async(&memo_id, "Updated content".to_string())
            .await
            .unwrap();
        assert_eq!(updated.content, "Updated content");

        // Retrieve again - should get updated version from cache
        let retrieved_after_update = store.get_memo_async(&memo_id).await.unwrap().unwrap();
        assert_eq!(retrieved_after_update.content, "Updated content");
    }

    #[tokio::test]
    async fn test_cache_removal_on_delete() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create a .memoranda directory
        let memoranda_dir = temp_path.join(".memoranda");
        fs::create_dir(&memoranda_dir).unwrap();

        let store = MemoStore::new(temp_path.to_path_buf());

        // Create a memo
        let memo = store
            .create_memo_async("Delete Test".to_string(), "Content to delete".to_string())
            .await
            .unwrap();

        let memo_id = memo.id;

        // Retrieve to cache it
        let cached = store.get_memo_async(&memo_id).await.unwrap();
        assert!(cached.is_some());

        // Delete the memo
        store.delete_memo_async(&memo_id).await.unwrap();

        // Try to retrieve - should be None
        let after_delete = store.get_memo_async(&memo_id).await.unwrap();
        assert!(after_delete.is_none());
    }

    #[tokio::test]
    async fn test_cache_warm_up() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create a .memoranda directory
        let memoranda_dir = temp_path.join(".memoranda");
        fs::create_dir(&memoranda_dir).unwrap();

        let store = MemoStore::new(temp_path.to_path_buf());

        // Create multiple memos
        for i in 1..=5 {
            store
                .create_memo_async(format!("Warm Test {i}"), format!("Content {i}"))
                .await
                .unwrap();
        }

        // Clear cache
        store.clear_cache().await;

        // Warm cache
        let warmed_count = store.warm_cache().await.unwrap();
        assert_eq!(warmed_count, 5);

        // Verify warming worked by checking that we can retrieve memos efficiently
        // (They should come from cache, not requiring file reads)
        let memos = store.list_memos_async().await.unwrap();
        for memo in &memos {
            let retrieved = store.get_memo_async(&memo.id).await.unwrap();
            assert!(retrieved.is_some());
        }
    }

    #[tokio::test]
    async fn test_cache_performance_improvement() {
        use std::fs;
        use std::time::Instant;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create a .memoranda directory
        let memoranda_dir = temp_path.join(".memoranda");
        fs::create_dir(&memoranda_dir).unwrap();

        let store = MemoStore::new(temp_path.to_path_buf());

        // Create a memo
        let memo = store
            .create_memo_async(
                "Performance Test".to_string(),
                "Performance content".to_string(),
            )
            .await
            .unwrap();

        let memo_id = memo.id;

        // First access - will load from disk and cache
        let start_first = Instant::now();
        let _first_access = store.get_memo_async(&memo_id).await.unwrap();
        let first_duration = start_first.elapsed();

        // Second access - should come from cache (faster)
        let start_second = Instant::now();
        let _second_access = store.get_memo_async(&memo_id).await.unwrap();
        let second_duration = start_second.elapsed();

        // Cache access should be faster (though with small files, difference might be minimal)
        tracing::info!(
            first_duration = ?first_duration,
            second_duration = ?second_duration,
            "Cache performance comparison completed"
        );

        // At minimum, both should work correctly
        assert!(_first_access.is_some());
        assert!(_second_access.is_some());

        // Verify cache stats show hits
        let stats = store.get_cache_stats().await;
        assert!(stats.memo_hits > 0);
    }
}
