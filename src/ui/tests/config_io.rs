use std::path::Path;

use crate::config::modifiers::ModifiersConfig;
use crate::ui::config_io::{AppConfig, SavedItem, resolve_trip_path};

#[test]
fn multi_category_reconcile_matches_any_membership() {
    use crate::config::trip_file::{TripFile, TripItem};
    let mut trip = TripFile {
        weekdays: 7,
        travel_style: "standard".to_string(),
        weather: "mixed".to_string(),
        items: vec![TripItem {
            name: "Knife".to_string(),
            category: "Kitchen".to_string(),
            packed: true,
            quantity: 1,
        }],
    };
    // The catalog item "Knife" moved to a new category set; the trip
    // item's old category matches none, but the name does
    let config_items = vec![(
        "Knife".to_string(),
        vec!["Cooking".to_string(), "Gear".to_string()],
    )];
    trip.reconcile(&config_items);
    assert_eq!(trip.items.len(), 1);
    // Remapped to the catalog item's primary category
    assert_eq!(trip.items[0].category, "Cooking");
    assert!(trip.items[0].packed, "packed state preserved");
}

#[test]
fn full_config_round_trips() {
    let config = AppConfig {
        categories: vec!["Clothing".to_string()],
        items: vec![
            SavedItem {
                name: "T-Shirt".to_string(),
                categories: vec!["Clothing".to_string()],
                quantity: None,
                auto_quantity: true,
                rate: None,
                reserve: None,
                buffer_multiplier: None,
            },
            SavedItem {
                name: "Toothbrush".to_string(),
                categories: vec!["Toiletries".to_string()],
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
        categories: vec![],
        items: vec![SavedItem {
            name: "T-Shirt".to_string(),
            categories: vec!["Clothing".to_string()],
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
                category: "Clothing".to_string(),
                packed: true,
                quantity: 1,
            },
            TripItem {
                name: "Old Item".to_string(),
                category: "Gone".to_string(),
                packed: false,
                quantity: 1,
            },
        ],
    };

    // Reconcile against a config where "Old Item" no longer exists
    let config_items = vec![
        ("T-Shirt".to_string(), vec!["Clothing".to_string()]),
        ("Socks".to_string(), vec!["Clothing".to_string()]),
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
        categories: vec!["Clothing".to_string()],
        items: vec![SavedItem {
            name: "Underwear".to_string(),
            categories: vec!["Clothing".to_string()],
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
        categories: vec![],
        items: vec![SavedItem {
            name: "Socks".to_string(),
            categories: vec!["Clothing".to_string()],
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
            category: "Clothing".to_string(),
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
                category: "Old".to_string(),
                packed: true,
                quantity: 3,
            },
            TripItem {
                name: "Gone".to_string(),
                category: "Nowhere".to_string(),
                packed: false,
                quantity: 9,
            },
        ],
    };
    let config_items = vec![(
        "Knife".to_string(),
        vec!["Cooking".to_string(), "Gear".to_string()],
    )];
    trip.reconcile(&config_items);
    assert_eq!(trip.items.len(), 1); // the gone item dropped
    assert_eq!(trip.items[0].quantity, 3); // the survivor kept it
    assert_eq!(trip.items[0].category, "Cooking"); // remapped
}

// ---- Spec 013: strict formats ----

#[test]
fn current_format_config_loads() {
    let yaml = "categories:\n- Clothing\nitems:\n- name: T-Shirt\n  categories:\n  - Clothing\n  auto_quantity: true\nweekdays: 7\ntravel_style: standard\nweather: mixed\nmodifiers:\n  travel_styles:\n  - name: standard\n    multiplier: 1.0\n  weather:\n  - name: mixed\n    multiplier: 1.0\n";
    let config: AppConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.items.len(), 1);
    assert_eq!(config.weekdays, 7);
}

#[test]
fn config_missing_a_required_field_fails() {
    // No weekdays key: the strict format rejects the file
    let yaml = "categories:\n- Clothing\nitems: []\ntravel_style: standard\nweather: mixed\nmodifiers:\n  travel_styles: []\n  weather: []\n";
    let result: Result<AppConfig, _> = serde_yaml::from_str(yaml);
    assert!(result.is_err(), "a missing required field must fail");
}

#[test]
fn config_with_unknown_field_fails() {
    // EC-002: unknown fields mean the file is not in the current format
    let yaml = "categories: []\nitems: []\nweekdays: 7\ntravel_style: standard\nweather: mixed\nmodifiers:\n  travel_styles: []\n  weather: []\nsome_legacy_key: 1\n";
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
    let yaml = "weekdays: 7\ntravel_style: standard\nweather: mixed\nitems:\n- name: A\n  category: C\n  packed: false\n";
    let result: Result<crate::config::trip_file::TripFile, _> = serde_yaml::from_str(yaml);
    assert!(result.is_err(), "a trip item without quantity must fail");
}
