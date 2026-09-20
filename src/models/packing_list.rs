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
            id: format!(
                "list_{}",
                chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
            ),
            name,
            items: Vec::new(),
            file_path: None,
        }
    }

    pub fn add_item(&mut self, item: Item) {
        self.items.push(item);
    }
}
