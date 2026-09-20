use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::modifiers::ModifiersConfig;
use crate::models::item::Item;
use crate::models::packing_list::PackingList;
use crate::models::weekdays::Weekdays;

// Config structure for saving to file
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct AppConfig {
    pub(crate) categories: Vec<String>,
    pub(crate) items: Vec<SavedItem>,
    #[serde(default)]
    pub(crate) weekdays: u32,
    #[serde(default = "default_travel_style_name")]
    pub(crate) travel_style: String,
    #[serde(default = "default_weather_name")]
    pub(crate) weather: String,
    #[serde(default)]
    pub(crate) modifiers: ModifiersConfig,
    /// Name of the trip file to continue by default
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) last_trip_file: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SavedItem {
    pub(crate) name: String,
    /// The item's categories; at least one. A legacy single `category:`
    /// string is lifted into a one-element list on load.
    pub(crate) categories: Vec<String>,
    /// Only present for fixed-quantity items (auto_quantity: false);
    /// auto items are recalculated on every load
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) quantity: Option<u32>,
    /// When false, the quantity is fixed and never recalculated
    pub(crate) auto_quantity: bool,
}

impl<'de> serde::Deserialize<'de> for SavedItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{Error, MapAccess, Visitor};

        struct SavedItemVisitor;

        impl<'de> Visitor<'de> for SavedItemVisitor {
            type Value = SavedItem;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a saved item mapping")
            }

            fn visit_map<A>(self, mut map: A) -> Result<SavedItem, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut name: Option<String> = None;
                let mut categories: Option<Vec<String>> = None;
                let mut category: Option<String> = None;
                let mut quantity: Option<u32> = None;
                let mut auto_quantity = true;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "name" => name = Some(map.next_value()?),
                        "categories" => categories = Some(map.next_value()?),
                        // Legacy single category (pre multi-category format)
                        "category" => category = Some(map.next_value()?),
                        "quantity" => quantity = map.next_value()?,
                        "auto_quantity" => auto_quantity = map.next_value()?,
                        _ => {
                            let _: serde_yaml::Value = map.next_value()?;
                        }
                    }
                }

                let name = name.ok_or_else(|| A::Error::missing_field("name"))?;
                // Prefer the categories list; lift a legacy single category
                let categories = match categories {
                    Some(cs) if !cs.is_empty() => cs,
                    _ => category.map(|c| vec![c]).unwrap_or_default(),
                };

                Ok(SavedItem {
                    name,
                    categories,
                    quantity,
                    auto_quantity,
                })
            }
        }

        deserializer.deserialize_map(SavedItemVisitor)
    }
}

pub(crate) fn default_travel_style_name() -> String {
    "standard".to_string()
}

pub(crate) fn default_weather_name() -> String {
    "mixed".to_string()
}

/// Trip parameters loaded from (or defaulted into) the config file
pub(crate) struct TripParams {
    pub(crate) weekdays: Weekdays,
    pub(crate) travel_style: String,
    pub(crate) weather: String,
    pub(crate) modifiers: ModifiersConfig,
    pub(crate) last_trip_file: Option<String>,
}

pub(crate) fn load_config(config_file: &PathBuf) -> Result<(PackingList, TripParams)> {
    let yaml_content = fs::read_to_string(config_file).context("Failed to read config file")?;

    let config: AppConfig =
        serde_yaml::from_str(&yaml_content).context("Failed to parse config file")?;

    // Create packing list from config
    let mut packing_list = PackingList::new();

    for saved_item in config.items {
        let auto = saved_item.auto_quantity;
        let fixed_quantity = if auto {
            1
        } else {
            saved_item.quantity.unwrap_or(1)
        };
        let mut item = Item::new(saved_item.name, saved_item.categories, fixed_quantity);
        item.auto_quantity = auto;
        packing_list.add_item(item);
    }

    let weekdays = Weekdays::new(config.weekdays).unwrap_or_default();
    let trip_params = TripParams {
        weekdays,
        travel_style: config.travel_style,
        weather: config.weather,
        modifiers: config.modifiers,
        last_trip_file: config.last_trip_file,
    };

    Ok((packing_list, trip_params))
}

pub(crate) fn create_default_config(config_file: &PathBuf) -> Result<(PackingList, TripParams)> {
    // Create default packing list
    let mut packing_list = PackingList::new();

    // Add default items
    let default_items = vec![
        ("T-Shirt", "Clothing", 5),
        ("Pants", "Clothing", 3),
        ("Socks", "Clothing", 7),
        ("Underwear", "Clothing", 7),
        ("Jacket", "Clothing", 1),
        ("Toothbrush", "Toiletries", 1),
        ("Toothpaste", "Toiletries", 1),
        ("Shampoo", "Toiletries", 1),
        ("Passport", "Documents", 1),
        ("Tickets", "Documents", 1),
        ("Phone", "Electronics", 1),
        ("Charger", "Electronics", 1),
        ("Shoes", "Footwear", 1),
        ("Hat", "Accessories", 1),
        ("Sunglasses", "Accessories", 1),
    ];

    for (name, category, quantity) in default_items {
        packing_list.add_item(Item::new(
            name.to_string(),
            vec![category.to_string()],
            quantity,
        ));
    }

    // Default trip parameters
    let trip_params = TripParams {
        weekdays: Weekdays::default(),
        travel_style: default_travel_style_name(),
        weather: default_weather_name(),
        modifiers: ModifiersConfig::default(),
        last_trip_file: None,
    };

    // Save config
    let config = AppConfig {
        categories: vec![
            "Clothing".to_string(),
            "Toiletries".to_string(),
            "Documents".to_string(),
            "Electronics".to_string(),
            "Footwear".to_string(),
            "Accessories".to_string(),
            "Miscellaneous".to_string(),
        ],
        items: packing_list
            .items
            .iter()
            .map(|item| {
                let auto = item.auto_quantity;
                SavedItem {
                    name: item.name.clone(),
                    categories: item.categories.clone(),
                    quantity: if auto { None } else { Some(item.quantity) },
                    auto_quantity: auto,
                }
            })
            .collect(),
        weekdays: trip_params.weekdays.get(),
        travel_style: trip_params.travel_style.clone(),
        weather: trip_params.weather.clone(),
        modifiers: trip_params.modifiers.clone(),
        last_trip_file: None,
    };

    let yaml = serde_yaml::to_string(&config).context("Failed to serialize default config")?;

    if let Some(parent) = config_file.parent() {
        fs::create_dir_all(parent).context("Failed to create config directory")?;
    }

    fs::write(config_file, yaml).context("Failed to write default config file")?;

    Ok((packing_list, trip_params))
}

/// Select every item of a fresh or empty packing list so a new trip starts
/// fully pre-selected
pub(crate) fn select_all_items(packing_list: &mut PackingList) {
    for item in &mut packing_list.items {
        item.selected = true;
        item.packed = false;
    }
}

/// Resolve a user-entered trip file path:
/// - `~` and `~/...` expand to the home directory
/// - relative paths resolve against the current working directory
/// - absolute paths are kept as-is
pub(crate) fn resolve_trip_path(input: &str) -> PathBuf {
    let trimmed = input.trim();
    if let Some(rest) = trimmed.strip_prefix("~/")
        && let Some(home) = dirs::home_dir()
    {
        return home.join(rest);
    } else if trimmed == "~"
        && let Some(home) = dirs::home_dir()
    {
        return home;
    }
    let path = Path::new(trimmed);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    }
}
