use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::modifiers::ModifiersConfig;
use crate::models::item::{Item, QuantityBuffer};
use crate::models::packing_list::PackingList;
use crate::models::weekdays::Weekdays;

// The input configuration file's schema (spec 013: strict — every
// field required as written, unknown fields rejected; the program
// loads only the current format)
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AppConfig {
    pub(crate) categories: Vec<String>,
    pub(crate) items: Vec<SavedItem>,
    pub(crate) weekdays: u32,
    pub(crate) travel_style: String,
    pub(crate) weather: String,
    pub(crate) modifiers: ModifiersConfig,
    /// Name of the trip file to continue by default
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) last_trip_file: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// One item as persisted in the config file: name, category
/// memberships, the quantity mode (fixed items keep their value),
/// and the per-item modifiers (spec 010). Strict per spec 013:
/// `categories` is required; `quantity` is optional-in-format
/// (automatic items legitimately omit it); unknown fields reject.
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
    /// The per-day rate (spec 010); absent in pre-feature files,
    /// loading as the default 1.0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) rate: Option<f32>,
    /// The flat reserve buffer (spec 010); absent = no reserve
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) reserve: Option<u32>,
    /// The multiplier buffer (spec 010); absent = no multiplier.
    /// Exactly one buffer kind is persisted per item (SC-007).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) buffer_multiplier: Option<f32>,
}

/// The travel style of a first-run default config (spec 013: this is
/// the initial value, not a load-time fallback).
pub(crate) fn default_travel_style_name() -> String {
    "standard".to_string()
}

/// The weather of a first-run default config (spec 013: the initial
/// value, not a load-time fallback).
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

/// Load the input config file into a packing list and the trip
/// parameters; missing optional fields fall back to their defaults.
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
        // Per-item modifiers (spec 010): absent fields load as the
        // defaults (rate 1.0, no buffer) — pre-feature files unchanged
        item.rate = saved_item.rate.unwrap_or(1.0);
        item.buffer = match (saved_item.reserve, saved_item.buffer_multiplier) {
            (Some(n), None) => QuantityBuffer::Reserve(n),
            (None, Some(m)) => QuantityBuffer::Multiplier(m),
            _ => QuantityBuffer::None,
        };
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

/// Create the default config file (built-in items and modifiers) and
/// return the packing list and trip parameters built from it.
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
                    rate: None,
                    reserve: None,
                    buffer_multiplier: None,
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
