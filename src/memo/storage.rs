use anyhow::Result;
use std::collections::HashMap;
use tracing::info;

use super::models::{Memo, MemoId};

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
        let memo = Memo::new("Test".to_string(), "Content".to_string());
        let memo_id = memo.id;
        
        storage.store_memo(memo).unwrap();
        
        let retrieved = storage.get_memo(&memo_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().title, "Test");
    }

    #[test]
    fn test_list_memos() {
        let mut storage = MemoStorage::new();
        let memo1 = Memo::new("Test1".to_string(), "Content1".to_string());
        let memo2 = Memo::new("Test2".to_string(), "Content2".to_string());
        
        storage.store_memo(memo1).unwrap();
        storage.store_memo(memo2).unwrap();
        
        let memos = storage.list_memos();
        assert_eq!(memos.len(), 2);
    }

    #[test]
    fn test_remove_memo() {
        let mut storage = MemoStorage::new();
        let memo = Memo::new("Test".to_string(), "Content".to_string());
        let memo_id = memo.id;
        
        storage.store_memo(memo).unwrap();
        assert!(storage.get_memo(&memo_id).is_some());
        
        let removed = storage.remove_memo(&memo_id);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().title, "Test");
        assert!(storage.get_memo(&memo_id).is_none());
    }
}

