use serde::{Deserialize, Serialize};

use super::item::Item;

/// The packing list: the item catalog with per-trip state (selection,
/// packed, quantities).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct PackingList {
    pub(crate) items: Vec<Item>,
}

impl PackingList {
    pub(crate) fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub(crate) fn add_item(&mut self, item: Item) {
        self.items.push(item);
    }
}
