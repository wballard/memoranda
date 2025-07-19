use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use ulid::Ulid;

// Validation constants
const MAX_TITLE_LENGTH: usize = 255;
const MAX_CONTENT_LENGTH: usize = 1024 * 1024; // 1MB
const MIN_TITLE_LENGTH: usize = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MemoId(Ulid);

impl MemoId {
    #[must_use]
    pub fn new() -> Self {
        Self(Ulid::new())
    }

    pub fn from_ulid(ulid: Ulid) -> Self {
        Self(ulid)
    }
}

impl Default for MemoId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for MemoId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memo {
    pub id: MemoId,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,
    pub file_path: Option<PathBuf>,
}

impl Memo {
    /// Creates a new memo with the given title and content.
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - Title is empty or exceeds maximum length
    /// - Content exceeds maximum length
    #[must_use]
    pub fn new(title: String, content: String) -> Result<Self> {
        Self::validate_title(&title)?;
        Self::validate_content(&content)?;

        let now = Utc::now();
        Ok(Self {
            id: MemoId::new(),
            title,
            content,
            created_at: now,
            updated_at: now,
            tags: Vec::new(),
            file_path: None,
        })
    }

    /// Creates a new memo with the given title, content, and optional file path.
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - Title is empty or exceeds maximum length
    /// - Content exceeds maximum length
    #[must_use]
    pub fn with_file_path(
        title: String,
        content: String,
        file_path: Option<PathBuf>,
    ) -> Result<Self> {
        Self::validate_title(&title)?;
        Self::validate_content(&content)?;

        let now = Utc::now();
        Ok(Self {
            id: MemoId::new(),
            title,
            content,
            created_at: now,
            updated_at: now,
            tags: Vec::new(),
            file_path,
        })
    }

    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    /// Updates the memo's content and sets the updated timestamp.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the content exceeds the maximum allowed length.
    pub fn update_content(&mut self, content: String) -> Result<()> {
        Self::validate_content(&content)?;
        self.content = content;
        self.updated_at = Utc::now();
        Ok(())
    }

    fn validate_title(title: &str) -> Result<()> {
        if title.is_empty() {
            return Err(anyhow!("Title cannot be empty"));
        }
        if title.len() < MIN_TITLE_LENGTH {
            return Err(anyhow!(
                "Title must be at least {} characters long",
                MIN_TITLE_LENGTH
            ));
        }
        if title.len() > MAX_TITLE_LENGTH {
            return Err(anyhow!(
                "Title cannot exceed {} characters",
                MAX_TITLE_LENGTH
            ));
        }
        if title.trim().is_empty() {
            return Err(anyhow!("Title cannot be only whitespace"));
        }
        Ok(())
    }

    fn validate_content(content: &str) -> Result<()> {
        if content.len() > MAX_CONTENT_LENGTH {
            return Err(anyhow!(
                "Content cannot exceed {} bytes",
                MAX_CONTENT_LENGTH
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memo_id_creation() {
        let id1 = MemoId::new();
        let id2 = MemoId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_memo_id_display() {
        let id = MemoId::new();
        let display_str = format!("{id}");
        assert!(!display_str.is_empty());
    }

    #[test]
    fn test_memo_creation() {
        let memo = Memo::new("Test Title".to_string(), "Test content".to_string()).unwrap();
        assert_eq!(memo.title, "Test Title");
        assert_eq!(memo.content, "Test content");
        assert_eq!(memo.created_at, memo.updated_at);
        assert!(memo.tags.is_empty());
    }

    #[test]
    fn test_memo_add_tag() {
        let mut memo = Memo::new("Test".to_string(), "Content".to_string()).unwrap();
        memo.add_tag("tag1".to_string());
        memo.add_tag("tag2".to_string());
        memo.add_tag("tag1".to_string()); // Duplicate should not be added

        assert_eq!(memo.tags.len(), 2);
        assert!(memo.tags.contains(&"tag1".to_string()));
        assert!(memo.tags.contains(&"tag2".to_string()));
    }

    #[test]
    fn test_memo_update_content() {
        let mut memo = Memo::new("Test".to_string(), "Original content".to_string()).unwrap();
        let original_updated_at = memo.updated_at;

        std::thread::sleep(std::time::Duration::from_millis(1));
        memo.update_content("New content".to_string()).unwrap();

        assert_eq!(memo.content, "New content");
        assert!(memo.updated_at > original_updated_at);
    }

    #[test]
    fn test_memo_validation_empty_title() {
        let result = Memo::new("".to_string(), "Content".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_memo_validation_whitespace_title() {
        let result = Memo::new("   ".to_string(), "Content".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_memo_validation_long_title() {
        let long_title = "a".repeat(256);
        let result = Memo::new(long_title, "Content".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_memo_validation_large_content() {
        let large_content = "a".repeat(1024 * 1024 + 1);
        let result = Memo::new("Valid Title".to_string(), large_content);
        assert!(result.is_err());
    }

    #[test]
    fn test_memo_validation_valid_content() {
        let valid_content = "a".repeat(1024 * 1024); // Exactly 1MB
        let result = Memo::new("Valid Title".to_string(), valid_content);
        assert!(result.is_ok());
    }
}
