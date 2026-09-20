use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub category: String,
    pub base_quantity: u32,
    pub quantity: u32,
    pub packed: bool,
    pub editable: bool,
    /// When false, the quantity is fixed and not recalculated from
    /// trip parameters (e.g. a toothbrush always stays at 1)
    pub auto_quantity: bool,
}

impl Item {
    pub fn new(id: String, name: String, category: String, base_quantity: u32) -> Self {
        Self {
            id,
            name,
            category,
            base_quantity,
            quantity: base_quantity,
            packed: false,
            editable: true,
            auto_quantity: true,
        }
    }
}
