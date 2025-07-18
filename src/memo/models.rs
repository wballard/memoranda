use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use ulid::Ulid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MemoId(Ulid);

impl MemoId {
    pub fn new() -> Self {
        Self(Ulid::new())
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
    pub fn new(title: String, content: String) -> Self {
        let now = Utc::now();
        Self {
            id: MemoId::new(),
            title,
            content,
            created_at: now,
            updated_at: now,
            tags: Vec::new(),
            file_path: None,
        }
    }

    pub fn with_file_path(title: String, content: String, file_path: Option<PathBuf>) -> Self {
        let now = Utc::now();
        Self {
            id: MemoId::new(),
            title,
            content,
            created_at: now,
            updated_at: now,
            tags: Vec::new(),
            file_path,
        }
    }

    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    pub fn update_content(&mut self, content: String) {
        self.content = content;
        self.updated_at = Utc::now();
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
        let display_str = format!("{}", id);
        assert!(!display_str.is_empty());
    }

    #[test]
    fn test_memo_creation() {
        let memo = Memo::new("Test Title".to_string(), "Test content".to_string());
        assert_eq!(memo.title, "Test Title");
        assert_eq!(memo.content, "Test content");
        assert_eq!(memo.created_at, memo.updated_at);
        assert!(memo.tags.is_empty());
    }

    #[test]
    fn test_memo_add_tag() {
        let mut memo = Memo::new("Test".to_string(), "Content".to_string());
        memo.add_tag("tag1".to_string());
        memo.add_tag("tag2".to_string());
        memo.add_tag("tag1".to_string()); // Duplicate should not be added
        
        assert_eq!(memo.tags.len(), 2);
        assert!(memo.tags.contains(&"tag1".to_string()));
        assert!(memo.tags.contains(&"tag2".to_string()));
    }

    #[test]
    fn test_memo_update_content() {
        let mut memo = Memo::new("Test".to_string(), "Original content".to_string());
        let original_updated_at = memo.updated_at;
        
        std::thread::sleep(std::time::Duration::from_millis(1));
        memo.update_content("New content".to_string());
        
        assert_eq!(memo.content, "New content");
        assert!(memo.updated_at > original_updated_at);
    }
}

