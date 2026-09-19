use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackingListConfig {
    pub name: String,
    pub default_weekdays: u32,
    pub default_travel_style: String,
    pub storage_path: PathBuf,
}

impl Default for PackingListConfig {
    fn default() -> Self {
        Self {
            name: "Default Packing List".to_string(),
            default_weekdays: 7,
            default_travel_style: "standard".to_string(),
            storage_path: dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("~/.config"))
                .join("pack-liste"),
        }
    }
}
