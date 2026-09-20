use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Item {
    pub id: String,
    pub name: String,
    /// The categories this item belongs to; at least one. The first entry
    /// is the primary category (used for display grouping in trip files).
    pub categories: Vec<String>,
    pub base_quantity: u32,
    pub quantity: u32,
    pub packed: bool,
    pub editable: bool,
    /// When false, the quantity is fixed and not recalculated from
    /// trip parameters (e.g. a toothbrush always stays at 1)
    pub auto_quantity: bool,
}

impl Item {
    pub fn new(id: String, name: String, categories: Vec<String>, base_quantity: u32) -> Self {
        Self {
            id,
            name,
            categories,
            base_quantity,
            quantity: base_quantity,
            packed: false,
            editable: true,
            auto_quantity: true,
        }
    }

    /// The item's primary (first) category.
    pub fn primary_category(&self) -> &str {
        self.categories.first().map(String::as_str).unwrap_or("")
    }

    /// Whether the item belongs to the given category.
    pub fn in_category(&self, category: &str) -> bool {
        self.categories.iter().any(|c| c == category)
    }
}
