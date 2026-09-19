use anyhow::{Context, Result};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use crate::models::packing_list::PackingList;

pub struct PackingService {
    storage_path: PathBuf,
}

impl PackingService {
    pub fn new(storage_path: PathBuf) -> Result<Self> {
        fs::create_dir_all(&storage_path)
            .context(format!("Failed to create storage directory: {:?}", storage_path))?;
        Ok(Self { storage_path })
    }

    pub fn create_packing_list(&self, name: String) -> Result<PackingList> {
        if name.is_empty() || name.len() > 50 {
            return Err(anyhow::anyhow!("Packing list name must be between 1 and 50 characters"));
        }
        Ok(PackingList::new(name))
    }

    pub fn rename_packing_list(&mut self, list: &mut PackingList, new_name: String) -> Result<()> {
        if new_name.is_empty() || new_name.len() > 50 {
            return Err(anyhow::anyhow!("Packing list name must be between 1 and 50 characters"));
        }
        list.name = new_name;
        Ok(())
    }

    pub fn export_packing_list(&self, list: &PackingList) -> Result<PathBuf> {
        let file_name = format!("{}.json", list.name);
        let file_path = self.storage_path.join(file_name);
        
        let json = serde_json::to_string_pretty(list)
            .context("Failed to serialize packing list")?;
        
        let mut file = File::create(&file_path)
            .context(format!("Failed to create file: {:?}", file_path))?;
        file.write_all(json.as_bytes())
            .context(format!("Failed to write to file: {:?}", file_path))?;
        
        Ok(file_path)
    }

    pub fn import_packing_list(&self, file_path: &PathBuf) -> Result<PackingList> {
        let content = fs::read_to_string(file_path)
            .context(format!("Failed to read file: {:?}", file_path))?;
        
        let list: PackingList = serde_json::from_str(&content)
            .context(format!("Failed to deserialize packing list from: {:?}", file_path))?;
        
        Ok(list)
    }
}
