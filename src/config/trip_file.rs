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
#[serde(deny_unknown_fields)]
pub(crate) struct TripFile {
    /// Trip parameters at the time of the last save
    pub(crate) weekdays: u32,
    pub(crate) travel_style: String,
    pub(crate) weather: String,
    /// The compiled packing list: selected items with packed state
    /// and quantities (spec 012). Strict per spec 013: required.
    pub(crate) items: Vec<TripItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// One selected item in the persisted trip file, with its packed state
/// and quantity. Strict per spec 013: every field required.
pub(crate) struct TripItem {
    pub(crate) name: String,
    /// The item's labels at save time (spec 025): the snapshot of
    /// the item's grouping.
    pub(crate) labels: Vec<String>,
    /// The singleton flag at save time (spec 025, US5); absent =
    /// false.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub(crate) singleton: bool,
    pub(crate) packed: bool,
    /// The item's quantity at save time (spec 012): the trip file is a
    /// complete snapshot.
    pub(crate) quantity: u32,
}

impl TripFile {
    /// Load a trip file. Returns None if the file does not exist yet
    /// (a new trip).
    pub(crate) fn load(path: &Path) -> Result<Option<Self>> {
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
    pub(crate) fn save(&self, path: &Path) -> Result<()> {
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
    /// Reconcile the trip snapshot against the current config
    /// (spec 025): match by name (case-insensitive), update the
    /// labels and the singleton flag from the config item; packed
    /// and quantity survive. Items with no config match drop.
    pub(crate) fn reconcile(&mut self, config_items: &[(String, Vec<String>, bool)]) {
        let find_config = |name: &str| -> Option<&(String, Vec<String>, bool)> {
            config_items
                .iter()
                .find(|(n, _, _)| n.eq_ignore_ascii_case(name))
        };

        self.items.retain(|item| find_config(&item.name).is_some());

        for item in &mut self.items {
            if let Some((_, labels, singleton)) = find_config(&item.name) {
                item.labels = labels.clone();
                item.singleton = *singleton;
            }
        }
    }
}
