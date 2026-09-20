use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct Item {
    pub(crate) name: String,
    /// The categories this item belongs to; at least one. The first entry
    /// is the primary category (used for display grouping in trip files).
    pub(crate) categories: Vec<String>,
    pub(crate) quantity: u32,
    pub(crate) packed: bool,
    pub(crate) selected: bool,
    /// When false, the quantity is fixed and not recalculated from
    /// trip parameters (e.g. a toothbrush always stays at 1)
    pub(crate) auto_quantity: bool,
}

impl Item {
    pub(crate) fn new(name: String, categories: Vec<String>, quantity: u32) -> Self {
        Self {
            name,
            categories,
            quantity,
            packed: false,
            selected: true,
            auto_quantity: true,
        }
    }

    /// The item's primary (first) category.
    pub(crate) fn primary_category(&self) -> &str {
        self.categories.first().map(String::as_str).unwrap_or("")
    }

    /// Whether the item belongs to the given category.
    pub(crate) fn in_category(&self, category: &str) -> bool {
        self.categories.iter().any(|c| c == category)
    }
}
