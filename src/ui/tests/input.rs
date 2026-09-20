use super::{test_item, test_state};
use crate::ui::state::{InputField, InputMode};

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
