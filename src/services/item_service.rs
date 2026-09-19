use anyhow::Result;

use crate::models::item::Item;

pub struct ItemService;

impl ItemService {
    pub fn new() -> Self {
        Self
    }

    pub fn update_quantity(&self, item: &mut Item, new_quantity: u32) -> Result<()> {
        if new_quantity == 0 {
            return Err(anyhow::anyhow!("Quantity must be at least 1"));
        }
        item.set_quantity(new_quantity).map_err(|e| anyhow::anyhow!(e))?;
        Ok(())
    }

    pub fn toggle_packed(&self, item: &mut Item) -> Result<()> {
        item.toggle_packed();
        Ok(())
    }
}
