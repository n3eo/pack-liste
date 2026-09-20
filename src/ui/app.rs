use anyhow::{Context, Result};
use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
        Wrap,
    },
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, stdout};
use std::path::{Path, PathBuf};

use crate::config::modifiers::ModifiersConfig;
use crate::config::trip_file::{TripFile, TripItem};
use crate::models::item::Item;
use crate::models::packing_list::PackingList;
use crate::models::weekdays::Weekdays;
use crate::quantity::calculation::QuantityCalculation;
use crate::quantity::model::{ItemQuantitySpec, QuantityCalculationInput};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Screen {
    Welcome,
    Main,
    Help,
    Config,
    /// Raw editor for the config file
    ConfigEditor,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FocusSide {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Focus {
    TripFile,
    Weekdays,
    TravelStyle,
    Weather,
}

impl Focus {
    pub fn next(&self) -> Self {
        match self {
            Focus::TripFile => Focus::Weekdays,
            Focus::Weekdays => Focus::TravelStyle,
            Focus::TravelStyle => Focus::Weather,
            Focus::Weather => Focus::TripFile,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Focus::TripFile => Focus::Weather,
            Focus::Weekdays => Focus::TripFile,
            Focus::TravelStyle => Focus::Weekdays,
            Focus::Weather => Focus::TravelStyle,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputField {
    Name,
    Category,
    Quantity,
    Mode,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,
    AddingItem {
        field: InputField,
        name: String,
        category: String,
        quantity: String,
        category_list: Vec<String>,
        category_index: usize,
    },
    EditingItem {
        index: usize,
        field: InputField,
        name: String,
        category: String,
        quantity: String,
        /// Quantity mode carried from the item; toggled in the dialog's
        /// Mode field and written back on save
        auto_quantity: bool,
        category_list: Vec<String>,
        category_index: usize,
        /// The item's category memberships; Space toggles the highlighted
        /// entry in the category dropdown
        selected_categories: Vec<String>,
    },
    ConfirmDelete {
        index: usize,
    },
}

pub enum AppState {
    Running,
    Exit,
}

/// Editing state shared by the add-item and edit-item popups, bundled so the
/// popup draw functions take one argument instead of eight.
struct PopupFields<'a> {
    field: InputField,
    name: &'a str,
    category: &'a str,
    quantity: &'a str,
    /// Quantity mode shown in the edit dialog's Mode field
    auto_quantity: Option<bool>,
    category_list: &'a [String],
    category_index: usize,
}

// Config structure for saving to file
#[derive(Debug, Serialize, Deserialize)]
struct AppConfig {
    categories: Vec<String>,
    items: Vec<SavedItem>,
    #[serde(default)]
    weekdays: u32,
    #[serde(default = "default_travel_style_name")]
    travel_style: String,
    #[serde(default = "default_weather_name")]
    weather: String,
    #[serde(default)]
    modifiers: ModifiersConfig,
    /// Name of the trip file to continue by default
    #[serde(default, skip_serializing_if = "Option::is_none")]
    last_trip_file: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct SavedItem {
    name: String,
    /// The item's categories; at least one. A legacy single `category:`
    /// string is lifted into a one-element list on load.
    categories: Vec<String>,
    /// Only present for fixed-quantity items (auto_quantity: false);
    /// auto items are recalculated on every load
    #[serde(skip_serializing_if = "Option::is_none")]
    quantity: Option<u32>,
    /// When false, the quantity is fixed and never recalculated
    auto_quantity: bool,
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

fn default_travel_style_name() -> String {
    "standard".to_string()
}

fn default_weather_name() -> String {
    "mixed".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Test helpers: construct AppDrawState without a terminal ----

    fn test_item(name: &str, category: &str) -> Item {
        Item::new(
            format!("id_{name}"),
            name.to_string(),
            vec![category.to_string()],
            1,
        )
    }

    fn test_state(items: Vec<Item>) -> AppDrawState {
        let mut packing_list = PackingList::new("Test Trip".to_string());
        for item in items {
            packing_list.add_item(item);
        }
        AppDrawState {
            screen: Screen::Main,
            packing_list,
            weekdays: Weekdays::new(7).unwrap(),
            travel_style: "standard".to_string(),
            weather: "mixed".to_string(),
            modifiers: ModifiersConfig::default(),
            focus: Focus::TripFile,
            collapsed_categories: Vec::new(),
            selected_index: 0,
            focus_side: FocusSide::Left,
            input_mode: InputMode::Normal,
            left_scroll: 0,
            right_scroll: 0,
            trip_file_path: PathBuf::new(),
            trip_file_input: String::new(),
            editing_trip_file: false,
            resume_prompt: false,
            remembered_trip_file: None,
            config_section: ConfigSection::TravelStyles,
            config_cursor: 0,
            config_scroll: 0,
            modifier_editor: None,
            previous_screen: Screen::Welcome,
            config_editor_text: Vec::new(),
            config_editor_cursor: (0, 0),
            config_editor_scroll: 0,
            config_editor_error: None,
        }
    }

    // ---- Input-handling tests (US5): key events in, state out ----

    #[test]
    fn cursor_moves_down_and_up_on_main() {
        let mut state = test_state(vec![
            test_item("A", "Cat"),
            test_item("B", "Cat"),
            test_item("C", "Cat"),
        ]);
        // Left panel rows: [Cat header, A, B, C]; cursor starts on header
        state.move_cursor_down();
        assert_eq!(state.selected_index, 1);
        state.move_cursor_down();
        assert_eq!(state.selected_index, 2);
        state.move_cursor_up();
        assert_eq!(state.selected_index, 1);
    }

    #[test]
    fn cursor_up_at_top_stays_put() {
        // Mutation guard: if the up-binding logic inverts, this fails
        let mut state = test_state(vec![test_item("A", "Cat"), test_item("B", "Cat")]);
        state.selected_index = 0;
        state.move_cursor_up();
        assert_eq!(state.selected_index, 0);
    }

    #[test]
    fn cursor_down_at_last_row_stays_put() {
        let mut state = test_state(vec![test_item("A", "Cat")]);
        // Left rows: [Cat header, A]; last valid index is 1
        state.selected_index = 1;
        state.move_cursor_down();
        assert_eq!(state.selected_index, 1);
    }

    #[test]
    fn welcome_focus_cycles_with_up_down() {
        let mut state = test_state(Vec::new());
        state.screen = Screen::Welcome;
        state.focus = Focus::TripFile;
        state.move_cursor_down();
        assert_eq!(state.focus, Focus::Weekdays);
        state.move_cursor_up();
        assert_eq!(state.focus, Focus::TripFile);
        // Wrapping: up from the first field goes to the last
        state.move_cursor_up();
        assert_eq!(state.focus, Focus::Weather);
    }

    #[test]
    fn item_selection_toggles() {
        let mut state = test_state(vec![test_item("A", "Cat")]);
        state.selected_index = 1; // cursor on item A
        assert!(state.packing_list.items[0].editable);
        state.toggle_item_selection();
        assert!(!state.packing_list.items[0].editable);
        state.toggle_item_selection();
        assert!(state.packing_list.items[0].editable);
    }

    #[test]
    fn category_selection_toggles_all_items_and_collapses() {
        let mut state = test_state(vec![
            test_item("A", "Cat"),
            test_item("B", "Cat"),
            test_item("C", "Other"),
        ]);
        state.selected_index = 0; // cursor on the Cat header
        state.toggle_category_selection();
        // Both Cat items deselected, Other untouched; category collapsed
        assert!(!state.packing_list.items[0].editable);
        assert!(!state.packing_list.items[1].editable);
        assert!(state.packing_list.items[2].editable);
        assert!(state.collapsed_categories.contains(&"Cat".to_string()));
        // Cursor stays on the header row after collapsing
        assert_eq!(state.selected_index, 0);
        // Toggling back reselects and expands
        state.toggle_category_selection();
        assert!(state.packing_list.items[0].editable);
        assert!(state.packing_list.items[1].editable);
        assert!(!state.collapsed_categories.contains(&"Cat".to_string()));
    }

    #[test]
    fn packed_toggles_only_on_right_side() {
        let mut state = test_state(vec![test_item("A", "Cat")]);
        state.focus_side = FocusSide::Left;
        state.selected_index = 1;
        state.toggle_item_packed();
        assert!(!state.packing_list.items[0].packed);
        // On the right side the rows are the selected items only: one row
        state.focus_side = FocusSide::Right;
        state.selected_index = 0;
        state.toggle_item_packed();
        assert!(state.packing_list.items[0].packed);
        state.toggle_item_packed();
        assert!(!state.packing_list.items[0].packed);
    }

    #[test]
    fn collapse_moves_cursor_to_header() {
        let mut state = test_state(vec![test_item("A", "Cat"), test_item("B", "Cat")]);
        state.selected_index = 2; // cursor on item B
        state.toggle_category_collapse();
        assert!(state.collapsed_categories.contains(&"Cat".to_string()));
        // Cursor moved to the category header (row 0)
        assert_eq!(state.selected_index, 0);
        // Expanding keeps the cursor on the header
        state.toggle_category_collapse();
        assert!(!state.collapsed_categories.contains(&"Cat".to_string()));
        assert_eq!(state.selected_index, 0);
    }

    #[test]
    fn edit_save_keeps_empty_fields_and_parses_quantity() {
        let mut state = test_state(vec![test_item("A", "Cat")]);
        // Empty name and category keep the old values; quantity parses
        state.apply_item_edit(0, String::new(), vec![], "3".to_string(), None);
        assert_eq!(state.packing_list.items[0].name, "A");
        assert_eq!(state.packing_list.items[0].primary_category(), "Cat");
        assert_eq!(state.packing_list.items[0].quantity, 3);
        // An unparsable quantity keeps the old value
        state.apply_item_edit(
            0,
            "Renamed".to_string(),
            vec!["NewCat".to_string()],
            "x".to_string(),
            None,
        );
        assert_eq!(state.packing_list.items[0].name, "Renamed");
        assert_eq!(state.packing_list.items[0].primary_category(), "NewCat");
        assert_eq!(state.packing_list.items[0].quantity, 3);
        // An out-of-range index is a no-op
        state.apply_item_edit(9, "X".to_string(), vec![], String::new(), None);
        assert_eq!(state.packing_list.items.len(), 1);
    }

    #[test]
    fn dialog_category_dropdown_moves_within_bounds() {
        let mut state = test_state(Vec::new());
        state.input_mode = InputMode::AddingItem {
            field: InputField::Category,
            name: String::new(),
            category: String::new(),
            quantity: String::new(),
            category_list: vec!["One".to_string(), "Two".to_string(), "Three".to_string()],
            category_index: 0,
        };
        state.move_cursor_down();
        match &state.input_mode {
            InputMode::AddingItem { category_index, .. } => assert_eq!(*category_index, 1),
            _ => panic!("expected AddingItem mode"),
        }
        state.move_cursor_up();
        state.move_cursor_up(); // below zero: stays at 0
        match &state.input_mode {
            InputMode::AddingItem { category_index, .. } => assert_eq!(*category_index, 0),
            _ => panic!("expected AddingItem mode"),
        }
        // Dropdown movement only applies when the category field is focused
        state.input_mode = InputMode::AddingItem {
            field: InputField::Name,
            name: String::new(),
            category: String::new(),
            quantity: String::new(),
            category_list: vec!["One".to_string(), "Two".to_string()],
            category_index: 0,
        };
        state.move_cursor_down();
        match &state.input_mode {
            InputMode::AddingItem { category_index, .. } => assert_eq!(*category_index, 0),
            _ => panic!("expected AddingItem mode"),
        }
    }

    // ---- US1 (spec 016): Left/Right cycle the category selection ----

    #[test]
    fn dialog_left_right_cycles_category_within_bounds() {
        let mut state = test_state(Vec::new());
        state.input_mode = InputMode::AddingItem {
            field: InputField::Category,
            name: String::new(),
            category: String::new(),
            quantity: String::new(),
            category_list: vec!["One".to_string(), "Two".to_string(), "Three".to_string()],
            category_index: 0,
        };
        state.cycle_category_selection(1);
        match &state.input_mode {
            InputMode::AddingItem { category_index, .. } => assert_eq!(*category_index, 1),
            _ => panic!("expected AddingItem mode"),
        }
        state.cycle_category_selection(1);
        state.cycle_category_selection(1); // at the end: stays at the last entry
        match &state.input_mode {
            InputMode::AddingItem { category_index, .. } => assert_eq!(*category_index, 2),
            _ => panic!("expected AddingItem mode"),
        }
        state.cycle_category_selection(-1);
        state.cycle_category_selection(-1);
        state.cycle_category_selection(-1); // below zero: stays at 0
        match &state.input_mode {
            InputMode::AddingItem { category_index, .. } => assert_eq!(*category_index, 0),
            _ => panic!("expected AddingItem mode"),
        }
    }

    #[test]
    fn dialog_left_right_only_in_category_field() {
        let mut state = test_state(Vec::new());
        state.input_mode = InputMode::AddingItem {
            field: InputField::Name,
            name: String::new(),
            category: String::new(),
            quantity: String::new(),
            category_list: vec!["One".to_string(), "Two".to_string()],
            category_index: 0,
        };
        state.cycle_category_selection(1);
        match &state.input_mode {
            InputMode::AddingItem { category_index, .. } => assert_eq!(*category_index, 0),
            _ => panic!("expected AddingItem mode"),
        }
        // EditingItem mode behaves the same
        state.input_mode = InputMode::EditingItem {
            index: 0,
            field: InputField::Quantity,
            name: String::new(),
            category: String::new(),
            quantity: String::new(),
            auto_quantity: true,
            category_list: vec!["One".to_string(), "Two".to_string()],
            category_index: 1,
            selected_categories: vec!["One".to_string()],
        };
        state.cycle_category_selection(1);
        match &state.input_mode {
            InputMode::EditingItem { category_index, .. } => assert_eq!(*category_index, 1),
            _ => panic!("expected EditingItem mode"),
        }
        state.input_mode = InputMode::EditingItem {
            index: 0,
            field: InputField::Category,
            name: String::new(),
            category: String::new(),
            quantity: String::new(),
            auto_quantity: true,
            category_list: vec!["One".to_string(), "Two".to_string()],
            category_index: 0,
            selected_categories: vec!["One".to_string()],
        };
        state.cycle_category_selection(1);
        match &state.input_mode {
            InputMode::EditingItem { category_index, .. } => assert_eq!(*category_index, 1),
            _ => panic!("expected EditingItem mode"),
        }
    }

    // ---- US1: auto/fixed quantity mode in the edit dialog ----

    #[test]
    fn mode_field_cycles_in_tab_rotation() {
        // Tab rotation: Name -> Category -> Quantity -> Mode -> Name
        let f = |x: InputField| match x {
            InputField::Name => InputField::Category,
            InputField::Category => InputField::Quantity,
            InputField::Quantity => InputField::Mode,
            InputField::Mode => InputField::Name,
        };
        let mut field = InputField::Name;
        field = f(field);
        assert_eq!(field, InputField::Category);
        field = f(field);
        assert_eq!(field, InputField::Quantity);
        field = f(field);
        assert_eq!(field, InputField::Mode);
        field = f(field);
        assert_eq!(field, InputField::Name);
    }

    #[test]
    fn mode_toggle_flips_auto_quantity_in_edit_dialog() {
        let mut state = test_state(vec![test_item("A", "Cat")]);
        state.input_mode = InputMode::EditingItem {
            index: 0,
            field: InputField::Mode,
            name: "A".to_string(),
            category: "Cat".to_string(),
            quantity: "1".to_string(),
            auto_quantity: true,
            category_list: vec!["Cat".to_string()],
            category_index: 0,
            selected_categories: vec!["Cat".to_string()],
        };
        state.toggle_edit_mode();
        match &state.input_mode {
            InputMode::EditingItem { auto_quantity, .. } => assert!(!*auto_quantity),
            _ => panic!("expected EditingItem mode"),
        }
        state.toggle_edit_mode();
        match &state.input_mode {
            InputMode::EditingItem { auto_quantity, .. } => assert!(*auto_quantity),
            _ => panic!("expected EditingItem mode"),
        }
    }

    #[test]
    fn apply_edit_writes_mode_and_freezes_quantity_when_fixed() {
        // auto -> fixed: the current quantity becomes the fixed value (EC-003)
        let mut state = test_state(vec![test_item("A", "Cat")]);
        state.packing_list.items[0].quantity = 5;
        state.packing_list.items[0].auto_quantity = false; // dialog toggled to fixed
        state.apply_item_edit(0, String::new(), vec![], String::new(), Some(false));
        assert!(!state.packing_list.items[0].auto_quantity);
        assert_eq!(state.packing_list.items[0].quantity, 5);
        // fixed -> auto: the flag flips; the caller recalculates
        state.apply_item_edit(0, String::new(), vec![], String::new(), Some(true));
        assert!(state.packing_list.items[0].auto_quantity);
    }

    #[test]
    fn fixed_quantity_cannot_drop_below_one() {
        // EC-004: no fixed item can hold a quantity below 1
        let mut state = test_state(vec![test_item("A", "Cat")]);
        state.packing_list.items[0].auto_quantity = false;
        state.apply_item_edit(0, String::new(), vec![], "0".to_string(), None);
        assert_eq!(state.packing_list.items[0].quantity, 1);
    }

    // ---- US2: category packing indicator ----

    #[test]
    fn category_all_packed_derivation() {
        // All packed -> all-packed
        assert!(category_all_packed(&[true, true]));
        // One unpacked -> partially packed
        assert!(!category_all_packed(&[true, false, true]));
        // None packed -> partially packed
        assert!(!category_all_packed(&[false]));
        // Empty category -> all-packed (vacuous truth)
        assert!(category_all_packed(&[]));
    }

    // ---- US4: help screen completeness ----

    #[test]
    fn help_lists_every_binding_token() {
        // The tokens the handlers actually respond to (the curated binding
        // list from contracts/input-and-help.md). Every token must appear
        // in the help content; the token list must have no duplicates.
        let tokens = help_binding_tokens();

        // No duplicates (SC-004)
        let mut sorted = tokens.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            tokens.len(),
            "help_binding_tokens contains duplicates"
        );

        // Every token is rendered by the help screen: the entries are
        // built from the same token strings, so the help text contains
        // each token verbatim.
        let help_text = help_screen_text();
        for token in &tokens {
            assert!(
                help_text.contains(token),
                "help screen is missing binding: {token}"
            );
        }
    }

    #[test]
    fn help_screen_text_covers_all_handler_bindings() {
        // The curated handler-binding list (from the code review): every
        // binding the key handlers respond to must be a help token.
        let handler_bindings = [
            // Global
            "Ctrl+C",
            "Ctrl+E",
            "q",
            "?",
            "O",
            // Welcome
            "↑/↓",
            "←/→",
            "+/-",
            "Space",
            "Enter",
            "F",
            "Esc",
            // Resume prompt
            "Y",
            "N",
            // Main
            "Tab",
            "BackTab",
            "Ctrl+Space",
            "C",
            "A",
            "E",
            "D",
            "R",
            // Dialog (Mode field added by US1)
            "Mode",
            // Raw config editor
            "Home/End",
            "Backspace",
            "Ctrl+S",
        ];
        let tokens = help_binding_tokens();
        for binding in handler_bindings {
            assert!(
                tokens.contains(&binding),
                "handler binding missing from help tokens: {binding}"
            );
        }
    }

    // ---- US5 (spec 016): hints and help agree ----

    fn all_hint_surfaces() -> [HintSurface; 8] {
        [
            HintSurface::LeftPanel,
            HintSurface::RightPanel,
            HintSurface::Welcome,
            HintSurface::Config,
            HintSurface::AddEditPopup,
            HintSurface::DeletePopup,
            HintSurface::ModifierEditor,
            HintSurface::RawEditor,
        ]
    }

    #[test]
    fn hint_keys_are_help_tokens() {
        // Every key shown in any surface's hint line must be a help
        // token, so hints cannot show a binding the help omits.
        let tokens = help_binding_tokens();
        for surface in all_hint_surfaces() {
            for (key, _) in hint_entries(surface) {
                assert!(
                    tokens.contains(key),
                    "hint key {key:?} ({surface:?}) missing from help tokens"
                );
            }
        }
    }

    #[test]
    fn hint_entries_match_the_contract_binding_sets() {
        // The drop/stale direction: each surface's entry set must equal
        // the contract's binding set exactly, so removing or corrupting
        // an entry (e.g. deleting ("C", "Collapse") from the left
        // panel) fails this test.
        let expected: [(HintSurface, &[(&str, &str)]); 8] = [
            (
                HintSurface::LeftPanel,
                &[
                    ("↑/↓", "Navigate"),
                    ("Space", "Select"),
                    ("Ctrl+Space", "Category"),
                    ("A", "Add"),
                    ("E", "Edit"),
                    ("D", "Delete"),
                    ("C", "Collapse"),
                    ("R", "Recalculate"),
                    ("Tab", "Switch"),
                    ("?", "Help"),
                ],
            ),
            (
                HintSurface::RightPanel,
                &[
                    ("↑/↓", "Navigate"),
                    ("Space", "Packed"),
                    ("R", "Recalculate"),
                    ("Tab", "Switch"),
                    ("?", "Help"),
                ],
            ),
            (
                HintSurface::Welcome,
                &[
                    ("↑/↓", "Navigate fields"),
                    ("←/→", "Adjust values"),
                    ("+/-", "Weekdays"),
                    ("Enter", "Edit file / Continue"),
                    ("F", "Edit path"),
                    ("O", "Config"),
                    ("Ctrl+E", "Raw editor"),
                    ("q", "Quit"),
                ],
            ),
            (
                HintSurface::Config,
                &[
                    ("Tab", "Section"),
                    ("↑/↓", "Move"),
                    ("A", "Add"),
                    ("E", "Edit"),
                    ("D", "Delete"),
                    ("Ctrl+E", "Raw editor"),
                    ("Esc", "Back"),
                ],
            ),
            (
                HintSurface::AddEditPopup,
                &[
                    ("Tab", "Cycle fields"),
                    ("↑/↓", "Category dropdown"),
                    ("←/→", "Cycle category"),
                    ("Space", "Mode / Category"),
                    ("Enter", "Save"),
                    ("Esc", "Cancel"),
                ],
            ),
            (
                HintSurface::DeletePopup,
                &[("Enter", "Confirm"), ("Esc", "Cancel")],
            ),
            (
                HintSurface::ModifierEditor,
                &[
                    ("Tab", "Switch field"),
                    ("Enter", "Save"),
                    ("Esc", "Cancel"),
                ],
            ),
            (
                HintSurface::RawEditor,
                &[
                    ("↑/↓/←/→", "Cursor"),
                    ("Home/End", "Line start/end"),
                    ("Enter", "New line"),
                    ("Backspace", "Delete"),
                    ("Ctrl+S", "Save & apply"),
                    ("Esc", "Cancel"),
                ],
            ),
        ];
        for (surface, contract) in expected {
            let actual = hint_entries(surface);
            assert_eq!(
                actual, contract,
                "{surface:?} hint entries drifted from the contract"
            );
        }
    }

    #[test]
    fn hint_lines_use_the_unified_style() {
        // The single constructor renders " | " separators and no
        // long-form arrow words; the corrupted text is gone.
        let line = hint_line(hint_entries(HintSurface::LeftPanel));
        let text: String = line.spans.iter().map(|s| s.content.to_string()).collect();
        assert!(text.contains(" | "), "hint line must use | separators");
        assert!(!text.contains("Up/Down"), "hint line must use glyphs");
        assert!(!text.contains("Left/Right"), "hint line must use glyphs");
        let all_text = format!("{}{}", help_screen_text(), {
            let mut s = String::new();
            for surface in [
                HintSurface::LeftPanel,
                HintSurface::RightPanel,
                HintSurface::Welcome,
                HintSurface::Config,
                HintSurface::AddEditPopup,
                HintSurface::DeletePopup,
                HintSurface::ModifierEditor,
                HintSurface::RawEditor,
            ] {
                for (k, a) in hint_entries(surface) {
                    s.push_str(k);
                    s.push_str(a);
                }
            }
            s
        });
        assert!(
            !all_text.contains("fields.category"),
            "corrupted hint text must be gone"
        );
    }

    #[test]
    fn hint_rows_fit_width_and_keep_every_entry() {
        // At the minimum panel width (39 columns at an 80-column
        // terminal), every entry still appears and no row overflows.
        let entries = hint_entries(HintSurface::LeftPanel);
        let rows = hint_rows(entries, 39);
        let mut seen: Vec<(&str, &str)> = Vec::new();
        for row in &rows {
            let text: String = row.spans.iter().map(|s| s.content.to_string()).collect();
            assert!(
                text.chars().count() <= 39,
                "hint row exceeds the panel width: {text:?}"
            );
            // Count full "KEY Action" occurrences, not substrings
            for (key, action) in entries {
                if text.contains(&format!("{key} {action}")) {
                    seen.push((key, action));
                }
            }
        }
        // Every entry appears exactly once across the rows
        assert_eq!(seen.len(), entries.len(), "every entry must appear once");
        // A wide width packs everything into one row
        assert_eq!(hint_rows(entries, 200).len(), 1);
        // An empty entry list still yields one (blank) row
        assert_eq!(hint_rows(&[], 39).len(), 1);
    }

    // ---- US1/US3 (spec 018): the recalculate action ----

    #[test]
    fn recalculate_updates_auto_and_preserves_fixed() {
        let mut auto_item = test_item("Auto", "Cat");
        auto_item.quantity = 99; // stale bogus value
        let mut fixed_item = test_item("Fixed", "Cat");
        fixed_item.quantity = 3;
        fixed_item.auto_quantity = false;
        let mut state = test_state(vec![auto_item, fixed_item]);
        // Non-quantity state the action must not touch
        state.packing_list.items[0].editable = true;
        state.packing_list.items[0].packed = true;
        state.collapsed_categories = vec!["Cat".to_string()];

        state
            .recalculate_quantities(&QuantityCalculation::new())
            .unwrap();

        // The auto item recalculates: 7 weekdays × 1.0 standard × 1.0 mixed = 7
        assert_eq!(state.packing_list.items[0].quantity, 7);
        // The fixed item keeps its value
        assert_eq!(state.packing_list.items[1].quantity, 3);
        // Non-quantity state is untouched (SC-003)
        assert!(state.packing_list.items[0].editable);
        assert!(state.packing_list.items[0].packed);
        assert_eq!(state.collapsed_categories, vec!["Cat".to_string()]);

        // Idempotent: a second run with unchanged inputs changes nothing
        state
            .recalculate_quantities(&QuantityCalculation::new())
            .unwrap();
        assert_eq!(state.packing_list.items[0].quantity, 7);
        assert_eq!(state.packing_list.items[1].quantity, 3);
    }

    // ---- US6: multi-category items ----

    #[test]
    fn multi_category_item_appears_under_each_category() {
        let mut item = test_item("A", "Cat");
        item.categories = vec!["Cat".to_string(), "Gear".to_string()];
        let state = test_state(vec![item]);
        let rows = state.left_rows();
        // The item appears under both category headers
        let mut cat_positions = Vec::new();
        let mut item_count = 0;
        for (i, row) in rows.iter().enumerate() {
            match row {
                PanelRow::Category(c) => {
                    if c == "Cat" || c == "Gear" {
                        cat_positions.push(i);
                    }
                }
                PanelRow::Item(idx) => {
                    if state.packing_list.items[*idx].name == "A" {
                        item_count += 1;
                    }
                }
            }
        }
        assert_eq!(cat_positions.len(), 2, "both category headers present");
        assert_eq!(item_count, 2, "item shown under both headers");
    }

    #[test]
    fn multi_category_selection_is_shared() {
        // One item, two memberships: toggling selection through either
        // occurrence flips the item once, everywhere
        let mut item = test_item("A", "Cat");
        item.categories = vec!["Cat".to_string(), "Gear".to_string()];
        let mut state = test_state(vec![item]);
        // Cursor on the item under the Cat header (row 1)
        state.selected_index = 1;
        state.toggle_item_selection();
        assert!(!state.packing_list.items[0].editable);
        // The compiled list shows the item at most once (primary category)
        let right = state.right_item_order();
        assert_eq!(right.len(), 0, "deselected item gone from compiled list");
        // Reselect through the Gear occurrence
        state.selected_index = 2; // Gear header is row 2, item row 3
        state.selected_index = 3;
        state.toggle_item_selection();
        assert!(state.packing_list.items[0].editable);
        let right = state.right_item_order();
        assert_eq!(right.len(), 1, "item appears once in compiled list");
    }

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
    fn old_config_without_trip_params_loads() {
        // Config files written before the modifiers feature only had
        // categories and items; they must keep loading with defaults
        let yaml = "categories:\n- Clothing\nitems:\n- name: T-Shirt\n  category: Clothing\n  quantity: 5\n";
        let config: AppConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.items.len(), 1);
        assert_eq!(config.items[0].name, "T-Shirt");
        // Old quantity is ignored for auto items; the item recalculates
        assert!(config.items[0].auto_quantity);
        assert_eq!(config.items[0].quantity, Some(5)); // kept on load, unused for auto
        assert_eq!(config.weekdays, 0); // serde default, clamped to Weekdays default on load
        assert_eq!(config.travel_style, "standard");
        assert_eq!(config.weather, "mixed");
        assert!(!config.modifiers.travel_styles.is_empty());
        assert!(!config.modifiers.weather.is_empty());
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
                },
                SavedItem {
                    name: "Toothbrush".to_string(),
                    categories: vec!["Toiletries".to_string()],
                    quantity: Some(1),
                    auto_quantity: false,
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
                },
                TripItem {
                    name: "Old Item".to_string(),
                    category: "Gone".to_string(),
                    packed: false,
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
    fn legacy_files_with_activity_fields_load_and_save_clean() {
        // US3/SC-003: pre-feature files carrying activity fields load
        // without error (serde ignores unknown fields); US3/SC-004: new
        // saves contain no activity keys
        let legacy_config = "categories:\n- Clothing\nitems:\n- name: T-Shirt\n  categories:\n  - Clothing\n  auto_quantity: true\nweekdays: 7\ntravel_style: standard\nweather: mixed\nselected_activities:\n- hiking\nmodifiers:\n  travel_styles:\n  - name: standard\n    multiplier: 1.0\n  weather:\n  - name: mixed\n    multiplier: 1.0\n  activities:\n  - name: hiking\n    multiplier: 1.5\n";
        let config: AppConfig = serde_yaml::from_str(legacy_config).unwrap();
        assert_eq!(config.items.len(), 1);
        assert_eq!(config.weekdays, 7);

        let legacy_trip = "weekdays: 7\ntravel_style: standard\nweather: mixed\nselected_activities:\n- hiking\nitems:\n- name: T-Shirt\n  category: Clothing\n  packed: true\n";
        let trip: crate::config::trip_file::TripFile = serde_yaml::from_str(legacy_trip).unwrap();
        assert_eq!(trip.items.len(), 1);
        assert!(trip.items[0].packed);

        // Serialized output contains no activity keys (SC-004)
        let config_yaml = serde_yaml::to_string(&config).unwrap();
        assert!(
            !config_yaml.contains("activit"),
            "config save must not contain activity fields: {config_yaml}"
        );
        let trip_yaml = serde_yaml::to_string(&trip).unwrap();
        assert!(
            !trip_yaml.contains("activit"),
            "trip save must not contain activity fields: {trip_yaml}"
        );
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
}

// State for drawing (cloneable)
#[derive(Clone)]
struct AppDrawState {
    screen: Screen,
    packing_list: PackingList,
    weekdays: Weekdays,
    travel_style: String,
    weather: String,
    modifiers: ModifiersConfig,
    focus: Focus,
    collapsed_categories: Vec<String>,
    selected_index: usize,
    focus_side: FocusSide,
    input_mode: InputMode,
    left_scroll: usize,
    right_scroll: usize,
    /// Path of the packing list file to persist (where the trip is saved on exit)
    trip_file_path: PathBuf,
    /// Text input for the trip file path on the welcome screen
    trip_file_input: String,
    /// Whether the trip file name is being edited
    editing_trip_file: bool,
    /// Whether the user is being asked to resume the previous trip file
    resume_prompt: bool,
    /// Trip file remembered from the previous session, offered for resume
    remembered_trip_file: Option<PathBuf>,
    /// Config screen state
    config_section: ConfigSection,
    config_cursor: usize,
    config_scroll: usize,
    /// Editor for adding/editing a modifier on the config screen
    modifier_editor: Option<ModifierEditor>,
    /// Screen to return to when leaving Config/ConfigEditor/Help
    previous_screen: Screen,
    /// Raw text of the config file being edited in the ConfigEditor screen
    config_editor_text: Vec<String>,
    /// Cursor (line, column) of the raw config editor
    config_editor_cursor: (usize, usize),
    /// Scroll offset of the raw config editor
    config_editor_scroll: usize,
    /// Error message shown in the raw config editor after a failed save
    config_editor_error: Option<String>,
}

/// Sections of the config screen
#[derive(Debug, Clone, Copy, PartialEq)]
enum ConfigSection {
    TravelStyles,
    Weather,
}

impl ConfigSection {
    fn title(&self) -> &'static str {
        match self {
            ConfigSection::TravelStyles => "Travel Styles",
            ConfigSection::Weather => "Weather",
        }
    }

    fn next(&self) -> Self {
        match self {
            ConfigSection::TravelStyles => ConfigSection::Weather,
            ConfigSection::Weather => ConfigSection::TravelStyles,
        }
    }

    fn prev(&self) -> Self {
        match self {
            ConfigSection::TravelStyles => ConfigSection::Weather,
            ConfigSection::Weather => ConfigSection::TravelStyles,
        }
    }
}

/// In-progress modifier being added or edited on the config screen
#[derive(Debug, Clone)]
struct ModifierEditor {
    /// Index into the section's list; None when adding a new modifier
    index: Option<usize>,
    section: ConfigSection,
    name: String,
    multiplier: String,
    field: ModifierField,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ModifierField {
    Name,
    Multiplier,
}

// A navigable row of a panel: a category header or an item
#[derive(Clone, Debug)]
enum PanelRow {
    Category(String),
    Item(usize), // index into packing_list.items
}

impl AppDrawState {
    fn left_rows(&self) -> Vec<PanelRow> {
        // Rows of the left panel in display order: each category header,
        // followed by its items unless the category is collapsed
        let mut categories_map: std::collections::BTreeMap<String, Vec<usize>> =
            std::collections::BTreeMap::new();
        for (idx, item) in self.packing_list.items.iter().enumerate() {
            for category in &item.categories {
                categories_map
                    .entry(category.clone())
                    .or_default()
                    .push(idx);
            }
        }

        let mut rows = Vec::new();
        for (category, indices) in &categories_map {
            rows.push(PanelRow::Category(category.clone()));
            if !self.collapsed_categories.contains(category) {
                for idx in indices {
                    rows.push(PanelRow::Item(*idx));
                }
            }
        }
        rows
    }

    fn right_item_order(&self) -> Vec<usize> {
        // Selected items in the order they are displayed on the right
        // panel (grouped by category)
        let mut categories_map: std::collections::BTreeMap<String, Vec<usize>> =
            std::collections::BTreeMap::new();
        for (idx, item) in self.packing_list.items.iter().enumerate() {
            if item.editable {
                categories_map
                    .entry(item.primary_category().to_string())
                    .or_default()
                    .push(idx);
            }
        }

        let mut order = Vec::new();
        for indices in categories_map.values() {
            order.extend(indices);
        }
        order
    }

    // Rows the cursor moves over on the focused side
    fn active_rows(&self) -> Vec<PanelRow> {
        match self.focus_side {
            FocusSide::Left => self.left_rows(),
            FocusSide::Right => self
                .right_item_order()
                .into_iter()
                .map(PanelRow::Item)
                .collect(),
        }
    }

    // The item the cursor is on, if it is on an item row
    fn cursor_item_index(&self) -> Option<usize> {
        let rows = self.active_rows();
        match rows.get(self.selected_index) {
            Some(PanelRow::Item(index)) => Some(*index),
            _ => None,
        }
    }

    // The category the cursor is on (header row or one of its items)
    fn cursor_category(&self) -> Option<String> {
        let rows = self.active_rows();
        match rows.get(self.selected_index) {
            Some(PanelRow::Category(c)) => Some(c.clone()),
            Some(PanelRow::Item(index)) => Some(
                self.packing_list.items[*index]
                    .primary_category()
                    .to_string(),
            ),
            None => None,
        }
    }

    // Collapse (true) or expand (false) the category under the cursor.
    // When collapsing, the cursor moves to the category's header row so it
    // stays visible.
    fn set_category_collapsed(&mut self, collapsed: bool) {
        let Some(category) = self.cursor_category() else {
            return;
        };

        if collapsed {
            if !self.collapsed_categories.contains(&category) {
                self.collapsed_categories.push(category.clone());
            }
            // Move the cursor to the category's header row
            let rows = self.active_rows();
            if let Some(row) = rows
                .iter()
                .position(|r| matches!(r, PanelRow::Category(c) if *c == category))
            {
                self.selected_index = row;
            }
        } else {
            self.collapsed_categories.retain(|c| c != &category);
        }
    }

    fn toggle_category_collapse(&mut self) {
        // Toggle collapse of the category the cursor is on
        let collapsed = self
            .cursor_category()
            .map(|c| self.collapsed_categories.contains(&c))
            .unwrap_or(false);
        self.set_category_collapsed(!collapsed);
    }

    fn toggle_item_selection(&mut self) {
        // Toggle selection of the currently selected item
        if let Some(item_index) = self.cursor_item_index() {
            let item = &mut self.packing_list.items[item_index];
            item.editable = !item.editable;
        }
    }

    fn toggle_category_selection(&mut self) {
        // Toggle selection of all items in the current category, then
        // collapse the group when items were added to the trip and expand
        // it when they were removed
        let Some(category) = self.cursor_category() else {
            return;
        };

        let was_selected = self
            .packing_list
            .items
            .iter()
            .any(|item| item.in_category(&category) && item.editable);

        for item in &mut self.packing_list.items {
            if item.in_category(&category) {
                item.editable = !item.editable;
            }
        }

        // Follow the selection state: added -> collapse, removed -> expand
        let now_selected = !was_selected;
        self.set_category_collapsed(!now_selected);
    }

    fn toggle_item_packed(&mut self) {
        // Only toggle packed status when on right side
        if self.focus_side == FocusSide::Right
            && let Some(item_index) = self.cursor_item_index()
        {
            self.packing_list.items[item_index].packed =
                !self.packing_list.items[item_index].packed;
        }
    }

    // Toggle the quantity mode (automatic/fixed) in the edit dialog
    fn toggle_edit_mode(&mut self) {
        if let InputMode::EditingItem { auto_quantity, .. } = &mut self.input_mode {
            *auto_quantity = !*auto_quantity;
        }
    }

    // Cycle the category selection in the add/edit dialog (Left/Right);
    // clamped at the list bounds, only when the Category field is focused
    fn cycle_category_selection(&mut self, direction: i32) {
        let (field, category_list, category_index) = match &mut self.input_mode {
            InputMode::AddingItem {
                field,
                category_list,
                category_index,
                ..
            } => (field, category_list, category_index),
            InputMode::EditingItem {
                field,
                category_list,
                category_index,
                ..
            } => (field, category_list, category_index),
            _ => return,
        };
        if *field == InputField::Category {
            let target = (*category_index as i32 + direction)
                .clamp(0, category_list.len().saturating_sub(1) as i32);
            *category_index = target as usize;
        }
    }

    /// Recalculate every automatic item's quantity from the current
    /// trip parameters and per-item configuration. Fixed items keep
    /// their values; packed, selection, and collapse states are
    /// untouched; nothing is written to disk. Idempotent for
    /// unchanged inputs.
    fn recalculate_quantities(&mut self, calc: &QuantityCalculation) -> Result<()> {
        // Resolve multipliers from the config; fall back to 1.0 for unknown names
        let style_multiplier = self
            .modifiers
            .travel_style_multiplier(&self.travel_style)
            .unwrap_or(1.0);
        let weather_multiplier = self
            .modifiers
            .weather_multiplier(&self.weather)
            .unwrap_or(1.0);

        let item_specs: Vec<ItemQuantitySpec> = self
            .packing_list
            .items
            .iter()
            .map(|item| {
                if item.auto_quantity {
                    ItemQuantitySpec::Auto
                } else {
                    ItemQuantitySpec::Fixed(item.quantity)
                }
            })
            .collect();

        let input = QuantityCalculationInput {
            weekdays: self.weekdays,
            item_specs,
            travel_style: self.travel_style.clone(),
            travel_style_multiplier: style_multiplier,
            weather: self.weather.clone(),
            weather_multiplier,
        };

        let output = calc.calculate(&input)?;

        // Update item quantities
        for (i, quantity) in output.item_quantities.iter().enumerate() {
            if i < self.packing_list.items.len() {
                self.packing_list.items[i].quantity = *quantity;
            }
        }

        Ok(())
    }

    // Apply the edit dialog's fields to an item; empty fields keep the
    // old value, an unparsable quantity keeps the old quantity. The
    // optional mode, when given, writes the dialog's auto/fixed flag.
    // The categories replace the item's memberships (never empty).
    fn apply_item_edit(
        &mut self,
        index: usize,
        name: String,
        categories: Vec<String>,
        quantity: String,
        auto_quantity: Option<bool>,
    ) {
        if index >= self.packing_list.items.len() {
            return;
        }
        let item = &mut self.packing_list.items[index];

        if !name.is_empty() {
            item.name = name;
        }
        if !categories.is_empty() {
            // Deduplicate memberships, preserving order
            let mut seen = std::collections::BTreeSet::new();
            item.categories = categories
                .into_iter()
                .filter(|c| seen.insert(c.clone()))
                .collect();
        }
        if let Ok(qty) = quantity.parse::<u32>() {
            // Quantities never drop below 1 (spec 003 EC)
            item.quantity = qty.max(1);
        }
        if let Some(auto) = auto_quantity {
            item.auto_quantity = auto;
        }
    }

    // Move the cursor up one row (or field / dropdown entry, per screen
    // and input mode); the caller handles the left-panel sync
    fn move_cursor_up(&mut self) {
        match self.screen {
            Screen::Welcome => {
                // Up/Down navigate between fields; Left/Right adjust values
                self.focus = self.focus.prev();
            }
            Screen::Config | Screen::ConfigEditor | Screen::Help => {}
            Screen::Main => {
                if matches!(self.input_mode, InputMode::Normal) {
                    // Navigate up through the visible rows of the focused side
                    if self.selected_index > 0 {
                        self.selected_index -= 1;
                    }
                } else if let InputMode::AddingItem {
                    field,
                    category_index,
                    ..
                } = &mut self.input_mode
                {
                    // Only navigate category dropdown when category field is selected
                    if *field == InputField::Category && *category_index > 0 {
                        *category_index -= 1;
                    }
                } else if let InputMode::EditingItem {
                    field,
                    category_index,
                    ..
                } = &mut self.input_mode
                {
                    // Only navigate category dropdown when category field is selected
                    if *field == InputField::Category && *category_index > 0 {
                        *category_index -= 1;
                    }
                }
            }
        }
    }

    // Move the cursor down one row (or field / dropdown entry, per screen
    // and input mode); the caller handles the left-panel sync
    fn move_cursor_down(&mut self) {
        match self.screen {
            Screen::Welcome => {
                // Up/Down navigate between fields; Left/Right adjust values
                self.focus = self.focus.next();
            }
            Screen::Config | Screen::ConfigEditor | Screen::Help => {}
            Screen::Main => {
                if matches!(self.input_mode, InputMode::Normal) {
                    // Navigate down through the visible rows of the focused side
                    let rows = self.active_rows();
                    if self.selected_index < rows.len().saturating_sub(1) {
                        self.selected_index += 1;
                    }
                } else if let InputMode::AddingItem {
                    field,
                    category_index,
                    category_list,
                    ..
                } = &mut self.input_mode
                {
                    // Only navigate category dropdown when category field is selected
                    if *field == InputField::Category
                        && *category_index < category_list.len().saturating_sub(1)
                    {
                        *category_index += 1;
                    }
                } else if let InputMode::EditingItem {
                    field,
                    category_index,
                    category_list,
                    ..
                } = &mut self.input_mode
                {
                    // Only navigate category dropdown when category field is selected
                    if *field == InputField::Category
                        && *category_index < category_list.len().saturating_sub(1)
                    {
                        *category_index += 1;
                    }
                }
            }
        }
    }
}

pub struct App {
    state: AppState,
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    draw_state: AppDrawState,
    quantity_calculation: QuantityCalculation,
    config_path: PathBuf,
}

impl App {
    pub fn new(trip_file_arg: Option<String>) -> Result<Self> {
        // Initialize terminal
        enable_raw_mode().context("Failed to enable raw mode")?;
        stdout()
            .execute(EnterAlternateScreen)
            .context("Failed to enter alternate screen")?;

        let backend = CrosstermBackend::new(stdout());
        let terminal = Terminal::new(backend).context("Failed to create terminal")?;

        // Determine config path
        let config_path = dirs::config_dir()
            .context("Could not get config directory")?
            .join("pack-liste");

        let config_file = config_path.join("config.yaml");

        // Load or create default config
        let (mut packing_list, trip_params) = if config_file.exists() {
            load_config(&config_file)?
        } else {
            create_default_config(&config_file)?
        };

        // A trip file passed on the command line is loaded directly; the
        // remembered path only prompts the user to resume on the welcome screen
        let cli_trip = trip_file_arg.filter(|s| !s.trim().is_empty());
        let remembered_trip = trip_params
            .last_trip_file
            .clone()
            .filter(|s| !s.trim().is_empty());

        let trip_file_path = cli_trip
            .as_deref()
            .map(resolve_trip_path)
            .unwrap_or_default();
        let mut resume_prompt = false;
        let mut remembered_trip_file: Option<PathBuf> = None;

        // Trip parameters, possibly overridden by a loaded trip file
        let mut weekdays = trip_params.weekdays;
        let mut travel_style = trip_params.travel_style;
        let mut weather = trip_params.weather;

        // Load and reconcile the trip file if a path is set
        if !trip_file_path.as_os_str().is_empty() {
            if let Some(mut trip) = TripFile::load(&trip_file_path)? {
                // Reconcile selected items against the current input config
                let config_items: Vec<(String, Vec<String>)> = packing_list
                    .items
                    .iter()
                    .map(|item| (item.name.clone(), item.categories.clone()))
                    .collect();
                trip.reconcile(&config_items);

                // Restore trip parameters from the trip file
                weekdays = Weekdays::new(trip.weekdays).unwrap_or(weekdays);
                travel_style = trip.travel_style;
                weather = trip.weather;

                // Reset the default selection, then restore from the file
                for item in &mut packing_list.items {
                    item.editable = false;
                    item.packed = false;
                    if let Some(t) = trip
                        .items
                        .iter()
                        .find(|t| t.name.eq_ignore_ascii_case(&item.name))
                    {
                        item.editable = true;
                        item.packed = t.packed;
                    }
                }
            } else {
                // A new trip file starts with every item selected
                select_all_items(&mut packing_list);
            }
        } else {
            // Offer to resume the remembered trip file on the welcome screen
            if let Some(remembered) = remembered_trip {
                let path = resolve_trip_path(&remembered);
                if path.exists() {
                    remembered_trip_file = Some(path);
                    resume_prompt = true;
                }
            }
            // A fresh list starts with every item selected
            select_all_items(&mut packing_list);
        }

        let draw_state = AppDrawState {
            screen: Screen::Welcome,
            packing_list,
            weekdays,
            travel_style,
            weather,
            modifiers: trip_params.modifiers,
            focus: Focus::TripFile,
            collapsed_categories: Vec::new(),
            selected_index: 0,
            focus_side: FocusSide::Left,
            input_mode: InputMode::Normal,
            left_scroll: 0,
            right_scroll: 0,
            trip_file_path,
            trip_file_input: String::new(),
            editing_trip_file: false,
            resume_prompt,
            remembered_trip_file,
            config_section: ConfigSection::TravelStyles,
            config_cursor: 0,
            config_scroll: 0,
            modifier_editor: None,
            previous_screen: Screen::Welcome,
            config_editor_text: Vec::new(),
            config_editor_cursor: (0, 0),
            config_editor_scroll: 0,
            config_editor_error: None,
        };

        let mut app = Self {
            state: AppState::Running,
            terminal,
            draw_state,
            quantity_calculation: QuantityCalculation::new(),
            config_path: config_file,
        };

        // Initialize quantities
        app.update_quantities()?;

        Ok(app)
    }

    fn save_config(&self) -> Result<()> {
        let config = AppConfig {
            categories: self.get_unique_categories(),
            items: self
                .draw_state
                .packing_list
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
            weekdays: self.draw_state.weekdays.get(),
            travel_style: self.draw_state.travel_style.clone(),
            weather: self.draw_state.weather.clone(),
            modifiers: self.draw_state.modifiers.clone(),
            last_trip_file: if self.draw_state.trip_file_path.as_os_str().is_empty() {
                // No new path chosen: keep remembering the previous trip file
                self.draw_state
                    .remembered_trip_file
                    .as_ref()
                    .map(|p| p.display().to_string())
            } else {
                Some(self.draw_state.trip_file_path.display().to_string())
            },
        };

        let yaml = serde_yaml::to_string(&config).context("Failed to serialize config")?;

        // Ensure config directory exists
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).context("Failed to create config directory")?;
        }

        fs::write(&self.config_path, yaml).context("Failed to write config file")?;

        Ok(())
    }

    /// Persist the compiled packing list to the trip file, in place.
    /// Quantities are not persisted; they are recalculated on load.
    fn save_trip_file(&self) -> Result<()> {
        if self.draw_state.trip_file_path.as_os_str().is_empty() {
            return Ok(());
        }

        let trip = TripFile {
            weekdays: self.draw_state.weekdays.get(),
            travel_style: self.draw_state.travel_style.clone(),
            weather: self.draw_state.weather.clone(),
            items: self
                .draw_state
                .packing_list
                .items
                .iter()
                .filter(|item| item.editable)
                .map(|item| TripItem {
                    name: item.name.clone(),
                    category: item.primary_category().to_string(),
                    packed: item.packed,
                })
                .collect(),
        };

        trip.save(&self.draw_state.trip_file_path)
    }

    fn get_unique_categories(&self) -> Vec<String> {
        let mut categories: Vec<String> = self
            .draw_state
            .packing_list
            .items
            .iter()
            .flat_map(|item| item.categories.iter().cloned())
            .collect();
        categories.sort();
        categories.dedup();
        categories
    }

    /// Set the trip file path from user input and load that trip file if it
    /// exists, restoring its trip parameters and packed state.
    fn apply_trip_file_path(&mut self, input: &str) -> Result<()> {
        let path = resolve_trip_path(input);
        self.draw_state.trip_file_path = path.clone();
        self.draw_state.editing_trip_file = false;

        // Load the trip file if it exists; a missing file means a new trip
        let Some(mut trip) = TripFile::load(&path)? else {
            // A new trip starts with every item selected
            select_all_items(&mut self.draw_state.packing_list);
            return self.update_quantities();
        };

        // Reconcile selected items against the current input config
        let config_items: Vec<(String, Vec<String>)> = self
            .draw_state
            .packing_list
            .items
            .iter()
            .map(|item| (item.name.clone(), item.categories.clone()))
            .collect();
        trip.reconcile(&config_items);

        // Restore trip parameters from the trip file
        self.draw_state.weekdays = Weekdays::new(trip.weekdays).unwrap_or(self.draw_state.weekdays);
        self.draw_state.travel_style = trip.travel_style;
        self.draw_state.weather = trip.weather;

        // Reset the previous trip's selection, then restore from the file
        for item in &mut self.draw_state.packing_list.items {
            item.editable = false;
            item.packed = false;
            if let Some(t) = trip
                .items
                .iter()
                .find(|t| t.name.eq_ignore_ascii_case(&item.name))
            {
                item.editable = true;
                item.packed = t.packed;
            }
        }

        self.update_quantities()
    }

    fn get_all_categories(&self) -> Vec<String> {
        let mut categories: Vec<String> = self
            .draw_state
            .packing_list
            .items
            .iter()
            .flat_map(|item| item.categories.iter().cloned())
            .collect();
        categories.sort();
        categories.dedup();
        categories
    }

    // Cycle through the configured travel styles (direction: 1 = forward, -1 = backward)
    fn cycle_travel_style(&mut self, direction: i32) {
        let styles = &self.draw_state.modifiers.travel_styles;
        if styles.is_empty() {
            return;
        }
        let current = styles
            .iter()
            .position(|m| m.name == self.draw_state.travel_style);
        let next = match current {
            Some(i) => (i as i32 + direction).rem_euclid(styles.len() as i32) as usize,
            None => 0,
        };
        self.draw_state.travel_style = styles[next].name.clone();
    }

    // Cycle through the configured weather options (direction: 1 = forward, -1 = backward)
    fn cycle_weather(&mut self, direction: i32) {
        let weather = &self.draw_state.modifiers.weather;
        if weather.is_empty() {
            return;
        }
        let current = weather
            .iter()
            .position(|m| m.name == self.draw_state.weather);
        let next = match current {
            Some(i) => (i as i32 + direction).rem_euclid(weather.len() as i32) as usize,
            None => 0,
        };
        self.draw_state.weather = weather[next].name.clone();
    }

    fn update_quantities(&mut self) -> Result<()> {
        self.draw_state
            .recalculate_quantities(&self.quantity_calculation)
    }

    pub fn run(&mut self) -> Result<()> {
        while let AppState::Running = self.state {
            self.draw()?;
            if self.handle_input()? {
                break;
            }
        }

        // Save config and trip file on every exit path (q, Esc, Ctrl+C)
        self.save_config()?;
        self.save_trip_file()?;

        Ok(())
    }

    fn draw(&mut self) -> Result<()> {
        let mut state = self.draw_state.clone();

        self.terminal.draw(|f| {
            let size = f.area();

            match state.screen {
                Screen::Welcome => {
                    draw_welcome_screen(f, size, &state);
                }
                Screen::Main => match state.input_mode {
                    InputMode::Normal => {
                        draw_main_screen(f, size, &mut state);
                    }
                    InputMode::AddingItem {
                        field,
                        ref name,
                        ref category,
                        ref quantity,
                        ref category_list,
                        category_index,
                    } => {
                        let fields = PopupFields {
                            field,
                            name,
                            category,
                            quantity,
                            auto_quantity: None,
                            category_list,
                            category_index,
                        };
                        draw_add_item_popup(f, size, &fields);
                    }
                    InputMode::EditingItem {
                        field,
                        ref name,
                        ref category,
                        ref quantity,
                        auto_quantity,
                        ref category_list,
                        category_index,
                        ..
                    } => {
                        let fields = PopupFields {
                            field,
                            name,
                            category,
                            quantity,
                            auto_quantity: Some(auto_quantity),
                            category_list,
                            category_index,
                        };
                        draw_edit_item_popup(f, size, &fields);
                    }
                    InputMode::ConfirmDelete { index } => {
                        draw_confirm_delete_popup(f, size, index, &state.packing_list);
                    }
                },
                Screen::Help => {
                    draw_help_screen(f, size);
                }
                Screen::Config => {
                    draw_config_screen(f, size, &state);
                }
                Screen::ConfigEditor => {
                    draw_config_editor_screen(f, size, &mut state);
                }
            }
        })?;

        // Persist the scroll offsets computed during rendering
        self.draw_state.left_scroll = state.left_scroll;
        self.draw_state.right_scroll = state.right_scroll;
        self.draw_state.config_editor_scroll = state.config_editor_scroll;
        Ok(())
    }

    fn handle_input(&mut self) -> Result<bool> {
        if event::poll(std::time::Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            // Terminate on Ctrl+C from any screen or input mode
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                self.state = AppState::Exit;
                return Ok(true);
            }

            // Ctrl+E opens the raw config file editor from any screen
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('e') {
                self.open_config_editor()?;
                return Ok(false);
            }

            // Resume prompt: Y/Enter loads the previous trip file,
            // N/Esc declines and starts a fresh list
            if self.draw_state.resume_prompt {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                        self.draw_state.resume_prompt = false;
                        if let Some(path) = self.draw_state.remembered_trip_file.clone() {
                            let input = path.display().to_string();
                            self.apply_trip_file_path(&input)?;
                        }
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        self.draw_state.resume_prompt = false;
                        // Fresh list: keep everything selected; the
                        // remembered path stays saved for the next run
                        select_all_items(&mut self.draw_state.packing_list);
                        self.update_quantities()?;
                    }
                    _ => {}
                }
                return Ok(false);
            }

            // Check for Ctrl+Space first
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char(' ') {
                if self.draw_state.screen == Screen::Main
                    && matches!(self.draw_state.input_mode, InputMode::Normal)
                    && self.draw_state.focus_side == FocusSide::Left
                {
                    self.toggle_category_selection();
                }
                return Ok(false);
            }

            // Left/Right adjust the value of the focused welcome field
            if self.draw_state.screen == Screen::Welcome
                && !self.draw_state.editing_trip_file
                && matches!(key.code, KeyCode::Left | KeyCode::Right)
            {
                let direction = if key.code == KeyCode::Right { 1 } else { -1 };
                match self.draw_state.focus {
                    Focus::TripFile => {
                        // No value to adjust; Enter opens the file name editor
                    }
                    Focus::Weekdays => {
                        let target = self.draw_state.weekdays.get() as i32 + direction;
                        if target >= Weekdays::MIN as i32
                            && target <= Weekdays::MAX as i32
                            && let Ok(new_weekdays) = Weekdays::new(target as u32)
                        {
                            self.draw_state.weekdays = new_weekdays;
                            self.update_quantities()?;
                        }
                    }
                    Focus::TravelStyle => {
                        self.cycle_travel_style(direction);
                        self.update_quantities()?;
                    }
                    Focus::Weather => {
                        self.cycle_weather(direction);
                        self.update_quantities()?;
                    }
                }
                return Ok(false);
            }

            // Route text input to the add/edit popups
            if self.draw_state.screen == Screen::Main
                && matches!(
                    self.draw_state.input_mode,
                    InputMode::AddingItem { .. } | InputMode::EditingItem { .. }
                )
                && matches!(key.code, KeyCode::Char(_) | KeyCode::Backspace)
            {
                self.handle_add_edit_input(key);
                return Ok(false);
            }

            // The config screen handles all its own keys
            if self.draw_state.screen == Screen::Config {
                return self.handle_config_screen_input(key);
            }

            // The raw config editor handles all its own keys
            if self.draw_state.screen == Screen::ConfigEditor {
                return self.handle_config_editor_input(key);
            }

            // Trip file path editing on the welcome screen
            if self.draw_state.screen == Screen::Welcome && self.draw_state.editing_trip_file {
                match key.code {
                    KeyCode::Enter => {
                        let input = self.draw_state.trip_file_input.trim().to_string();
                        // The file path is required: empty input keeps the editor open
                        if !input.is_empty() {
                            self.apply_trip_file_path(&input)?;
                        }
                    }
                    KeyCode::Esc => {
                        self.draw_state.editing_trip_file = false;
                    }
                    KeyCode::Backspace => {
                        self.draw_state.trip_file_input.pop();
                    }
                    KeyCode::Char(c) => {
                        self.draw_state.trip_file_input.push(c);
                    }
                    _ => {}
                }
                return Ok(false);
            }

            match key.code {
                KeyCode::Esc => {
                    match self.draw_state.screen {
                        Screen::Welcome => {
                            self.state = AppState::Exit;
                            return Ok(true);
                        }
                        Screen::Help => {
                            self.draw_state.screen = self.draw_state.previous_screen;
                        }
                        Screen::Config => {
                            // Cancel the modifier editor, else leave the config screen
                            if self.draw_state.modifier_editor.is_some() {
                                self.draw_state.modifier_editor = None;
                            } else {
                                self.draw_state.screen = self.draw_state.previous_screen;
                            }
                        }
                        Screen::ConfigEditor => {
                            // Discard edits and return
                            self.draw_state.screen = self.draw_state.previous_screen;
                        }
                        Screen::Main => {
                            match self.draw_state.input_mode {
                                InputMode::Normal => {
                                    self.state = AppState::Exit;
                                    return Ok(true);
                                }
                                InputMode::AddingItem { .. }
                                | InputMode::EditingItem { .. }
                                | InputMode::ConfirmDelete { .. } => {
                                    // Cancel adding/editing/confirming
                                    self.draw_state.input_mode = InputMode::Normal;
                                }
                            }
                        }
                    }
                }
                KeyCode::Char('q') => {
                    self.state = AppState::Exit;
                    return Ok(true);
                }
                KeyCode::Char('?') => {
                    if matches!(self.draw_state.screen, Screen::Main | Screen::Welcome)
                        && matches!(self.draw_state.input_mode, InputMode::Normal)
                    {
                        self.draw_state.previous_screen = self.draw_state.screen;
                        self.draw_state.screen = Screen::Help;
                    }
                }
                KeyCode::Enter => {
                    match self.draw_state.screen {
                        Screen::Welcome => {
                            // Enter on the file field opens its editor;
                            // a file path is required to continue
                            if self.draw_state.focus == Focus::TripFile
                                || self.draw_state.trip_file_path.as_os_str().is_empty()
                            {
                                self.draw_state.trip_file_input =
                                    self.draw_state.trip_file_path.display().to_string();
                                self.draw_state.editing_trip_file = true;
                            } else {
                                // Move to main screen after configuring
                                self.draw_state.screen = Screen::Main;
                                self.draw_state.focus_side = FocusSide::Left;
                                self.draw_state.selected_index = 0;
                            }
                        }
                        Screen::Help => {
                            self.draw_state.screen = self.draw_state.previous_screen;
                        }
                        Screen::Config => {}
                        Screen::ConfigEditor => {}
                        Screen::Main => {
                            match &self.draw_state.input_mode {
                                InputMode::Normal => {
                                    if self.draw_state.focus_side == FocusSide::Left {
                                        // Toggle selection on left side
                                        self.toggle_item_selection();
                                    } else {
                                        // Toggle packed status on right side
                                        self.toggle_item_packed_right_side();
                                    }
                                }
                                InputMode::AddingItem {
                                    name,
                                    category,
                                    quantity,
                                    category_index,
                                    ..
                                } => {
                                    // Get the selected category from the list
                                    let categories = self.get_all_categories();
                                    let selected_category = if *category_index < categories.len() {
                                        categories[*category_index].clone()
                                    } else if !categories.is_empty() {
                                        categories[0].clone()
                                    } else {
                                        category.clone()
                                    };
                                    // Save the new item
                                    self.save_new_item(
                                        name.clone(),
                                        selected_category,
                                        quantity.clone(),
                                    );
                                    self.draw_state.input_mode = InputMode::Normal;
                                }
                                InputMode::EditingItem {
                                    index,
                                    name,
                                    category,
                                    quantity,
                                    auto_quantity,
                                    category_index,
                                    selected_categories,
                                    ..
                                } => {
                                    // The membership set from the dialog's
                                    // multi-select; fall back to the single
                                    // category when empty (never saved empty)
                                    let categories = if !selected_categories.is_empty() {
                                        selected_categories.clone()
                                    } else {
                                        let all = self.get_all_categories();
                                        if *category_index < all.len() {
                                            vec![all[*category_index].clone()]
                                        } else {
                                            vec![category.clone()]
                                        }
                                    };
                                    // Save the edited item
                                    self.save_edited_item(
                                        *index,
                                        name.clone(),
                                        categories,
                                        quantity.clone(),
                                        *auto_quantity,
                                    );
                                    self.draw_state.input_mode = InputMode::Normal;
                                }
                                InputMode::ConfirmDelete { index } => {
                                    // Confirm deletion
                                    self.remove_item_confirmed(*index);
                                    self.draw_state.input_mode = InputMode::Normal;
                                    // Save config after deletion
                                    self.save_config()?;
                                }
                            }
                        }
                    }
                }
                KeyCode::Tab => {
                    if self.draw_state.screen == Screen::Main {
                        match &mut self.draw_state.input_mode {
                            InputMode::Normal => {
                                // Switch between left and right panels
                                self.draw_state.focus_side = match self.draw_state.focus_side {
                                    FocusSide::Left => FocusSide::Right,
                                    FocusSide::Right => FocusSide::Left,
                                };
                                // Reset selected index for the new side
                                self.draw_state.selected_index = 0;
                                // Mirror the right cursor's category on the left panel
                                if self.draw_state.focus_side == FocusSide::Right {
                                    self.sync_left_view_to_right_cursor();
                                }
                            }
                            InputMode::AddingItem { field, .. } => {
                                // Cycle through fields
                                *field = match field {
                                    InputField::Name => InputField::Category,
                                    InputField::Category => InputField::Quantity,
                                    InputField::Quantity => InputField::Mode,
                                    InputField::Mode => InputField::Name,
                                };
                            }
                            InputMode::EditingItem { field, .. } => {
                                // Cycle through fields
                                *field = match field {
                                    InputField::Name => InputField::Category,
                                    InputField::Category => InputField::Quantity,
                                    InputField::Quantity => InputField::Mode,
                                    InputField::Mode => InputField::Name,
                                };
                            }
                            _ => {}
                        }
                    } else if self.draw_state.screen == Screen::Welcome {
                        // In welcome screen, Tab cycles through fields
                        self.draw_state.focus = self.draw_state.focus.next();
                    }
                }
                KeyCode::BackTab => {
                    if self.draw_state.screen == Screen::Main
                        && matches!(self.draw_state.input_mode, InputMode::Normal)
                    {
                        self.draw_state.focus_side = match self.draw_state.focus_side {
                            FocusSide::Left => FocusSide::Right,
                            FocusSide::Right => FocusSide::Left,
                        };
                        self.draw_state.selected_index = 0;
                        // Mirror the right cursor's category on the left panel
                        if self.draw_state.focus_side == FocusSide::Right {
                            self.sync_left_view_to_right_cursor();
                        }
                    } else if self.draw_state.screen == Screen::Welcome {
                        self.draw_state.focus = self.draw_state.focus.prev();
                    }
                }
                KeyCode::Char('c') | KeyCode::Char('C') => {
                    if self.draw_state.screen == Screen::Main
                        && matches!(self.draw_state.input_mode, InputMode::Normal)
                        && self.draw_state.focus_side == FocusSide::Left
                    {
                        self.toggle_category_collapse();
                    }
                }
                KeyCode::Char(' ') => {
                    if self.draw_state.screen == Screen::Main
                        && matches!(self.draw_state.input_mode, InputMode::Normal)
                    {
                        if self.draw_state.focus_side == FocusSide::Left {
                            // Toggle selection on left side
                            self.toggle_item_selection();
                        } else {
                            // Toggle packed status on right side
                            self.toggle_item_packed_right_side();
                        }
                    } else if self.draw_state.screen == Screen::Main {
                        // Handle space in edit mode for category selection
                        if let InputMode::EditingItem { field, .. } =
                            &mut self.draw_state.input_mode
                            && *field == InputField::Category
                        {
                            // Space toggles category selection in edit mode
                            // This is handled by the field cycling
                        }
                    }
                }
                KeyCode::Char('a') | KeyCode::Char('A') => {
                    if self.draw_state.screen == Screen::Main
                        && matches!(self.draw_state.input_mode, InputMode::Normal)
                        && self.draw_state.focus_side == FocusSide::Left
                    {
                        // Start adding new item with the current category preselected
                        let categories = self.get_all_categories();
                        let category_index = self
                            .current_category()
                            .and_then(|c| categories.iter().position(|k| k == &c))
                            .unwrap_or(0);
                        self.draw_state.input_mode = InputMode::AddingItem {
                            field: InputField::Name,
                            name: String::new(),
                            category: String::new(),
                            quantity: String::new(),
                            category_list: categories,
                            category_index,
                        };
                    }
                }
                KeyCode::Char('e') | KeyCode::Char('E') => {
                    if self.draw_state.screen == Screen::Main
                        && matches!(self.draw_state.input_mode, InputMode::Normal)
                        && self.draw_state.focus_side == FocusSide::Left
                    {
                        // Start editing selected item
                        self.start_editing_item();
                    }
                }
                KeyCode::Char('d') | KeyCode::Char('D') => {
                    if self.draw_state.screen == Screen::Main
                        && matches!(self.draw_state.input_mode, InputMode::Normal)
                        && self.draw_state.focus_side == FocusSide::Left
                    {
                        // Ask for confirmation before deleting
                        if let Some(item_index) = self.current_item_index() {
                            self.draw_state.input_mode =
                                InputMode::ConfirmDelete { index: item_index };
                        }
                    }
                }
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    // Recalculate the current list's quantities (spec 018):
                    // works from both list views, unbound everywhere else
                    if self.draw_state.screen == Screen::Main
                        && matches!(self.draw_state.input_mode, InputMode::Normal)
                    {
                        self.update_quantities()?;
                    }
                }
                KeyCode::Up => {
                    self.handle_up()?;
                }
                KeyCode::Down => {
                    self.handle_down()?;
                }
                KeyCode::Char('f') | KeyCode::Char('F') => {
                    if self.draw_state.screen == Screen::Welcome {
                        // Edit the trip file path
                        self.draw_state.trip_file_input =
                            self.draw_state.trip_file_path.display().to_string();
                        self.draw_state.editing_trip_file = true;
                    }
                }
                KeyCode::Char('o') | KeyCode::Char('O') => {
                    // Open the config screen from any screen
                    if !matches!(
                        self.draw_state.screen,
                        Screen::Config | Screen::ConfigEditor
                    ) {
                        self.draw_state.previous_screen = self.draw_state.screen;
                        self.draw_state.screen = Screen::Config;
                    }
                }
                KeyCode::Char('+') => {
                    if self.draw_state.screen == Screen::Welcome
                        && self.draw_state.focus == Focus::Weekdays
                        && let Ok(new_weekdays) = Weekdays::new(self.draw_state.weekdays.get() + 1)
                    {
                        self.draw_state.weekdays = new_weekdays;
                        self.update_quantities()?;
                    }
                }
                KeyCode::Char('-') => {
                    if self.draw_state.screen == Screen::Welcome
                        && self.draw_state.focus == Focus::Weekdays
                        && self.draw_state.weekdays.get() > Weekdays::MIN
                        && let Ok(new_weekdays) = Weekdays::new(self.draw_state.weekdays.get() - 1)
                    {
                        self.draw_state.weekdays = new_weekdays;
                        self.update_quantities()?;
                    }
                }
                // Handle input mode for adding/editing item
                _ => {
                    if self.draw_state.screen == Screen::Main {
                        self.handle_add_edit_input(key);
                    }
                }
            }
        }
        Ok(false)
    }
    fn handle_up(&mut self) -> Result<()> {
        self.draw_state.move_cursor_up();
        // Keep the left panel in sync when navigating the final list
        if self.draw_state.screen == Screen::Main
            && matches!(self.draw_state.input_mode, InputMode::Normal)
            && self.draw_state.focus_side == FocusSide::Right
        {
            self.sync_left_view_to_right_cursor();
        }
        Ok(())
    }

    fn handle_down(&mut self) -> Result<()> {
        self.draw_state.move_cursor_down();
        // Keep the left panel in sync when navigating the final list
        if self.draw_state.screen == Screen::Main
            && matches!(self.draw_state.input_mode, InputMode::Normal)
            && self.draw_state.focus_side == FocusSide::Right
        {
            self.sync_left_view_to_right_cursor();
        }
        Ok(())
    }

    // The modifiers of the focused config section
    fn config_section_modifiers(
        &self,
        section: ConfigSection,
    ) -> &Vec<crate::config::modifiers::ModifierConfig> {
        match section {
            ConfigSection::TravelStyles => &self.draw_state.modifiers.travel_styles,
            ConfigSection::Weather => &self.draw_state.modifiers.weather,
        }
    }

    fn config_section_modifiers_mut(
        &mut self,
        section: ConfigSection,
    ) -> &mut Vec<crate::config::modifiers::ModifierConfig> {
        match section {
            ConfigSection::TravelStyles => &mut self.draw_state.modifiers.travel_styles,
            ConfigSection::Weather => &mut self.draw_state.modifiers.weather,
        }
    }

    /// Open the raw config file editor, loading the current file content
    fn open_config_editor(&mut self) -> Result<()> {
        if self.draw_state.screen == Screen::ConfigEditor {
            return Ok(());
        }
        let text = fs::read_to_string(&self.config_path).context("Failed to read config file")?;
        self.draw_state.config_editor_text = text.lines().map(|l| l.to_string()).collect();
        if self.draw_state.config_editor_text.is_empty() {
            self.draw_state.config_editor_text.push(String::new());
        }
        self.draw_state.config_editor_cursor = (0, 0);
        self.draw_state.config_editor_scroll = 0;
        self.draw_state.config_editor_error = None;
        self.draw_state.previous_screen = self.draw_state.screen;
        self.draw_state.screen = Screen::ConfigEditor;
        Ok(())
    }

    /// Key handling of the raw config editor screen
    fn handle_config_editor_input(&mut self, key: event::KeyEvent) -> Result<bool> {
        let (line, col) = self.draw_state.config_editor_cursor;

        match key.code {
            KeyCode::Esc => {
                // Discard edits and return
                self.draw_state.screen = self.draw_state.previous_screen;
            }
            KeyCode::Enter => {
                // Split the current line at the cursor and insert a new line
                let lines = &mut self.draw_state.config_editor_text;
                if line < lines.len() {
                    let (head, tail) = lines[line].split_at(col.min(lines[line].len()));
                    let head = head.to_string();
                    let tail = tail.to_string();
                    lines[line] = head;
                    lines.insert(line + 1, tail);
                    self.draw_state.config_editor_cursor = (line + 1, 0);
                }
            }
            KeyCode::Backspace => {
                let lines = &mut self.draw_state.config_editor_text;
                if col > 0 {
                    lines[line].remove(col - 1);
                    self.draw_state.config_editor_cursor = (line, col - 1);
                } else if line > 0 {
                    // Join with the previous line
                    let current = lines.remove(line);
                    let prev_len = lines[line - 1].len();
                    lines[line - 1].push_str(&current);
                    self.draw_state.config_editor_cursor = (line - 1, prev_len);
                }
            }
            KeyCode::Up => {
                if line > 0 {
                    let col = col.min(self.draw_state.config_editor_text[line - 1].len());
                    self.draw_state.config_editor_cursor = (line - 1, col);
                }
            }
            KeyCode::Down => {
                if line + 1 < self.draw_state.config_editor_text.len() {
                    let col = col.min(self.draw_state.config_editor_text[line + 1].len());
                    self.draw_state.config_editor_cursor = (line + 1, col);
                }
            }
            KeyCode::Left => {
                if col > 0 {
                    self.draw_state.config_editor_cursor = (line, col - 1);
                } else if line > 0 {
                    let prev_len = self.draw_state.config_editor_text[line - 1].len();
                    self.draw_state.config_editor_cursor = (line - 1, prev_len);
                }
            }
            KeyCode::Right => {
                if col < self.draw_state.config_editor_text[line].len() {
                    self.draw_state.config_editor_cursor = (line, col + 1);
                } else if line + 1 < self.draw_state.config_editor_text.len() {
                    self.draw_state.config_editor_cursor = (line + 1, 0);
                }
            }
            KeyCode::Home => {
                self.draw_state.config_editor_cursor = (line, 0);
            }
            KeyCode::End => {
                let len = self.draw_state.config_editor_text[line].len();
                self.draw_state.config_editor_cursor = (line, len);
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Ctrl+S saves and applies the edited config
                self.save_config_editor()?;
            }
            KeyCode::Char(c) => {
                let lines = &mut self.draw_state.config_editor_text;
                let col = col.min(lines[line].len());
                lines[line].insert(col, c);
                self.draw_state.config_editor_cursor = (line, col + 1);
            }
            _ => {}
        }

        Ok(false)
    }

    /// Save the edited config text, reload it into the app state, and return
    /// to the previous screen. On a parse error the editor stays open and
    /// shows the error.
    fn save_config_editor(&mut self) -> Result<()> {
        let text = self.draw_state.config_editor_text.join("\n");

        // Validate by parsing before writing anything to disk
        if let Err(e) = serde_yaml::from_str::<AppConfig>(&text) {
            self.draw_state.config_editor_error = Some(format!("YAML error: {}", e));
            return Ok(());
        }

        if let Err(e) = fs::write(&self.config_path, &text) {
            self.draw_state.config_editor_error = Some(format!("Write error: {}", e));
            return Ok(());
        }

        // Snapshot the trip state (selection/packed) so reloading the input
        // config does not reset the current trip
        let trip_state: Vec<(String, bool, bool)> = self
            .draw_state
            .packing_list
            .items
            .iter()
            .map(|item| (item.name.clone(), item.editable, item.packed))
            .collect();

        // Reload the saved config into the app state
        let (packing_list, trip_params) = load_config(&self.config_path)?;
        self.draw_state.packing_list = packing_list;
        self.draw_state.weekdays = trip_params.weekdays;
        self.draw_state.travel_style = trip_params.travel_style;
        self.draw_state.weather = trip_params.weather;
        self.draw_state.modifiers = trip_params.modifiers;

        // Restore the trip state for items that still exist (by name)
        for item in &mut self.draw_state.packing_list.items {
            if let Some((_, editable, packed)) = trip_state
                .iter()
                .find(|(name, _, _)| name.eq_ignore_ascii_case(&item.name))
            {
                item.editable = *editable;
                item.packed = *packed;
            }
        }

        self.reconcile_selections();
        self.update_quantities()?;
        self.draw_state.config_editor_error = None;
        self.draw_state.screen = self.draw_state.previous_screen;
        Ok(())
    }

    fn handle_config_screen_input(&mut self, key: event::KeyEvent) -> Result<bool> {
        // Modifier editor is open: route all keys to it
        if let Some(editor) = self.draw_state.modifier_editor.as_mut() {
            match key.code {
                KeyCode::Esc => {
                    self.draw_state.modifier_editor = None;
                }
                KeyCode::Tab | KeyCode::BackTab => {
                    editor.field = match editor.field {
                        ModifierField::Name => ModifierField::Multiplier,
                        ModifierField::Multiplier => ModifierField::Name,
                    };
                }
                KeyCode::Enter => {
                    // Save the modifier
                    let editor = self.draw_state.modifier_editor.take().unwrap();
                    let name = editor.name.trim().to_string();
                    let multiplier: f32 = editor.multiplier.trim().parse().unwrap_or(1.0);
                    if !name.is_empty() {
                        let list = self.config_section_modifiers_mut(editor.section);
                        match editor.index {
                            Some(i) if i < list.len() => {
                                list[i].name = name;
                                list[i].multiplier = multiplier;
                            }
                            _ => {
                                list.push(crate::config::modifiers::ModifierConfig {
                                    name,
                                    multiplier,
                                });
                            }
                        }
                        // Keep the current selections valid
                        self.reconcile_selections();
                        self.save_config().ok();
                        self.update_quantities()?;
                    }
                }
                KeyCode::Backspace => match editor.field {
                    ModifierField::Name => {
                        editor.name.pop();
                    }
                    ModifierField::Multiplier => {
                        editor.multiplier.pop();
                    }
                },
                KeyCode::Char(c) => match editor.field {
                    ModifierField::Name => {
                        editor.name.push(c);
                    }
                    ModifierField::Multiplier => {
                        editor.multiplier.push(c);
                    }
                },
                _ => {}
            }
            return Ok(false);
        }

        match key.code {
            KeyCode::Esc => {
                // Leave the config screen, back to where it was opened from
                self.draw_state.screen = self.draw_state.previous_screen;
            }
            KeyCode::Tab => {
                self.draw_state.config_section = self.draw_state.config_section.next();
                self.draw_state.config_cursor = 0;
                self.draw_state.config_scroll = 0;
            }
            KeyCode::BackTab => {
                self.draw_state.config_section = self.draw_state.config_section.prev();
                self.draw_state.config_cursor = 0;
                self.draw_state.config_scroll = 0;
            }
            KeyCode::Up => {
                if self.draw_state.config_cursor > 0 {
                    self.draw_state.config_cursor -= 1;
                }
            }
            KeyCode::Down => {
                let count = self
                    .config_section_modifiers(self.draw_state.config_section)
                    .len();
                if self.draw_state.config_cursor + 1 < count {
                    self.draw_state.config_cursor += 1;
                }
            }
            KeyCode::Char('a') | KeyCode::Char('A') => {
                // Add a new modifier to the focused section
                self.draw_state.modifier_editor = Some(ModifierEditor {
                    index: None,
                    section: self.draw_state.config_section,
                    name: String::new(),
                    multiplier: String::from("1.0"),
                    field: ModifierField::Name,
                });
            }
            KeyCode::Char('e') | KeyCode::Char('E') => {
                // Edit the modifier under the cursor
                let section = self.draw_state.config_section;
                let cursor = self.draw_state.config_cursor;
                if let Some(m) = self.config_section_modifiers(section).get(cursor) {
                    self.draw_state.modifier_editor = Some(ModifierEditor {
                        index: Some(cursor),
                        section,
                        name: m.name.clone(),
                        multiplier: m.multiplier.to_string(),
                        field: ModifierField::Name,
                    });
                }
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                // Delete the modifier under the cursor
                let section = self.draw_state.config_section;
                let cursor = self.draw_state.config_cursor;
                let list = self.config_section_modifiers_mut(section);
                if cursor < list.len() {
                    list.remove(cursor);
                    let count = list.len();
                    if self.draw_state.config_cursor >= count {
                        self.draw_state.config_cursor = count.saturating_sub(1);
                    }
                    self.reconcile_selections();
                    self.save_config().ok();
                    self.update_quantities()?;
                }
            }
            _ => {}
        }
        Ok(false)
    }

    /// Keep the current travel style / weather / activity selections valid
    /// after modifiers change
    fn reconcile_selections(&mut self) {
        if self
            .draw_state
            .modifiers
            .travel_style_multiplier(&self.draw_state.travel_style)
            .is_none()
        {
            self.draw_state.travel_style = self
                .draw_state
                .modifiers
                .travel_styles
                .first()
                .map(|m| m.name.clone())
                .unwrap_or_default();
        }
        if self
            .draw_state
            .modifiers
            .weather_multiplier(&self.draw_state.weather)
            .is_none()
        {
            self.draw_state.weather = self
                .draw_state
                .modifiers
                .weather
                .first()
                .map(|m| m.name.clone())
                .unwrap_or_default();
        }
    }

    fn handle_add_edit_input(&mut self, key: event::KeyEvent) {
        // Left/Right cycle the category selection (uniform arrow model)
        if matches!(key.code, KeyCode::Left | KeyCode::Right) {
            let direction = if key.code == KeyCode::Right { 1 } else { -1 };
            self.draw_state.cycle_category_selection(direction);
            return;
        }
        match &mut self.draw_state.input_mode {
            InputMode::AddingItem {
                field,
                name,
                category,
                quantity,
                ..
            } => match key.code {
                KeyCode::Char(c) => match field {
                    InputField::Name => name.push(c),
                    InputField::Category => category.push(c),
                    InputField::Quantity => quantity.push(c),
                    InputField::Mode => {}
                },
                KeyCode::Backspace => match field {
                    InputField::Name => {
                        name.pop();
                    }
                    InputField::Category => {
                        category.pop();
                    }
                    InputField::Quantity => {
                        quantity.pop();
                    }
                    InputField::Mode => {}
                },
                _ => {}
            },
            InputMode::EditingItem {
                field,
                name,
                category,
                quantity,
                category_list,
                category_index,
                selected_categories,
                ..
            } => match key.code {
                KeyCode::Char(' ') => match field {
                    // Space toggles the quantity mode in the Mode field
                    InputField::Mode => self.draw_state.toggle_edit_mode(),
                    // Space toggles membership of the highlighted category
                    InputField::Category => {
                        if let Some(highlighted) = category_list.get(*category_index) {
                            let highlighted = highlighted.clone();
                            if let Some(pos) =
                                selected_categories.iter().position(|c| c == &highlighted)
                            {
                                // Keep at least one membership
                                if selected_categories.len() > 1 {
                                    selected_categories.remove(pos);
                                }
                            } else {
                                selected_categories.push(highlighted);
                            }
                        }
                    }
                    _ => {}
                },
                KeyCode::Char(c) => match field {
                    InputField::Name => name.push(c),
                    InputField::Category => category.push(c),
                    InputField::Quantity => quantity.push(c),
                    InputField::Mode => {}
                },
                KeyCode::Backspace => match field {
                    InputField::Name => {
                        name.pop();
                    }
                    InputField::Category => {
                        category.pop();
                    }
                    InputField::Quantity => {
                        quantity.pop();
                    }
                    InputField::Mode => {}
                },
                _ => {}
            },
            _ => {}
        }
    }

    // The item currently selected on the focused side, if the cursor is on an item row
    fn current_item_index(&self) -> Option<usize> {
        self.draw_state.cursor_item_index()
    }

    // The category the cursor is on (header row or one of its items)
    fn current_category(&self) -> Option<String> {
        self.draw_state.cursor_category()
    }

    fn toggle_category_collapse(&mut self) {
        self.draw_state.toggle_category_collapse();
    }

    fn toggle_item_selection(&mut self) {
        self.draw_state.toggle_item_selection();
    }

    fn toggle_category_selection(&mut self) {
        self.draw_state.toggle_category_selection();
    }

    fn toggle_item_packed_right_side(&mut self) {
        self.draw_state.toggle_item_packed();
    }

    /// After navigating on the right panel: collapse all categories on the
    /// left and expand the one the cursor's item belongs to, so the left
    /// panel mirrors the right panel's position.
    fn sync_left_view_to_right_cursor(&mut self) {
        let Some(item_index) = self.current_item_index() else {
            return;
        };
        let category = self.draw_state.packing_list.items[item_index]
            .primary_category()
            .to_string();

        // Collapse everything, then expand only the cursor's category
        let all_categories: Vec<String> = self
            .draw_state
            .packing_list
            .items
            .iter()
            .flat_map(|item| item.categories.iter().cloned())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        self.draw_state.collapsed_categories = all_categories;
        self.draw_state
            .collapsed_categories
            .retain(|c| c != &category);
    }

    fn start_editing_item(&mut self) {
        if let Some(item_index) = self.current_item_index() {
            let item = &self.draw_state.packing_list.items[item_index];
            let categories = self.get_all_categories();
            let category_index = categories
                .iter()
                .position(|c| c == item.primary_category())
                .unwrap_or(0);

            self.draw_state.input_mode = InputMode::EditingItem {
                index: item_index,
                field: InputField::Name,
                name: item.name.clone(),
                category: item.primary_category().to_string(),
                quantity: item.quantity.to_string(),
                auto_quantity: item.auto_quantity,
                category_list: categories,
                category_index,
                selected_categories: item.categories.clone(),
            };
        }
    }

    fn save_new_item(&mut self, name: String, category: String, quantity: String) {
        // Parse quantity
        let qty = quantity.parse::<u32>().unwrap_or(1);

        // Create new item
        let new_id = format!("item_{}", self.draw_state.packing_list.items.len() + 1);
        let new_name = if name.is_empty() {
            "New Item".to_string()
        } else {
            name
        };
        let new_category = if category.is_empty() {
            // Use first category or default
            if !self.draw_state.packing_list.items.is_empty() {
                self.draw_state.packing_list.items[0]
                    .primary_category()
                    .to_string()
            } else {
                "Other".to_string()
            }
        } else {
            category
        };

        self.draw_state
            .packing_list
            .add_item(Item::new(new_id, new_name, vec![new_category], qty));

        // Update quantities for the new item
        self.update_quantities().ok();

        // Save config after adding
        self.save_config().ok();

        // Reset selection to the new item, clamped to the visible rows
        let rows = self.draw_state.active_rows();
        self.draw_state.selected_index =
            (self.draw_state.packing_list.items.len() - 1).min(rows.len().saturating_sub(1));
    }

    fn save_edited_item(
        &mut self,
        index: usize,
        name: String,
        categories: Vec<String>,
        quantity: String,
        auto_quantity: bool,
    ) {
        let was_auto = index < self.draw_state.packing_list.items.len()
            && self.draw_state.packing_list.items[index].auto_quantity;
        self.draw_state
            .apply_item_edit(index, name, categories, quantity, Some(auto_quantity));
        // Save config after editing
        self.save_config().ok();
        // When the item stays automatic, recalculate so the quantity
        // reflects the trip parameters rather than a stale dialog value
        if was_auto && auto_quantity {
            self.update_quantities().ok();
        }
    }

    fn remove_item_confirmed(&mut self, index: usize) {
        if index < self.draw_state.packing_list.items.len() {
            self.draw_state.packing_list.items.remove(index);

            // Adjust selection index to stay within the visible rows
            let rows = self.draw_state.active_rows();
            if self.draw_state.selected_index >= rows.len() {
                self.draw_state.selected_index = rows.len().saturating_sub(1);
            }
        }
    }
}

/// Trip parameters loaded from (or defaulted into) the config file
struct TripParams {
    weekdays: Weekdays,
    travel_style: String,
    weather: String,
    modifiers: ModifiersConfig,
    last_trip_file: Option<String>,
}

fn load_config(config_file: &PathBuf) -> Result<(PackingList, TripParams)> {
    let yaml_content = fs::read_to_string(config_file).context("Failed to read config file")?;

    let config: AppConfig =
        serde_yaml::from_str(&yaml_content).context("Failed to parse config file")?;

    // Create packing list from config
    let mut packing_list = PackingList::new("My Trip".to_string());

    for saved_item in config.items {
        let new_id = format!("item_{}", packing_list.items.len() + 1);
        let auto = saved_item.auto_quantity;
        let fixed_quantity = if auto {
            1
        } else {
            saved_item.quantity.unwrap_or(1)
        };
        let mut item = Item::new(
            new_id,
            saved_item.name,
            saved_item.categories,
            fixed_quantity,
        );
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

fn create_default_config(config_file: &PathBuf) -> Result<(PackingList, TripParams)> {
    // Create default packing list
    let mut packing_list = PackingList::new("My Trip".to_string());

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
        let new_id = format!("item_{}", packing_list.items.len() + 1);
        packing_list.add_item(Item::new(
            new_id,
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

// Capitalize the first character of a value for display
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// Select every item of a fresh or empty packing list so a new trip starts
/// fully pre-selected
fn select_all_items(packing_list: &mut PackingList) {
    for item in &mut packing_list.items {
        item.editable = true;
        item.packed = false;
    }
}

/// Resolve a user-entered trip file path:
/// - `~` and `~/...` expand to the home directory
/// - relative paths resolve against the current working directory
/// - absolute paths are kept as-is
fn resolve_trip_path(input: &str) -> PathBuf {
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

fn draw_welcome_screen(f: &mut ratatui::prelude::Frame, size: Rect, state: &AppDrawState) {
    let [title_area, content_area, instructions_area] = *Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(12),
            Constraint::Length(4),
        ])
        .split(size)
    else {
        return;
    };

    // Title
    let title = Paragraph::new("Welcome to Pack-Liste")
        .style(Style::new().bold().fg(Color::Green))
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(title, title_area);

    // Content - trip parameters
    let params_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(3),
        ])
        .split(content_area);

    // Inner area of a bordered block (one cell inside each border)
    let inner = |area: Rect| -> Rect {
        Rect {
            x: area.x + 1,
            y: area.y + 1,
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(2),
        }
    };

    // Trip file path (persist target for the compiled packing list, required)
    let file_block = Block::default()
        .title("Packing List File (Required)")
        .borders(Borders::ALL)
        .border_style(
            if state.focus == Focus::TripFile || state.editing_trip_file {
                Style::new().fg(Color::Yellow)
            } else {
                Style::new()
            },
        );

    let file_text = if state.editing_trip_file {
        vec![
            Line::from(format!(" {}_", state.trip_file_input)),
            Line::from(" Enter to confirm | Esc to cancel".dim()),
        ]
    } else {
        let shown = if state.trip_file_path.as_os_str().is_empty() {
            "(required - set a path to persist your packing list)".dim()
        } else {
            state.trip_file_path.display().to_string().into()
        };
        vec![
            Line::from(vec![Span::raw(" "), shown]),
            Line::from(" Enter to edit (relative paths resolve against the current folder)".dim()),
        ]
    };
    let file_para = Paragraph::new(file_text);
    f.render_widget(file_block, params_layout[0]);
    f.render_widget(file_para, inner(params_layout[0]));

    // Weekdays
    let weekdays_block = Block::default()
        .title("Trip Duration (Weekdays)")
        .borders(Borders::ALL)
        .border_style(if state.focus == Focus::Weekdays {
            Style::new().fg(Color::Yellow)
        } else {
            Style::new()
        });

    let weekdays_text = vec![
        Line::from(format!(" {} Days", state.weekdays.get())),
        Line::from(" Left/Right or +/- to adjust".dim()),
    ];
    let weekdays_para = Paragraph::new(weekdays_text);
    f.render_widget(weekdays_block, params_layout[1]);
    f.render_widget(weekdays_para, inner(params_layout[1]));

    // Travel Style
    let style_block = Block::default()
        .title("Travel Style")
        .borders(Borders::ALL)
        .border_style(if state.focus == Focus::TravelStyle {
            Style::new().fg(Color::Yellow)
        } else {
            Style::new()
        });

    let style_text = vec![
        Line::from(format!(" {}", capitalize_first(&state.travel_style))),
        Line::from(" Left/Right to cycle through options".dim()),
    ];
    let style_para = Paragraph::new(style_text);
    f.render_widget(style_block, params_layout[2]);
    f.render_widget(style_para, inner(params_layout[2]));

    // Weather
    let weather_block = Block::default()
        .title("Weather")
        .borders(Borders::ALL)
        .border_style(if state.focus == Focus::Weather {
            Style::new().fg(Color::Yellow)
        } else {
            Style::new()
        });

    let weather_text = vec![
        Line::from(format!(" {}", capitalize_first(&state.weather))),
        Line::from(" Left/Right to cycle through options".dim()),
    ];
    let weather_para = Paragraph::new(weather_text);
    f.render_widget(weather_block, params_layout[3]);
    f.render_widget(weather_para, inner(params_layout[3]));

    // Instructions
    let instructions =
        Paragraph::new(hint_line(hint_entries(HintSurface::Welcome))).wrap(Wrap { trim: true });
    f.render_widget(instructions, instructions_area);

    // Resume prompt for the remembered trip file
    if state.resume_prompt {
        let popup_width = 60.min(size.width.saturating_sub(4));
        let popup_height = 6.min(size.height);
        let popup_area = Rect {
            x: size.x + (size.width.saturating_sub(popup_width)) / 2,
            y: size.y + (size.height.saturating_sub(popup_height)) / 2,
            width: popup_width,
            height: popup_height,
        };

        let block = Block::default()
            .title("Resume Previous Trip?")
            .borders(Borders::ALL)
            .style(Style::new().bg(Color::Black).fg(Color::White));
        f.render_widget(block, popup_area);

        let inner = Rect {
            x: popup_area.x + 1,
            y: popup_area.y + 1,
            width: popup_area.width.saturating_sub(2),
            height: popup_area.height.saturating_sub(2),
        };

        let lines = vec![
            Line::from(vec![
                Span::raw("Load "),
                Span::raw(
                    state
                        .remembered_trip_file
                        .as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_default(),
                )
                .fg(Color::Cyan)
                .bold(),
                Span::raw("?"),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("Y/Enter").bold().fg(Color::Yellow),
                Span::raw(" resume | "),
                Span::raw("N/Esc").bold().fg(Color::Red),
                Span::raw(" start fresh"),
            ]),
        ];
        f.render_widget(Paragraph::new(lines), inner);
    }
}

fn draw_main_screen(f: &mut ratatui::prelude::Frame, size: Rect, state: &mut AppDrawState) {
    // Split screen: left (all items), right (selected items only)
    let [left_area, right_area] = *Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(size)
    else {
        return;
    };

    // Draw left panel - all items with selection toggle
    draw_all_items_panel(f, left_area, state);

    // Draw right panel - only selected items
    draw_selected_items_panel(f, right_area, state);
}

/// The surfaces that carry a navigation hint line.
#[derive(Debug, Clone, Copy, PartialEq)]
enum HintSurface {
    LeftPanel,
    RightPanel,
    Welcome,
    Config,
    AddEditPopup,
    DeletePopup,
    ModifierEditor,
    RawEditor,
}

/// The hint entries for a surface: (key, action) pairs rendered by
/// `hint_line`. Single source for rendering and the extraction test.
fn hint_entries(surface: HintSurface) -> &'static [(&'static str, &'static str)] {
    match surface {
        HintSurface::LeftPanel => &[
            ("↑/↓", "Navigate"),
            ("Space", "Select"),
            ("Ctrl+Space", "Category"),
            ("A", "Add"),
            ("E", "Edit"),
            ("D", "Delete"),
            ("C", "Collapse"),
            ("R", "Recalculate"),
            ("Tab", "Switch"),
            ("?", "Help"),
        ],
        HintSurface::RightPanel => &[
            ("↑/↓", "Navigate"),
            ("Space", "Packed"),
            ("R", "Recalculate"),
            ("Tab", "Switch"),
            ("?", "Help"),
        ],
        HintSurface::Welcome => &[
            ("↑/↓", "Navigate fields"),
            ("←/→", "Adjust values"),
            ("+/-", "Weekdays"),
            ("Enter", "Edit file / Continue"),
            ("F", "Edit path"),
            ("O", "Config"),
            ("Ctrl+E", "Raw editor"),
            ("q", "Quit"),
        ],
        HintSurface::Config => &[
            ("Tab", "Section"),
            ("↑/↓", "Move"),
            ("A", "Add"),
            ("E", "Edit"),
            ("D", "Delete"),
            ("Ctrl+E", "Raw editor"),
            ("Esc", "Back"),
        ],
        HintSurface::AddEditPopup => &[
            ("Tab", "Cycle fields"),
            ("↑/↓", "Category dropdown"),
            ("←/→", "Cycle category"),
            ("Space", "Mode / Category"),
            ("Enter", "Save"),
            ("Esc", "Cancel"),
        ],
        HintSurface::DeletePopup => &[("Enter", "Confirm"), ("Esc", "Cancel")],
        HintSurface::ModifierEditor => &[
            ("Tab", "Switch field"),
            ("Enter", "Save"),
            ("Esc", "Cancel"),
        ],
        HintSurface::RawEditor => &[
            ("↑/↓/←/→", "Cursor"),
            ("Home/End", "Line start/end"),
            ("Enter", "New line"),
            ("Backspace", "Delete"),
            ("Ctrl+S", "Save & apply"),
            ("Esc", "Cancel"),
        ],
    }
}

/// The single hint-line constructor: keys in the highlight color and
/// bold, actions plain, " | " between entries.
fn hint_line(entries: &[(&'static str, &'static str)]) -> Line<'static> {
    let mut spans = Vec::new();
    for (i, (key, action)) in entries.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw(" | "));
        }
        spans.push(Span::raw(*key).bold().fg(Color::Yellow));
        spans.push(Span::raw(" "));
        spans.push(Span::raw(*action));
    }
    Line::from(spans)
}

/// The rendered width of one hint entry ("KEY Action").
fn hint_entry_width(key: &str, action: &str) -> usize {
    key.chars().count() + 1 + action.chars().count()
}

/// Split a surface's hint entries into rows that each fit `width`
/// columns, packing entries greedily and breaking only at entry
/// boundaries so no binding is ever cut off. Every entry appears
/// exactly once across the rows.
fn hint_rows(entries: &[(&'static str, &'static str)], width: usize) -> Vec<Line<'static>> {
    let mut rows = Vec::new();
    let mut current: Vec<(&'static str, &'static str)> = Vec::new();
    let mut current_width = 0;
    for entry in entries {
        let entry_width = hint_entry_width(entry.0, entry.1);
        let separator = if current.is_empty() { 0 } else { 3 };
        if !current.is_empty() && current_width + separator + entry_width > width {
            rows.push(hint_line(&current));
            current = Vec::new();
            current_width = 0;
        }
        current_width += if current.is_empty() { 0 } else { 3 };
        current_width += entry_width;
        current.push(*entry);
    }
    if !current.is_empty() {
        rows.push(hint_line(&current));
    }
    if rows.is_empty() {
        // An empty entry list still renders one (blank) hint line so
        // the footer area is never zero-height.
        rows.push(Line::from(""));
    }
    rows
}

// The binding tokens the help screen must list; the extraction test
// compares this against the handler bindings (SC-004).
#[cfg(test)]
fn help_binding_tokens() -> Vec<&'static str> {
    vec![
        // Global
        "Ctrl+C",
        "Ctrl+E",
        "q",
        "?",
        "O",
        // Welcome
        "↑/↓",
        "←/→",
        "+/-",
        "Space",
        "Enter",
        "F",
        "Esc",
        // Resume prompt
        "Y",
        "N",
        // Main screen
        "Tab",
        "BackTab",
        "Ctrl+Space",
        "C",
        "A",
        "E",
        "D",
        "R",
        // Add/Edit dialog
        "Mode",
        // Config screen
        // (Tab/BackTab/↑/↓/A/E/D/Esc shared with above tokens)
        // Modifier editor
        // (Tab/Enter/Esc shared with above tokens)
        // Raw config editor
        "↑/↓/←/→",
        "Home/End",
        "Backspace",
        "Ctrl+S",
    ]
}

// The help screen's content as plain text, for the completeness test.
#[cfg(test)]
fn help_screen_text() -> String {
    let help_text = help_content();
    let mut text = String::new();
    for line in &help_text {
        for span in &line.spans {
            text.push_str(&span.content);
        }
        text.push('\n');
    }
    text
}

// The help screen's content, shared by the renderer and the
// completeness test.
fn help_content() -> Vec<Line<'static>> {
    fn key_span(k: &'static str) -> Span<'static> {
        Span::raw(k).bold().fg(Color::Yellow)
    }
    fn quit_key_span(k: &'static str) -> Span<'static> {
        Span::raw(k).bold().fg(Color::Red)
    }
    fn entry(k: &'static str, d: &'static str) -> Line<'static> {
        Line::from(vec![key_span(k), Span::raw(" - "), Span::raw(d)])
    }
    fn quit_entry(k: &'static str, d: &'static str) -> Line<'static> {
        Line::from(vec![quit_key_span(k), Span::raw(" - "), Span::raw(d)])
    }
    fn section(t: &'static str, c: Color) -> Line<'static> {
        Line::from(Span::raw(t).bold().fg(c))
    }
    fn blank() -> Line<'static> {
        Line::from("")
    }

    vec![
        section("GLOBAL (any screen):", Color::Yellow),
        entry("Ctrl+C", "Quit immediately (saves config and trip file)"),
        entry("Ctrl+E", "Open the raw config file editor"),
        quit_entry("q", "Quit (saves config and trip file)"),
        entry("?", "Show this help (Main and Welcome screens)"),
        entry("O", "Open the configuration screen (modifiers)"),
        blank(),
        section("WELCOME SCREEN:", Color::Yellow),
        entry(
            "↑/↓",
            "Move focus between fields (file, weekdays, style, weather)",
        ),
        entry(
            "←/→",
            "Adjust the focused value (weekdays, cycle style/weather)",
        ),
        entry("+/-", "Adjust weekdays"),
        entry(
            "Enter",
            "Edit file name (on file field) / Continue to main screen",
        ),
        entry("F", "Edit the trip file path"),
        quit_entry("Esc", "Quit"),
        blank(),
        section("RESUME PROMPT:", Color::Yellow),
        entry("Y / Enter", "Resume the previous trip file"),
        entry("N / Esc", "Start a fresh list"),
        blank(),
        section("MAIN SCREEN (normal mode):", Color::Green),
        entry("↑/↓", "Move the cursor through the focused panel's rows"),
        entry("Tab / BackTab", "Switch focus between panels"),
        entry("Space", "Toggle selection (left) / toggle packed (right)"),
        entry("Ctrl+Space", "Toggle the category's selection (left panel)"),
        entry(
            "C",
            "Toggle collapse/expand of the category under the cursor",
        ),
        entry("A", "Add a new item"),
        entry("E", "Edit the item under the cursor"),
        entry("D", "Delete the item under the cursor (with confirmation)"),
        entry("R", "Recalculate quantities on the current list"),
        quit_entry("Esc", "Quit"),
        blank(),
        section("ADD/EDIT ITEM DIALOG:", Color::Yellow),
        entry(
            "Tab",
            "Cycle fields: Name -> Category -> Quantity -> Mode -> Name",
        ),
        entry(
            "↑/↓",
            "Move the category dropdown highlight (category field)",
        ),
        entry("←/→", "Cycle the category selection (category field)"),
        entry("Space", "Toggle the quantity mode (Mode field)"),
        entry("Enter", "Save"),
        quit_entry("Esc", "Cancel"),
        blank(),
        section("CONFIG SCREEN (modifiers):", Color::Yellow),
        entry("Tab / BackTab", "Switch section (travel styles / weather)"),
        entry("↑/↓", "Move the modifier cursor"),
        entry("A", "Add a modifier"),
        entry("E", "Edit the modifier under the cursor"),
        entry("D", "Delete the modifier under the cursor"),
        quit_entry(
            "Esc",
            "Close the modifier editor if open; otherwise leave the config screen",
        ),
        blank(),
        section("MODIFIER EDITOR POPUP:", Color::Yellow),
        entry("Tab", "Switch between name and multiplier field"),
        entry("Enter", "Save the modifier"),
        quit_entry("Esc", "Cancel the editor"),
        blank(),
        section("RAW CONFIG EDITOR (Ctrl+E):", Color::Yellow),
        entry("↑/↓/←/→", "Move the text cursor"),
        entry("Home/End", "Cursor to line start / end"),
        entry("Enter", "Insert a new line"),
        entry("Backspace", "Delete backward"),
        entry("Ctrl+S", "Save and apply (validates YAML first)"),
        quit_entry("Esc", "Discard edits and return"),
    ]
}

fn draw_help_screen(f: &mut ratatui::prelude::Frame, size: Rect) {
    let block = Block::default()
        .title("Help - Navigation & Controls")
        .borders(Borders::ALL)
        .style(Style::new().bg(Color::Black).fg(Color::White));
    f.render_widget(block, size);

    let inner = Rect {
        x: size.x + 1,
        y: size.y + 1,
        width: size.width - 2,
        height: size.height - 2,
    };

    let help_text = help_content();
    let help_para = Paragraph::new(help_text).wrap(Wrap { trim: true });
    f.render_widget(help_para, inner);
}

fn draw_config_screen(f: &mut ratatui::prelude::Frame, size: Rect, state: &AppDrawState) {
    let [title_area, content_area, instructions_area] = *Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .split(size)
    else {
        return;
    };

    // Title
    let title =
        Paragraph::new("Configuration - Modifiers").style(Style::new().bold().fg(Color::Green));
    f.render_widget(title, title_area);

    // Modifier editor popup takes precedence
    if let Some(editor) = &state.modifier_editor {
        let popup_width = 50.min(size.width.saturating_sub(4));
        let popup_area = Rect {
            x: size.x + (size.width.saturating_sub(popup_width)) / 2,
            y: size.y + 5,
            width: popup_width,
            height: 7,
        };

        let block = Block::default()
            .title(if editor.index.is_some() {
                "Edit Modifier"
            } else {
                "Add Modifier"
            })
            .borders(Borders::ALL)
            .style(Style::new().bg(Color::Black).fg(Color::White));
        f.render_widget(block, popup_area);

        let inner = Rect {
            x: popup_area.x + 1,
            y: popup_area.y + 1,
            width: popup_area.width.saturating_sub(2),
            height: popup_area.height.saturating_sub(2),
        };

        let field_style = Style::new().fg(Color::Yellow).bold();
        let normal_style = Style::new().fg(Color::White);
        let name_line = Line::from(vec![
            Span::raw("Name: ").style(if editor.field == ModifierField::Name {
                field_style
            } else {
                normal_style
            }),
            Span::raw(&editor.name),
        ]);
        let multiplier_line = Line::from(vec![
            Span::raw("Multiplier: ").style(if editor.field == ModifierField::Multiplier {
                field_style
            } else {
                normal_style
            }),
            Span::raw(&editor.multiplier),
        ]);
        let hint = hint_line(hint_entries(HintSurface::ModifierEditor));

        let para = Paragraph::new(vec![name_line, multiplier_line, hint]);
        f.render_widget(para, inner);
        return;
    }

    // Section tabs
    let tabs = Line::from(vec![
        Span::raw(if state.config_section == ConfigSection::TravelStyles {
            "[Travel Styles]"
        } else {
            " Travel Styles "
        })
        .style(if state.config_section == ConfigSection::TravelStyles {
            Style::new().fg(Color::Yellow).bold()
        } else {
            Style::new().fg(Color::DarkGray)
        }),
        Span::raw(" | "),
        Span::raw(if state.config_section == ConfigSection::Weather {
            "[Weather]"
        } else {
            " Weather "
        })
        .style(if state.config_section == ConfigSection::Weather {
            Style::new().fg(Color::Yellow).bold()
        } else {
            Style::new().fg(Color::DarkGray)
        }),
    ]);
    f.render_widget(
        Paragraph::new(tabs),
        Rect {
            x: content_area.x,
            y: content_area.y,
            width: content_area.width,
            height: 1,
        },
    );

    // Modifier list of the focused section
    let list_area = Rect {
        x: content_area.x,
        y: content_area.y + 2,
        width: content_area.width,
        height: content_area.height.saturating_sub(2),
    };

    let modifiers = match state.config_section {
        ConfigSection::TravelStyles => &state.modifiers.travel_styles,
        ConfigSection::Weather => &state.modifiers.weather,
    };

    // Window the list so the cursor stays visible
    let viewport_height = list_area.height as usize;
    let mut scroll = state
        .config_scroll
        .min(modifiers.len().saturating_sub(viewport_height));
    if state.config_cursor >= scroll + viewport_height {
        scroll = state.config_cursor + 1 - viewport_height;
    } else if state.config_cursor < scroll {
        scroll = state.config_cursor;
    }

    let lines: Vec<Line> = modifiers
        .iter()
        .enumerate()
        .skip(scroll)
        .take(viewport_height)
        .map(|(i, m)| {
            let is_cursor = i == state.config_cursor;
            let mut line = Line::from(vec![
                Span::raw("  "),
                Span::raw(format!("{:<20}", m.name)),
                Span::raw(format!(" x{:.2}", m.multiplier)),
            ]);
            if is_cursor {
                line = line.style(Style::new().fg(Color::Yellow).bg(Color::DarkGray));
            }
            line
        })
        .collect();

    let list_block = Block::default()
        .title(state.config_section.title())
        .borders(Borders::ALL);
    f.render_widget(list_block, list_area);
    f.render_widget(
        Paragraph::new(lines),
        Rect {
            x: list_area.x + 1,
            y: list_area.y + 1,
            width: list_area.width.saturating_sub(2),
            height: list_area.height.saturating_sub(2),
        },
    );

    // Instructions
    let instructions =
        Paragraph::new(hint_line(hint_entries(HintSurface::Config))).wrap(Wrap { trim: true });
    f.render_widget(instructions, instructions_area);
}

fn draw_config_editor_screen(
    f: &mut ratatui::prelude::Frame,
    size: Rect,
    state: &mut AppDrawState,
) {
    let [title_area, editor_area, footer_area] = *Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(5),
            Constraint::Length(2),
        ])
        .split(size)
    else {
        return;
    };

    let title = Paragraph::new(format!("Edit Config - {}", "config.yaml"))
        .style(Style::new().bold().fg(Color::Green));
    f.render_widget(title, title_area);

    let block = Block::default()
        .title("config.yaml (YAML)")
        .borders(Borders::ALL);
    f.render_widget(block, editor_area);

    let inner = Rect {
        x: editor_area.x + 1,
        y: editor_area.y + 1,
        width: editor_area.width.saturating_sub(2),
        height: editor_area.height.saturating_sub(2),
    };

    let (cursor_line, cursor_col) = state.config_editor_cursor;
    let viewport_height = inner.height as usize;

    // Keep the cursor line inside the viewport
    let mut scroll = state.config_editor_scroll;
    if cursor_line < scroll {
        scroll = cursor_line;
    } else if cursor_line >= scroll + viewport_height {
        scroll = cursor_line + 1 - viewport_height;
    }
    scroll = scroll.min(
        state
            .config_editor_text
            .len()
            .saturating_sub(viewport_height),
    );
    state.config_editor_scroll = scroll;

    let lines: Vec<Line> = state
        .config_editor_text
        .iter()
        .enumerate()
        .skip(scroll)
        .take(viewport_height)
        .map(|(i, text)| {
            let is_cursor_line = i == cursor_line;
            let mut line = Line::from(vec![
                Span::raw(format!("{:>4} ", i + 1)).fg(Color::DarkGray),
                Span::raw(text.clone()),
            ]);
            if is_cursor_line {
                line = line.style(Style::new().bg(Color::DarkGray));
            }
            line
        })
        .collect();

    let para = Paragraph::new(lines);
    f.render_widget(para, inner);

    // Cursor: a block on the cursor cell, drawn after the text
    let cursor_in_view = cursor_line >= scroll && cursor_line < scroll + viewport_height;
    if cursor_in_view {
        let cursor_y = inner.y + (cursor_line - scroll) as u16;
        let cursor_x = inner.x + 5 + cursor_col as u16;
        if cursor_x < inner.x + inner.width {
            let cell = Rect {
                x: cursor_x,
                y: cursor_y,
                width: 1,
                height: 1,
            };
            f.render_widget(
                Paragraph::new(" ").style(Style::new().bg(Color::Yellow).fg(Color::Yellow)),
                cell,
            );
        }
    }

    // Footer: status/error and key hints
    let footer = match &state.config_editor_error {
        Some(error) => Line::from(vec![
            Span::raw(" ").fg(Color::Red),
            Span::raw(error.clone()).fg(Color::Red).bold(),
        ]),
        None => hint_line(hint_entries(HintSurface::RawEditor)),
    };
    f.render_widget(Paragraph::new(footer), footer_area);
}

fn draw_all_items_panel(f: &mut ratatui::prelude::Frame, area: Rect, state: &mut AppDrawState) {
    // The hint footer's height adapts to the panel width: entries are
    // packed into as few rows as fit, so every binding stays visible
    let hint_lines = hint_rows(hint_entries(HintSurface::LeftPanel), area.width as usize);
    let hints_height = hint_lines.len() as u16;
    let [title_area, list_area, hints_area] = *Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(10),
            Constraint::Length(hints_height),
        ])
        .split(area)
    else {
        return;
    };

    // Title with side indicator
    let title = if state.focus_side == FocusSide::Left {
        Paragraph::new("All Items [LEFT]")
            .style(Style::new().bold().fg(Color::Green))
            .block(Block::default().borders(Borders::BOTTOM))
    } else {
        Paragraph::new("All Items [LEFT]")
            .style(Style::new().bold().fg(Color::White))
            .block(Block::default().borders(Borders::BOTTOM))
    };
    f.render_widget(title, title_area);

    // Group items by category
    let mut categories_map: std::collections::BTreeMap<String, Vec<(usize, &Item)>> =
        std::collections::BTreeMap::new();
    for (idx, item) in state.packing_list.items.iter().enumerate() {
        for category in &item.categories {
            categories_map
                .entry(category.clone())
                .or_default()
                .push((idx, item));
        }
    }

    // Cursor position within the panel's rows (only when the left side is focused)
    let current_row = if state.focus_side == FocusSide::Left {
        Some(state.selected_index)
    } else {
        None
    };

    // Rows in display order (category headers + items of expanded categories)
    let rows = state.left_rows();
    let total_rows = rows.len();

    // Visible window: keep the cursor row in view, otherwise keep the current offset
    let viewport_height = list_area.height as usize;
    let mut scroll = state
        .left_scroll
        .min(total_rows.saturating_sub(viewport_height));
    if let Some(cursor) = current_row {
        if cursor < scroll {
            scroll = cursor;
        } else if cursor >= scroll + viewport_height {
            scroll = cursor + 1 - viewport_height;
        }
    }
    scroll = scroll.min(total_rows.saturating_sub(viewport_height));
    state.left_scroll = scroll;

    let mut row_index = 0;

    for (category, items) in &categories_map {
        let is_collapsed = state.collapsed_categories.contains(category);

        // Category header
        let category_symbol = if is_collapsed { "▶ " } else { "▼ " };
        let category_line = Line::from(vec![
            Span::raw(category_symbol).fg(Color::Yellow),
            Span::raw(category).bold().fg(Color::Cyan),
        ]);

        let category_para =
            Paragraph::new(vec![category_line]).block(Block::default().borders(Borders::NONE));

        if row_index >= scroll && row_index - scroll < viewport_height {
            let category_area = Rect {
                x: list_area.x,
                y: list_area.y + (row_index - scroll) as u16,
                width: list_area.width,
                height: 1,
            };
            f.render_widget(category_para, category_area);

            if current_row == Some(row_index) {
                // Highlight the category header the cursor is on
                let highlight = Block::default().style(Style::new().bg(Color::DarkGray));
                f.render_widget(highlight, category_area);
            }
        }

        row_index += 1;

        // Draw items if category is not collapsed
        if !is_collapsed {
            for (_, item) in items {
                let is_selected = item.editable;
                let is_current = current_row == Some(row_index);

                let item_style = if is_current {
                    Style::new().fg(Color::Yellow).bg(Color::DarkGray)
                } else {
                    Style::new()
                };

                let selection_symbol = if is_selected { "✓" } else { "○" };
                let item_symbol = if item.packed { "✓ " } else { "○ " };

                let item_line = Line::from(vec![
                    Span::raw("  "),
                    Span::raw(selection_symbol).fg(if is_selected {
                        Color::Green
                    } else {
                        Color::Red
                    }),
                    Span::raw(" "),
                    Span::raw(item_symbol).fg(if item.packed {
                        Color::Green
                    } else {
                        Color::Red
                    }),
                    Span::raw(" "),
                    Span::raw(&item.name).style(item_style),
                    Span::raw(" x"),
                    Span::raw(item.quantity.to_string()).bold(),
                ]);

                let item_para =
                    Paragraph::new(vec![item_line]).block(Block::default().borders(Borders::NONE));

                if row_index >= scroll && row_index - scroll < viewport_height {
                    let item_area = Rect {
                        x: list_area.x,
                        y: list_area.y + (row_index - scroll) as u16,
                        width: list_area.width,
                        height: 1,
                    };
                    f.render_widget(item_para, item_area);
                }
                row_index += 1;
            }
        }
    }

    // Scrollbar when the content overflows the viewport
    if total_rows > viewport_height {
        let mut scrollbar_state = ScrollbarState::new(total_rows)
            .position(scroll)
            .viewport_content_length(viewport_height);
        f.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight),
            list_area,
            &mut scrollbar_state,
        );
    }

    // Navigation hints: fixed footer, always visible
    let hints = Paragraph::new(hint_lines).wrap(Wrap { trim: true });
    f.render_widget(hints, hints_area);
}

// Whether every item of a category is packed; an empty category counts
// as all-packed (vacuous truth). Used for the compiled list's category
// header indicator.
fn category_all_packed(packed_states: &[bool]) -> bool {
    packed_states.iter().all(|&p| p)
}

fn draw_selected_items_panel(
    f: &mut ratatui::prelude::Frame,
    area: Rect,
    state: &mut AppDrawState,
) {
    // The hint footer's height adapts to the panel width: entries are
    // packed into as few rows as fit, so every binding stays visible
    let hint_lines = hint_rows(hint_entries(HintSurface::RightPanel), area.width as usize);
    let hints_height = hint_lines.len() as u16;
    let [title_area, list_area, hints_area, stats_area] = *Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(10),
            Constraint::Length(hints_height),
            Constraint::Length(5),
        ])
        .split(area)
    else {
        return;
    };

    // Title with side indicator
    let title = if state.focus_side == FocusSide::Right {
        Paragraph::new("Selected Items [RIGHT]")
            .style(Style::new().bold().fg(Color::Cyan))
            .block(Block::default().borders(Borders::BOTTOM))
    } else {
        Paragraph::new("Selected Items [RIGHT]")
            .style(Style::new().bold().fg(Color::White))
            .block(Block::default().borders(Borders::BOTTOM))
    };
    f.render_widget(title, title_area);

    // Get only selected items
    let selected_items: Vec<(usize, &Item)> = state
        .packing_list
        .items
        .iter()
        .enumerate()
        .filter(|(_, item)| item.editable)
        .collect();

    // Cursor position within the panel's item rows (only when the right side is focused)
    let current_row = if state.focus_side == FocusSide::Right {
        Some(state.selected_index)
    } else {
        None
    };

    // Group selected items by category
    let mut categories_map: std::collections::BTreeMap<String, Vec<(usize, &Item)>> =
        std::collections::BTreeMap::new();
    for (idx, item) in &selected_items {
        categories_map
            .entry(item.primary_category().to_string())
            .or_default()
            .push((*idx, item));
    }

    // Drawn rows: one header per category plus its items
    let mut drawn_rows: Vec<PanelRow> = Vec::new();
    for (category, items) in &categories_map {
        drawn_rows.push(PanelRow::Category(category.clone()));
        for (idx, _) in items {
            drawn_rows.push(PanelRow::Item(*idx));
        }
    }
    let total_rows = drawn_rows.len();

    // Map the cursor (an item-only index) to its position among the drawn rows
    let cursor_drawn_row = current_row.and_then(|cursor| {
        let item_order = state.right_item_order();
        let cursor_item = item_order.get(cursor).copied();
        cursor_item.and_then(|item| {
            drawn_rows
                .iter()
                .position(|r| matches!(r, PanelRow::Item(i) if *i == item))
        })
    });

    // Visible window: keep the cursor row in view, otherwise keep the current offset
    let viewport_height = list_area.height as usize;
    let mut scroll = state
        .right_scroll
        .min(total_rows.saturating_sub(viewport_height));
    if let Some(cursor) = cursor_drawn_row {
        if cursor < scroll {
            scroll = cursor;
        } else if cursor >= scroll + viewport_height {
            scroll = cursor + 1 - viewport_height;
        }
    }
    scroll = scroll.min(total_rows.saturating_sub(viewport_height));
    state.right_scroll = scroll;

    for (row_index, row) in drawn_rows.iter().enumerate() {
        let in_view = row_index >= scroll && row_index - scroll < viewport_height;

        match row {
            PanelRow::Category(category) => {
                // Packing indicator on the compiled list's category header:
                // all-packed (✓ green) vs. partially packed (○ red)
                let packed_states: Vec<bool> = state
                    .packing_list
                    .items
                    .iter()
                    .filter(|item| item.editable && item.in_category(category))
                    .map(|item| item.packed)
                    .collect();
                let all_packed = category_all_packed(&packed_states);
                let (marker, marker_color) = if all_packed {
                    ("✓ ", Color::Green)
                } else {
                    ("○ ", Color::Red)
                };

                let category_line = Line::from(vec![
                    Span::raw(marker).fg(marker_color),
                    Span::raw("▼ "),
                    Span::raw(category).bold().fg(Color::Cyan),
                ]);

                let category_para = Paragraph::new(vec![category_line])
                    .block(Block::default().borders(Borders::NONE));

                if in_view {
                    let category_area = Rect {
                        x: list_area.x,
                        y: list_area.y + (row_index - scroll) as u16,
                        width: list_area.width,
                        height: 1,
                    };
                    f.render_widget(category_para, category_area);
                }
            }
            PanelRow::Item(item_index) => {
                let item = &state.packing_list.items[*item_index];
                let is_current = cursor_drawn_row == Some(row_index);

                let item_style = if is_current {
                    Style::new().fg(Color::Yellow).bg(Color::DarkGray)
                } else {
                    Style::new()
                };

                let item_symbol = if item.packed { "✓ " } else { "○ " };

                let item_line = Line::from(vec![
                    Span::raw("  "),
                    Span::raw(item_symbol).fg(if item.packed {
                        Color::Green
                    } else {
                        Color::Red
                    }),
                    Span::raw(" "),
                    Span::raw(&item.name).style(item_style),
                    Span::raw(" x"),
                    Span::raw(item.quantity.to_string()).bold(),
                ]);

                let item_para =
                    Paragraph::new(vec![item_line]).block(Block::default().borders(Borders::NONE));

                if in_view {
                    let item_area = Rect {
                        x: list_area.x,
                        y: list_area.y + (row_index - scroll) as u16,
                        width: list_area.width,
                        height: 1,
                    };
                    f.render_widget(item_para, item_area);
                }
            }
        }
    }

    // Scrollbar when the content overflows the viewport
    if total_rows > viewport_height {
        let mut scrollbar_state = ScrollbarState::new(total_rows)
            .position(scroll)
            .viewport_content_length(viewport_height);
        f.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight),
            list_area,
            &mut scrollbar_state,
        );
    }

    // Stats
    let total_selected = selected_items.len();
    let total_packing = state.packing_list.items.len();
    let packed = selected_items
        .iter()
        .filter(|(_, item)| item.packed)
        .count();

    let stats = Paragraph::new(vec![
        Line::from(vec![
            Span::raw("Total: ").bold(),
            Span::raw(total_selected.to_string()).fg(Color::White),
            Span::raw(" / "),
            Span::raw(total_packing.to_string()).fg(Color::Gray),
        ]),
        Line::from(vec![
            Span::raw("Packed: ").bold().fg(Color::Green),
            Span::raw(packed.to_string()).fg(Color::Green),
            Span::raw(" | "),
            Span::raw("Unpacked: ").bold().fg(Color::Red),
            Span::raw((total_selected - packed).to_string()).fg(Color::Red),
        ]),
        Line::from(vec![
            Span::raw("Progress: "),
            Span::raw(format!(
                "{:.0}%",
                if total_selected > 0 {
                    packed as f32 / total_selected as f32 * 100.0
                } else {
                    0.0
                }
            ))
            .bold(),
        ]),
    ])
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(stats, stats_area);

    // Navigation hints: fixed footer row, always visible
    let hints = Paragraph::new(hint_lines).wrap(Wrap { trim: true });
    f.render_widget(hints, hints_area);
}

fn draw_add_item_popup(f: &mut ratatui::prelude::Frame, size: Rect, fields: &PopupFields) {
    // Create a centered popup with dynamic size
    let popup_width = (size.width * 60 / 100).clamp(40, 60);
    let popup_height = 14;

    let popup_area = Rect {
        x: (size.width.saturating_sub(popup_width)) / 2,
        y: (size.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height.min(size.height),
    };

    // Draw background overlay
    let background = Block::default()
        .style(Style::new().bg(Color::Black).fg(Color::White))
        .borders(Borders::NONE);
    f.render_widget(background, size);

    // Draw popup
    let popup_block = Block::default()
        .title("Add New Item")
        .borders(Borders::ALL)
        .style(Style::new().bg(Color::Black).fg(Color::White));
    f.render_widget(popup_block, popup_area);

    // Popup content
    let content_area = Rect {
        x: popup_area.x + 1,
        y: popup_area.y + 1,
        width: popup_area.width - 2,
        height: popup_area.height - 2,
    };

    let field_style = Style::new().fg(Color::Yellow).bold();
    let normal_style = Style::new().fg(Color::White);

    // Draw name field
    let name_area = Rect {
        x: content_area.x,
        y: content_area.y,
        width: content_area.width,
        height: 1,
    };
    let name_line = Line::from(vec![
        Span::raw("Name: ").style(if fields.field == InputField::Name {
            field_style
        } else {
            normal_style
        }),
        Span::raw(fields.name),
    ]);
    let name_para = Paragraph::new(vec![name_line]);
    f.render_widget(name_para, name_area);

    // Draw category field
    let category_area = Rect {
        x: content_area.x,
        y: content_area.y + 1,
        width: content_area.width,
        height: 1,
    };
    let category_line = Line::from(vec![Span::raw("Category: ").style(
        if fields.field == InputField::Category {
            field_style
        } else {
            normal_style
        },
    )]);
    let category_para = Paragraph::new(vec![category_line]);
    f.render_widget(category_para, category_area);

    // Draw category dropdown or selected category
    let dropdown_area = Rect {
        x: content_area.x + 10,
        y: content_area.y + 1,
        width: content_area.width - 10,
        height: 1,
    };

    // Draw quantity field
    let quantity_area = Rect {
        x: content_area.x,
        y: content_area.y + 2,
        width: content_area.width,
        height: 1,
    };
    let quantity_line = Line::from(vec![
        Span::raw("Quantity: ").style(if fields.field == InputField::Quantity {
            field_style
        } else {
            normal_style
        }),
        Span::raw(fields.quantity),
    ]);
    let quantity_para = Paragraph::new(vec![quantity_line]);
    f.render_widget(quantity_para, quantity_area);

    // Draw instructions
    let instructions_area = Rect {
        x: content_area.x,
        y: content_area.y + 4,
        width: content_area.width,
        height: content_area.height - 4,
    };
    let instructions = Paragraph::new(hint_line(hint_entries(HintSurface::AddEditPopup)));
    f.render_widget(instructions, instructions_area);

    // Draw category dropdown or selected category (last, so the open dropdown overlays the fields below)
    if fields.field == InputField::Category {
        // Draw dropdown list
        let category_items: Vec<ListItem> = fields
            .category_list
            .iter()
            .map(|c| ListItem::new(c.as_str()))
            .collect();

        let category_list_widget = List::new(category_items)
            .highlight_style(Style::new().fg(Color::Yellow).bold())
            .highlight_symbol("> ");

        let dropdown_height = 5.min(fields.category_list.len() as u16 + 1);
        let dropdown_list_area = Rect {
            x: dropdown_area.x,
            y: dropdown_area.y,
            width: dropdown_area.width,
            height: dropdown_height,
        };
        f.render_widget(category_list_widget, dropdown_list_area);

        // Highlight selected fields.category
        if fields.category_index < fields.category_list.len() {
            let selected_category = &fields.category_list[fields.category_index];
            let selected_line =
                Line::from(vec![Span::raw(selected_category).fg(Color::Yellow).bold()]);
            let selected_para = Paragraph::new(vec![selected_line]);
            f.render_widget(selected_para, dropdown_area);
        }
    } else {
        // Show selected fields.category text
        let selected_category = if fields.category_index < fields.category_list.len() {
            &fields.category_list[fields.category_index]
        } else {
            fields.category
        };
        let category_text = Paragraph::new(vec![Line::from(selected_category)]);
        f.render_widget(category_text, dropdown_area);
    }
}

fn draw_edit_item_popup(f: &mut ratatui::prelude::Frame, size: Rect, fields: &PopupFields) {
    // Create a centered popup with dynamic size
    let popup_width = (size.width * 60 / 100).clamp(40, 60);
    let popup_height = 14;

    let popup_area = Rect {
        x: (size.width.saturating_sub(popup_width)) / 2,
        y: (size.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height.min(size.height),
    };

    // Draw background overlay
    let background = Block::default()
        .style(Style::new().bg(Color::Black).fg(Color::White))
        .borders(Borders::NONE);
    f.render_widget(background, size);

    // Draw popup
    let popup_block = Block::default()
        .title("Edit Item")
        .borders(Borders::ALL)
        .style(Style::new().bg(Color::Black).fg(Color::White));
    f.render_widget(popup_block, popup_area);

    // Popup content
    let content_area = Rect {
        x: popup_area.x + 1,
        y: popup_area.y + 1,
        width: popup_area.width - 2,
        height: popup_area.height - 2,
    };

    let field_style = Style::new().fg(Color::Yellow).bold();
    let normal_style = Style::new().fg(Color::White);

    // Draw name field
    let name_area = Rect {
        x: content_area.x,
        y: content_area.y,
        width: content_area.width,
        height: 1,
    };
    let name_line = Line::from(vec![
        Span::raw("Name: ").style(if fields.field == InputField::Name {
            field_style
        } else {
            normal_style
        }),
        Span::raw(fields.name),
    ]);
    let name_para = Paragraph::new(vec![name_line]);
    f.render_widget(name_para, name_area);

    // Draw category field
    let category_area = Rect {
        x: content_area.x,
        y: content_area.y + 1,
        width: content_area.width,
        height: 1,
    };
    let category_line = Line::from(vec![Span::raw("Category: ").style(
        if fields.field == InputField::Category {
            field_style
        } else {
            normal_style
        },
    )]);
    let category_para = Paragraph::new(vec![category_line]);
    f.render_widget(category_para, category_area);

    // Draw category dropdown or selected category
    let dropdown_area = Rect {
        x: content_area.x + 10,
        y: content_area.y + 1,
        width: content_area.width - 10,
        height: 1,
    };

    // Draw quantity field
    let quantity_area = Rect {
        x: content_area.x,
        y: content_area.y + 2,
        width: content_area.width,
        height: 1,
    };
    let quantity_line = Line::from(vec![
        Span::raw("Quantity: ").style(if fields.field == InputField::Quantity {
            field_style
        } else {
            normal_style
        }),
        Span::raw(fields.quantity),
    ]);
    let quantity_para = Paragraph::new(vec![quantity_line]);
    f.render_widget(quantity_para, quantity_area);

    // Draw mode field (edit dialog only): automatic or fixed quantity
    if let Some(auto) = fields.auto_quantity {
        let mode_area = Rect {
            x: content_area.x,
            y: content_area.y + 3,
            width: content_area.width,
            height: 1,
        };
        let mode_line = Line::from(vec![
            Span::raw("Mode: ").style(if fields.field == InputField::Mode {
                field_style
            } else {
                normal_style
            }),
            Span::raw(if auto { "automatic" } else { "fixed" }).style(
                if fields.field == InputField::Mode {
                    field_style
                } else {
                    normal_style
                },
            ),
            Span::raw("  (Space toggles)").dim(),
        ]);
        let mode_para = Paragraph::new(vec![mode_line]);
        f.render_widget(mode_para, mode_area);
    }

    // Draw instructions
    let instructions_area = Rect {
        x: content_area.x,
        y: content_area.y + 4,
        width: content_area.width,
        height: content_area.height - 4,
    };
    let instructions = Paragraph::new(hint_line(hint_entries(HintSurface::AddEditPopup)));
    f.render_widget(instructions, instructions_area);

    // Draw category dropdown or selected category (last, so the open dropdown overlays the fields below)
    if fields.field == InputField::Category {
        // Draw dropdown list
        let category_items: Vec<ListItem> = fields
            .category_list
            .iter()
            .map(|c| ListItem::new(c.as_str()))
            .collect();

        let category_list_widget = List::new(category_items)
            .highlight_style(Style::new().fg(Color::Yellow).bold())
            .highlight_symbol("> ");

        let dropdown_height = 5.min(fields.category_list.len() as u16 + 1);
        let dropdown_list_area = Rect {
            x: dropdown_area.x,
            y: dropdown_area.y,
            width: dropdown_area.width,
            height: dropdown_height,
        };
        f.render_widget(category_list_widget, dropdown_list_area);

        // Highlight selected fields.category
        if fields.category_index < fields.category_list.len() {
            let selected_category = &fields.category_list[fields.category_index];
            let selected_line =
                Line::from(vec![Span::raw(selected_category).fg(Color::Yellow).bold()]);
            let selected_para = Paragraph::new(vec![selected_line]);
            f.render_widget(selected_para, dropdown_area);
        }
    } else {
        // Show selected fields.category text
        let selected_category = if fields.category_index < fields.category_list.len() {
            &fields.category_list[fields.category_index]
        } else {
            fields.category
        };
        let category_text = Paragraph::new(vec![Line::from(selected_category)]);
        f.render_widget(category_text, dropdown_area);
    }
}

fn draw_confirm_delete_popup(
    f: &mut ratatui::prelude::Frame,
    size: Rect,
    index: usize,
    packing_list: &PackingList,
) {
    // Create a centered popup with dynamic size
    let popup_width = (size.width * 40 / 100).clamp(30, 40);
    let popup_height = 8;

    let popup_area = Rect {
        x: (size.width.saturating_sub(popup_width)) / 2,
        y: (size.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height.min(size.height),
    };

    // Draw background overlay
    let background = Block::default()
        .style(Style::new().bg(Color::Black).fg(Color::White))
        .borders(Borders::NONE);
    f.render_widget(background, size);

    // Draw popup
    let popup_block = Block::default()
        .title("Confirm Delete")
        .borders(Borders::ALL)
        .style(Style::new().bg(Color::Black).fg(Color::White));
    f.render_widget(popup_block, popup_area);

    // Popup content
    let content_area = Rect {
        x: popup_area.x + 1,
        y: popup_area.y + 1,
        width: popup_area.width - 2,
        height: popup_area.height - 2,
    };

    let item_name = if index < packing_list.items.len() {
        &packing_list.items[index].name
    } else {
        "Unknown"
    };

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("Delete ").bold(),
            Span::raw(item_name).fg(Color::Red).bold(),
            Span::raw("?"),
        ]),
        Line::from(""),
        hint_line(hint_entries(HintSurface::DeletePopup)),
    ];

    let content = Paragraph::new(lines);
    f.render_widget(content, content_area);
}

impl Drop for App {
    fn drop(&mut self) {
        let _ = stdout().execute(LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}
