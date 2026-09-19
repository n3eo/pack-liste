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
}

impl Item {
    pub fn new(
        id: String,
        name: String,
        category: String,
        base_quantity: u32,
    ) -> Self {
        Self {
            id,
            name,
            category,
            base_quantity,
            quantity: base_quantity,
            packed: false,
            editable: true,
        }
    }

    pub fn toggle_packed(&mut self) {
        self.packed = !self.packed;
    }

    pub fn set_quantity(&mut self, quantity: u32) -> Result<(), String> {
        if quantity == 0 {
            return Err("Quantity must be at least 1".to_string());
        }
        self.quantity = quantity;
        Ok(())
    }
}
