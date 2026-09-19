use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::item::Item;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackingList {
    pub id: String,
    pub name: String,
    pub items: Vec<Item>,
    pub file_path: Option<PathBuf>,
}

impl PackingList {
    pub fn new(name: String) -> Self {
        Self {
            id: format!("list_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)),
            name,
            items: Vec::new(),
            file_path: None,
        }
    }

    pub fn add_item(&mut self, item: Item) {
        self.items.push(item);
    }

    pub fn remove_item(&mut self, item_id: &str) -> Option<Item> {
        let index = self.items.iter().position(|i| i.id == item_id);
        index.map(|i| self.items.remove(i))
    }

    pub fn get_item_mut(&mut self, item_id: &str) -> Option<&mut Item> {
        self.items.iter_mut().find(|i| i.id == item_id)
    }

    pub fn total_items(&self) -> usize {
        self.items.len()
    }

    pub fn packed_items(&self) -> usize {
        self.items.iter().filter(|i| i.packed).count()
    }

    pub fn unpacked_items(&self) -> usize {
        self.total_items() - self.packed_items()
    }
}
