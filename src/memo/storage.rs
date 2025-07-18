use anyhow::Result;
use std::collections::HashMap;
use tracing::info;
use uuid::Uuid;

use super::models::Memo;

pub struct MemoStorage {
    memos: HashMap<Uuid, Memo>,
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

    pub fn get_memo(&self, id: &Uuid) -> Option<&Memo> {
        self.memos.get(id)
    }

    pub fn list_memos(&self) -> Vec<&Memo> {
        self.memos.values().collect()
    }

    pub fn remove_memo(&mut self, id: &Uuid) -> Option<Memo> {
        info!("Removing memo: {}", id);
        self.memos.remove(id)
    }
}