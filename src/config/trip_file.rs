use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// A packing list file: the compiled, persisted state of one trip.
///
/// This is separate from the input config (config.yaml). It stores the trip
/// parameters and the selected items (with packed state) so a trip can be
/// continued later. The file is human-readable YAML and can be edited by
/// hand; on load it is reconciled against the current input config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TripFile {
    /// Trip parameters at the time of the last save
    pub weekdays: u32,
    pub travel_style: String,
    pub weather: String,
    /// The compiled packing list: selected items with packed state.
    /// Quantities are not persisted; they are recalculated on load.
    #[serde(default)]
    pub items: Vec<TripItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TripItem {
    pub name: String,
    pub category: String,
    #[serde(default)]
    pub packed: bool,
}

impl TripFile {
    /// Load a trip file. Returns None if the file does not exist yet
    /// (a new trip).
    pub fn load(path: &Path) -> Result<Option<Self>> {
        if !path.exists() {
            return Ok(None);
        }
        let yaml = fs::read_to_string(path)
            .with_context(|| format!("Failed to read trip file {}", path.display()))?;
        let trip: TripFile = serde_yaml::from_str(&yaml)
            .with_context(|| format!("Failed to parse trip file {}", path.display()))?;
        Ok(Some(trip))
    }

    /// Save the trip file in place.
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {}", parent.display()))?;
        }
        let yaml = serde_yaml::to_string(self).context("Failed to serialize trip file")?;
        fs::write(path, yaml)
            .with_context(|| format!("Failed to write trip file {}", path.display()))?;
        Ok(())
    }

    /// Reconcile the trip's selected items against the current input config.
    ///
    /// Items are matched by name (case-insensitive) and category. Items that
    /// no longer exist in the input config are dropped; items are matched to
    /// the current category from the config. The packed state is preserved
    /// for items that still exist.
    /// Reconcile the trip's selected items against the current item catalog.
    ///
    /// Items are matched by name (case-insensitive) against any of a
    /// catalog item's categories. Items that no longer exist in the
    /// catalog are dropped; surviving items are matched to the catalog
    /// item's primary category. The packed state is preserved for items
    /// that still exist.
    pub fn reconcile(&mut self, config_items: &[(String, Vec<String>)]) {
        let find_config_categories = |name: &str, category: &str| -> Option<Vec<String>> {
            // Prefer an exact match on both name and category, fall back
            // to a name-only match
            config_items
                .iter()
                .find(|(n, cs)| {
                    n.eq_ignore_ascii_case(name)
                        && cs.iter().any(|c| c.eq_ignore_ascii_case(category))
                })
                .or_else(|| {
                    config_items
                        .iter()
                        .find(|(n, _)| n.eq_ignore_ascii_case(name))
                })
                .map(|(_, cs)| cs.clone())
        };

        self.items
            .retain(|item| find_config_categories(&item.name, &item.category).is_some());

        for item in &mut self.items {
            if let Some(categories) = find_config_categories(&item.name, &item.category) {
                // The trip file stores the primary category for display
                item.category = categories
                    .first()
                    .cloned()
                    .unwrap_or_else(|| item.category.clone());
            }
        }
    }
}
