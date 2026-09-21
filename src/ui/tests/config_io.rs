use std::path::Path;

use crate::config::modifiers::ModifiersConfig;
use crate::ui::config_io::{AppConfig, SavedItem, resolve_trip_path};

#[test]
fn reconcile_updates_labels_and_singleton() {
    use crate::config::trip_file::{TripFile, TripItem};
    let mut trip = TripFile {
        weekdays: 7,
        travel_style: "standard".to_string(),
        weather: "mixed".to_string(),
        items: vec![TripItem {
            name: "Knife".to_string(),
            labels: vec!["Kitchen".to_string()],
            singleton: false,
            packed: true,
            quantity: 1,
        }],
    };
    // The config item "Knife" now carries new labels and is a
    // singleton; the name matches, so the trip item follows
    let config_items = vec![(
        "Knife".to_string(),
        vec!["Cooking".to_string(), "Gear".to_string()],
        true,
    )];
    trip.reconcile(&config_items);
    assert_eq!(trip.items.len(), 1);
    assert_eq!(trip.items[0].labels, vec!["Cooking", "Gear"]);
    assert!(trip.items[0].singleton);
    assert!(trip.items[0].packed, "packed state preserved");
}

#[test]
fn full_config_round_trips() {
    let config = AppConfig {
        items: vec![
            SavedItem {
                name: "T-Shirt".to_string(),
                labels: vec!["Clothing".to_string()],
                singleton: false,
                quantity: None,
                auto_quantity: true,
                rate: None,
                reserve: None,
                buffer_multiplier: None,
            },
            SavedItem {
                name: "Toothbrush".to_string(),
                labels: vec!["Toiletries".to_string()],
                singleton: false,
                quantity: Some(1),
                auto_quantity: false,
                rate: None,
                reserve: None,
                buffer_multiplier: None,
            },
        ],
        weekdays: 10,
        travel_style: "luxury".to_string(),
        weather: "snowy".to_string(),
        modifiers: ModifiersConfig::default(),
        last_trip_file: Some("my-trip.yaml".to_string()),
    };
    let yaml = serde_yaml::to_string(&config).unwrap();
    let parsed: AppConfig = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(parsed.weekdays, 10);
    assert_eq!(parsed.travel_style, "luxury");
    assert_eq!(parsed.weather, "snowy");
    assert_eq!(parsed.last_trip_file.as_deref(), Some("my-trip.yaml"));
    // Auto items do not persist a quantity
    assert_eq!(parsed.items[0].quantity, None);
    assert!(parsed.items[0].auto_quantity);
    // Fixed items keep their quantity
    assert_eq!(parsed.items[1].quantity, Some(1));
    assert!(!parsed.items[1].auto_quantity);
    assert_eq!(
        parsed.modifiers.travel_style_multiplier("luxury"),
        Some(1.5)
    );
}

#[test]
fn auto_quantities_are_not_persisted() {
    // The serialized config must not persist a quantity value for auto
    // items (only the auto_quantity flag may appear)
    let config = AppConfig {
        items: vec![SavedItem {
            name: "T-Shirt".to_string(),
            labels: vec!["Clothing".to_string()],
            singleton: false,
            quantity: None,
            auto_quantity: true,
            rate: None,
            reserve: None,
            buffer_multiplier: None,
        }],
        weekdays: 7,
        travel_style: "standard".to_string(),
        weather: "mixed".to_string(),
        modifiers: ModifiersConfig::default(),
        last_trip_file: None,
    };
    let yaml = serde_yaml::to_string(&config).unwrap();
    // No "quantity: <n>" line for the auto item
    assert!(
        !yaml
            .lines()
            .any(|l| l.trim_start().starts_with("quantity:"))
    );
    assert!(yaml.contains("auto_quantity: true"));
}

#[test]
fn trip_file_round_trips_and_reconciles() {
    use crate::config::trip_file::{TripFile, TripItem};
    let mut trip = TripFile {
        weekdays: 7,
        travel_style: "standard".to_string(),
        weather: "cold".to_string(),
        items: vec![
            TripItem {
                name: "T-Shirt".to_string(),
                labels: vec!["Clothing".to_string()],
                singleton: false,
                packed: true,
                quantity: 1,
            },
            TripItem {
                name: "Old Item".to_string(),
                labels: vec!["Gone".to_string()],
                singleton: false,
                packed: false,
                quantity: 1,
            },
        ],
    };

    // Reconcile against a config where "Old Item" no longer exists
    let config_items = vec![
        ("T-Shirt".to_string(), vec!["Clothing".to_string()], false),
        ("Socks".to_string(), vec!["Clothing".to_string()], false),
    ];
    trip.reconcile(&config_items);

    // Dropped item that is gone from the config, kept the rest
    assert_eq!(trip.items.len(), 1);
    assert_eq!(trip.items[0].name, "T-Shirt");
    assert!(trip.items[0].packed);

    // Round trip through YAML
    let yaml = serde_yaml::to_string(&trip).unwrap();
    let parsed: TripFile = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(parsed.weekdays, 7);
    assert_eq!(parsed.items.len(), 1);
    assert!(parsed.items[0].packed);
}

#[test]
fn trip_paths_resolve_relative_absolute_and_tilde() {
    // Relative names resolve against the current working directory
    let cwd = std::env::current_dir().unwrap();
    assert_eq!(resolve_trip_path("my-trip.yaml"), cwd.join("my-trip.yaml"));
    assert_eq!(
        resolve_trip_path("sub/my-trip.yaml"),
        cwd.join("sub/my-trip.yaml")
    );

    // Absolute paths are kept as-is
    assert_eq!(
        resolve_trip_path("/tmp/my-trip.yaml"),
        Path::new("/tmp/my-trip.yaml")
    );

    // ~ expands to the home directory
    let home = dirs::home_dir().unwrap();
    assert_eq!(
        resolve_trip_path("~/my-trip.yaml"),
        home.join("my-trip.yaml")
    );
    assert_eq!(resolve_trip_path("~"), home);
}

// ---- Spec 010: per-item modifier persistence ----

#[test]
fn rate_and_reserve_round_trip() {
    let config = AppConfig {
        items: vec![SavedItem {
            name: "Underwear".to_string(),
            labels: vec!["Clothing".to_string()],
            singleton: false,
            quantity: None,
            auto_quantity: true,
            rate: Some(1.0),
            reserve: Some(2),
            buffer_multiplier: None,
        }],
        weekdays: 5,
        travel_style: "standard".to_string(),
        weather: "mixed".to_string(),
        modifiers: ModifiersConfig::default(),
        last_trip_file: None,
    };
    let yaml = serde_yaml::to_string(&config).unwrap();
    let parsed: AppConfig = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(parsed.items[0].rate, Some(1.0));
    assert_eq!(parsed.items[0].reserve, Some(2));
    assert_eq!(parsed.items[0].buffer_multiplier, None);
    // Exactly one buffer kind on the wire (SC-007)
    assert!(yaml.contains("reserve: 2"));
    assert!(!yaml.contains("buffer_multiplier"));
}

#[test]
fn rate_and_multiplier_round_trip() {
    let config = AppConfig {
        items: vec![SavedItem {
            name: "Socks".to_string(),
            labels: vec!["Clothing".to_string()],
            singleton: false,
            quantity: None,
            auto_quantity: true,
            rate: Some(1.0),
            reserve: None,
            buffer_multiplier: Some(1.25),
        }],
        weekdays: 5,
        travel_style: "standard".to_string(),
        weather: "mixed".to_string(),
        modifiers: ModifiersConfig::default(),
        last_trip_file: None,
    };
    let yaml = serde_yaml::to_string(&config).unwrap();
    let parsed: AppConfig = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(parsed.items[0].buffer_multiplier, Some(1.25));
    assert_eq!(parsed.items[0].reserve, None);
    assert!(yaml.contains("buffer_multiplier: 1.25"));
    assert!(!yaml.contains("reserve:"));
}

// ---- Spec 012: quantities in the trip file ----

#[test]
fn trip_file_round_trips_with_quantities() {
    use crate::config::trip_file::{TripFile, TripItem};
    let trip = TripFile {
        weekdays: 7,
        travel_style: "standard".to_string(),
        weather: "mixed".to_string(),
        items: vec![TripItem {
            name: "T-Shirt".to_string(),
            labels: vec!["Clothing".to_string()],
            singleton: false,
            packed: true,
            quantity: 5,
        }],
    };
    let yaml = serde_yaml::to_string(&trip).unwrap();
    assert!(yaml.contains("quantity: 5"), "quantity on the wire: {yaml}");
    let parsed: TripFile = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(parsed.items[0].quantity, 5);
}

#[test]
fn reconcile_preserves_quantities() {
    // EC-002: quantities survive reconciliation
    use crate::config::trip_file::{TripFile, TripItem};
    let mut trip = TripFile {
        weekdays: 7,
        travel_style: "standard".to_string(),
        weather: "mixed".to_string(),
        items: vec![
            TripItem {
                name: "Knife".to_string(),
                labels: vec!["Old".to_string()],
                singleton: false,
                packed: true,
                quantity: 3,
            },
            TripItem {
                name: "Gone".to_string(),
                labels: vec!["Nowhere".to_string()],
                singleton: false,
                packed: false,
                quantity: 9,
            },
        ],
    };
    let config_items = vec![(
        "Knife".to_string(),
        vec!["Cooking".to_string(), "Gear".to_string()],
        false,
    )];
    trip.reconcile(&config_items);
    assert_eq!(trip.items.len(), 1); // the gone item dropped
    assert_eq!(trip.items[0].quantity, 3); // the survivor kept it
    assert_eq!(trip.items[0].labels, vec!["Cooking", "Gear"]); // remapped
}

// ---- Spec 013: strict formats ----

#[test]
fn current_format_config_loads() {
    let yaml = "items:\n- name: T-Shirt\n  labels:\n  - Clothing\n  auto_quantity: true\nweekdays: 7\ntravel_style: standard\nweather: mixed\nmodifiers:\n  travel_styles:\n  - name: standard\n    multiplier: 1.0\n  weather:\n  - name: mixed\n    multiplier: 1.0\n";
    let config: AppConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.items.len(), 1);
    assert_eq!(config.weekdays, 7);
}

#[test]
fn config_missing_a_required_field_fails() {
    // No weekdays key: the strict format rejects the file
    let yaml = "items: []\ntravel_style: standard\nweather: mixed\nmodifiers:\n  travel_styles: []\n  weather: []\n";
    let result: Result<AppConfig, _> = serde_yaml::from_str(yaml);
    assert!(result.is_err(), "a missing required field must fail");
}

#[test]
fn config_with_unknown_field_fails() {
    // EC-002: unknown fields mean the file is not in the current format
    let yaml = "items: []\nweekdays: 7\ntravel_style: standard\nweather: mixed\nmodifiers:\n  travel_styles: []\n  weather: []\nsome_legacy_key: 1\n";
    let result: Result<AppConfig, _> = serde_yaml::from_str(yaml);
    assert!(result.is_err(), "an unknown field must fail");
}

#[test]
fn trip_file_missing_a_field_fails() {
    let yaml = "weekdays: 7\ntravel_style: standard\nweather: mixed\n";
    let result: Result<crate::config::trip_file::TripFile, _> = serde_yaml::from_str(yaml);
    assert!(result.is_err(), "a trip file without items must fail");
}

#[test]
fn trip_item_requires_quantity() {
    // Post-013 the quantity is required on the wire
    let yaml = "weekdays: 7\ntravel_style: standard\nweather: mixed\nitems:\n- name: A\n  labels:\n  - C\n  packed: false\n";
    let result: Result<crate::config::trip_file::TripFile, _> = serde_yaml::from_str(yaml);
    assert!(result.is_err(), "a trip item without quantity must fail");
}

// ---- Spec 025: the label format ----

#[test]
fn labeled_config_loads_with_optional_singleton() {
    // FR-001/FR-002: labels replace categories; the singleton flag is
    // optional in format
    let yaml = "items:\n- name: T-Shirt\n  labels:\n  - Clothing\n  auto_quantity: true\n- name: Führerschein\n  labels:\n  - Ausland\n  - Karten\n  singleton: true\n  auto_quantity: false\n  quantity: 1\nweekdays: 7\ntravel_style: standard\nweather: mixed\nmodifiers:\n  travel_styles:\n  - name: standard\n    multiplier: 1.0\n  weather:\n  - name: mixed\n    multiplier: 1.0\n";
    let config: AppConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.items.len(), 2);
    assert_eq!(config.items[0].labels, vec!["Clothing"]);
    assert!(!config.items[0].singleton);
    assert_eq!(config.items[1].labels, vec!["Ausland", "Karten"]);
    assert!(config.items[1].singleton);
}

#[test]
fn item_without_singleton_serializes_without_the_key() {
    // The singleton flag is skip_serializing_if-false: a plain item's
    // yaml carries no `singleton` key
    let config = AppConfig {
        items: vec![SavedItem {
            name: "T-Shirt".to_string(),
            labels: vec!["Clothing".to_string()],
            singleton: false,
            quantity: None,
            auto_quantity: true,
            rate: None,
            reserve: None,
            buffer_multiplier: None,
        }],
        weekdays: 7,
        travel_style: "standard".to_string(),
        weather: "mixed".to_string(),
        modifiers: ModifiersConfig::default(),
        last_trip_file: None,
    };
    let yaml = serde_yaml::to_string(&config).unwrap();
    assert!(!yaml.contains("singleton"));
}

#[test]
fn config_round_trip_preserves_labels_and_singleton() {
    // FR-001: a load→save round trip preserves every label and the
    // singleton flag verbatim
    let config = AppConfig {
        items: vec![
            SavedItem {
                name: "Führerschein".to_string(),
                labels: vec!["Ausland".to_string(), "Karten".to_string()],
                singleton: true,
                quantity: Some(1),
                auto_quantity: false,
                rate: None,
                reserve: None,
                buffer_multiplier: None,
            },
            SavedItem {
                name: "T-Shirt".to_string(),
                labels: vec!["Clothing".to_string()],
                singleton: false,
                quantity: None,
                auto_quantity: true,
                rate: Some(1.0),
                reserve: Some(2),
                buffer_multiplier: None,
            },
        ],
        weekdays: 7,
        travel_style: "standard".to_string(),
        weather: "mixed".to_string(),
        modifiers: ModifiersConfig::default(),
        last_trip_file: None,
    };
    let yaml = serde_yaml::to_string(&config).unwrap();
    let parsed: AppConfig = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(parsed.items[0].labels, vec!["Ausland", "Karten"]);
    assert!(parsed.items[0].singleton);
    assert_eq!(parsed.items[1].labels, vec!["Clothing"]);
    assert!(!parsed.items[1].singleton);
}

#[test]
fn old_format_with_categories_is_rejected() {
    // FR-004: an old-format file (with `categories:`) fails with an
    // unknown-field error naming `categories`
    let yaml = "items:\n- name: T-Shirt\n  categories:\n  - Clothing\n  auto_quantity: true\nweekdays: 7\ntravel_style: standard\nweather: mixed\nmodifiers:\n  travel_styles: []\n  weather: []\n";
    let result: Result<AppConfig, _> = serde_yaml::from_str(yaml);
    let Err(err) = result else {
        panic!("an old-format file must be rejected");
    };
    let message = err.to_string();
    assert!(
        message.contains("categories"),
        "the error must name the unknown field: {message}"
    );
}

#[test]
fn old_format_with_top_level_categories_is_rejected() {
    // FR-002/FR-004: the top-level `categories` key is gone from the
    // schema; a file carrying it is rejected
    let yaml = "items: []\nweekdays: 7\ntravel_style: standard\nweather: mixed\nmodifiers:\n  travel_styles: []\n  weather: []\ncategories:\n- Clothing\n";
    let result: Result<AppConfig, _> = serde_yaml::from_str(yaml);
    let Err(err) = result else {
        panic!("a top-level categories key must be rejected");
    };
    assert!(
        err.to_string().contains("categories"),
        "the error must name the unknown field: {}",
        err
    );
}

// ---- Spec 025, US2: the dedup on load (FR-005) ----

#[test]
fn duplicate_entries_load_as_one() {
    // Two identical (name, labels) entries: the first wins, the second
    // drops (FR-005)
    let yaml = "items:\n- name: Ladekabel\n  labels:\n  - Elektronik\n  auto_quantity: false\n  quantity: 1\n- name: Ladekabel\n  labels:\n  - Elektronik\n  auto_quantity: false\n  quantity: 1\nweekdays: 7\ntravel_style: standard\nweather: mixed\nmodifiers:\n  travel_styles: []\n  weather: []\n";
    let config: AppConfig = serde_yaml::from_str(yaml).unwrap();
    let mut packing_list = crate::models::packing_list::PackingList::new();
    let mut seen: std::collections::BTreeSet<(String, Vec<String>)> =
        std::collections::BTreeSet::new();
    for saved_item in config.items {
        let key = (saved_item.name.clone(), saved_item.labels.clone());
        if !seen.insert(key) {
            continue;
        }
        packing_list.add_item(crate::models::item::Item::new(
            saved_item.name,
            saved_item.labels,
            saved_item.quantity.unwrap_or(1),
        ));
    }
    assert_eq!(packing_list.items.len(), 1);
    assert_eq!(packing_list.items[0].name, "Ladekabel");
}

#[test]
fn same_name_with_different_labels_stays_distinct() {
    // Same name, different labels: two distinct items (FR-005)
    let yaml = "items:\n- name: Ladekabel\n  labels:\n  - Elektronik\n  auto_quantity: false\n  quantity: 1\n- name: Ladekabel\n  labels:\n  - Auto\n  - Elektronik\n  auto_quantity: false\n  quantity: 1\nweekdays: 7\ntravel_style: standard\nweather: mixed\nmodifiers:\n  travel_styles: []\n  weather: []\n";
    let config: AppConfig = serde_yaml::from_str(yaml).unwrap();
    let mut packing_list = crate::models::packing_list::PackingList::new();
    let mut seen: std::collections::BTreeSet<(String, Vec<String>)> =
        std::collections::BTreeSet::new();
    for saved_item in config.items {
        let key = (saved_item.name.clone(), saved_item.labels.clone());
        if !seen.insert(key) {
            continue;
        }
        packing_list.add_item(crate::models::item::Item::new(
            saved_item.name,
            saved_item.labels,
            saved_item.quantity.unwrap_or(1),
        ));
    }
    assert_eq!(packing_list.items.len(), 2);
    assert_eq!(packing_list.items[0].labels, vec!["Elektronik"]);
    assert_eq!(packing_list.items[1].labels, vec!["Auto", "Elektronik"]);
}

#[test]
fn real_data_shape_fivefold_duplicate_merges_to_one() {
    // The real config's shape: the same item repeated across blocks —
    // one item after the dedup, first occurrence wins
    let yaml = "items:\n- name: Ladekabel\n  labels:\n  - Elektronik\n  auto_quantity: false\n  quantity: 1\n- name: Ladekabel\n  labels:\n  - Elektronik\n  auto_quantity: false\n  quantity: 1\n- name: Ladekabel\n  labels:\n  - Elektronik\n  auto_quantity: false\n  quantity: 1\n- name: Ladekabel\n  labels:\n  - Elektronik\n  auto_quantity: false\n  quantity: 1\n- name: Ladekabel\n  labels:\n  - Elektronik\n  auto_quantity: false\n  quantity: 1\nweekdays: 7\ntravel_style: standard\nweather: mixed\nmodifiers:\n  travel_styles: []\n  weather: []\n";
    let config: AppConfig = serde_yaml::from_str(yaml).unwrap();
    let mut packing_list = crate::models::packing_list::PackingList::new();
    let mut seen: std::collections::BTreeSet<(String, Vec<String>)> =
        std::collections::BTreeSet::new();
    for saved_item in config.items {
        let key = (saved_item.name.clone(), saved_item.labels.clone());
        if !seen.insert(key) {
            continue;
        }
        packing_list.add_item(crate::models::item::Item::new(
            saved_item.name,
            saved_item.labels,
            saved_item.quantity.unwrap_or(1),
        ));
    }
    assert_eq!(packing_list.items.len(), 1);
}

#[test]
fn dedup_first_occurrence_wins_on_quantity() {
    // The first occurrence's quantity survives the merge
    let yaml = "items:\n- name: Ladekabel\n  labels:\n  - Elektronik\n  auto_quantity: false\n  quantity: 2\n- name: Ladekabel\n  labels:\n  - Elektronik\n  auto_quantity: false\n  quantity: 5\nweekdays: 7\ntravel_style: standard\nweather: mixed\nmodifiers:\n  travel_styles: []\n  weather: []\n";
    let config: AppConfig = serde_yaml::from_str(yaml).unwrap();
    let mut packing_list = crate::models::packing_list::PackingList::new();
    let mut seen: std::collections::BTreeSet<(String, Vec<String>)> =
        std::collections::BTreeSet::new();
    for saved_item in config.items {
        let key = (saved_item.name.clone(), saved_item.labels.clone());
        if !seen.insert(key) {
            continue;
        }
        packing_list.add_item(crate::models::item::Item::new(
            saved_item.name,
            saved_item.labels,
            saved_item.quantity.unwrap_or(1),
        ));
    }
    assert_eq!(packing_list.items.len(), 1);
    assert_eq!(packing_list.items[0].quantity, 2);
}

// ---- Spec 025, US2: individual treatment (FR-006, SC-003) ----

#[test]
fn multi_label_item_appears_under_each_label_but_is_one_item() {
    // FR-006/SC-003: a multi-label item appears under every label in
    // both panels but is one item — editing it changes the one item
    let mut item = crate::models::item::Item::new(
        "Führerschein".to_string(),
        vec!["Ausland".to_string(), "Karten".to_string()],
        1,
    );
    item.singleton = true;
    let mut state = super::test_state(vec![item]);
    let left = state.left_rows();
    // One header per distinct label, sorted: Ausland, Karten
    let labels: Vec<&String> = left
        .iter()
        .filter_map(|r| match r {
            crate::ui::state::PanelRow::Label(l) => Some(l),
            _ => None,
        })
        .collect();
    assert_eq!(labels.len(), 2);
    assert_eq!(labels[0], "Ausland");
    assert_eq!(labels[1], "Karten");
    // The item appears under both labels
    let item_rows: Vec<usize> = left
        .iter()
        .enumerate()
        .filter_map(|(pos, r)| match r {
            crate::ui::state::PanelRow::Item(i) => Some((pos, *i)),
            _ => None,
        })
        .map(|(_, i)| i)
        .collect();
    assert_eq!(item_rows, vec![0, 0]);
    // Editing the quantity changes the one item
    state.apply_item_edit(
        0,
        "Führerschein".to_string(),
        vec!["Ausland".to_string(), "Karten".to_string()],
        true,
        "3".to_string(),
        Some(false),
        String::new(),
        crate::ui::state::QuantityBufferKind::None,
        String::new(),
    );
    assert_eq!(state.packing_list.items.len(), 1);
    assert_eq!(state.packing_list.items[0].quantity, 3);
}

#[test]
fn compiled_list_counts_a_multi_label_item_once() {
    // SC-003: the compiled list counts a multi-label item once
    let mut item = crate::models::item::Item::new(
        "Führerschein".to_string(),
        vec!["Ausland".to_string(), "Karten".to_string()],
        1,
    );
    item.selected = true;
    let state = super::test_state(vec![item]);
    let selected: Vec<&crate::models::item::Item> = state
        .packing_list
        .items
        .iter()
        .filter(|i| i.selected)
        .collect();
    assert_eq!(selected.len(), 1);
    // The right panel shows it under both labels but the underlying
    // list has one entry
    let right = state.right_rows();
    let item_count = right
        .iter()
        .filter(|r| matches!(r, crate::ui::state::PanelRow::Item(_)))
        .count();
    assert_eq!(item_count, 2);
    assert_eq!(state.packing_list.items.len(), 1);
}

#[test]
fn saved_config_contains_no_separator_and_no_categories_key() {
    // SC-001: no ": " inside any label value, no top-level
    // `categories` key on the wire
    let config = AppConfig {
        items: vec![SavedItem {
            name: "Führerschein".to_string(),
            labels: vec!["Ausland".to_string(), "Karten".to_string()],
            singleton: true,
            quantity: Some(1),
            auto_quantity: false,
            rate: None,
            reserve: None,
            buffer_multiplier: None,
        }],
        weekdays: 7,
        travel_style: "standard".to_string(),
        weather: "mixed".to_string(),
        modifiers: ModifiersConfig::default(),
        last_trip_file: None,
    };
    let yaml = serde_yaml::to_string(&config).unwrap();
    // Walk the wire: the `- X` entries directly under a `labels:` key
    // are the label values — none may contain ": "
    let mut in_labels_block = false;
    for line in yaml.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("labels:") {
            in_labels_block = true;
            continue;
        }
        if in_labels_block {
            if let Some(label) = trimmed.strip_prefix("- ") {
                assert!(
                    !label.contains(": "),
                    "a label must not contain ': ' — found {label:?}"
                );
            } else {
                in_labels_block = false;
            }
        }
    }
    assert!(
        !yaml.lines().any(|l| l.starts_with("categories:")),
        "no top-level categories key on the wire"
    );
}
