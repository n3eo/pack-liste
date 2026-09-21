mod config_io;
mod hints;
mod input;
mod state;

use crate::config::modifiers::ModifiersConfig;
use crate::models::item::Item;
use crate::models::packing_list::PackingList;
use crate::models::weekdays::Weekdays;
use crate::ui::hints::HintSurface;
use crate::ui::state::{AppDrawState, ConfigSection, Focus, FocusSide, InputMode, Screen};
use std::path::PathBuf;

fn test_item(name: &str, label: &str) -> Item {
    Item::new(name.to_string(), vec![label.to_string()], 1)
}

fn test_state(items: Vec<Item>) -> AppDrawState {
    let mut packing_list = PackingList::new();
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
        collapsed_labels: Vec::new(),
        occurrence_overrides: std::collections::BTreeMap::new(),
        selected_index: 0,
        focus_side: FocusSide::Left,
        fullscreen: false,
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
        status: None,
    }
}

// ---- Input-handling tests (US5): key events in, state out ----

fn all_hint_surfaces() -> [HintSurface; 8] {
    [
        HintSurface::MainScreen,
        HintSurface::Welcome,
        HintSurface::Config,
        HintSurface::AddEditPopup,
        HintSurface::DeletePopup,
        HintSurface::ConfirmEditPopup,
        HintSurface::ModifierEditor,
        HintSurface::RawEditor,
    ]
}
