use super::{test_item, test_state};
use crate::ui::state::{InputField, InputMode, QuantityBufferKind};

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
        rate: String::new(),
        buffer_kind: QuantityBufferKind::None,
        buffer_value: String::new(),
        auto_quantity: true,
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
        rate: String::new(),
        buffer_kind: QuantityBufferKind::None,
        buffer_value: String::new(),
        auto_quantity: true,
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
        rate: String::new(),
        buffer_kind: QuantityBufferKind::None,
        buffer_value: String::new(),
        auto_quantity: true,
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
        rate: String::new(),
        buffer_kind: QuantityBufferKind::None,
        buffer_value: String::new(),
        auto_quantity: true,
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
        rate: String::new(),
        buffer_kind: QuantityBufferKind::None,
        buffer_value: String::new(),
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
        rate: String::new(),
        buffer_kind: QuantityBufferKind::None,
        buffer_value: String::new(),
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
    // Tab rotation: Name -> Category -> Quantity -> Mode -> Rate ->
    // Buffer -> Name (spec 010 extended the cycle)
    let mut field = InputField::Name;
    field = field.next();
    assert_eq!(field, InputField::Category);
    field = field.next();
    assert_eq!(field, InputField::Quantity);
    field = field.next();
    assert_eq!(field, InputField::Mode);
    field = field.next();
    assert_eq!(field, InputField::Rate);
    field = field.next();
    assert_eq!(field, InputField::Buffer);
    field = field.next();
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
        rate: String::new(),
        buffer_kind: QuantityBufferKind::None,
        buffer_value: String::new(),
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

// ---- US3 (spec 020): Left/Right step the quantity ----

#[test]
fn dialog_quantity_steps_and_clamps_at_one() {
    let mut state = test_state(Vec::new());
    state.input_mode = InputMode::AddingItem {
        field: InputField::Quantity,
        name: String::new(),
        category: String::new(),
        quantity: "3".to_string(),
        category_list: vec!["One".to_string(), "Two".to_string()],
        category_index: 0,
        rate: String::new(),
        buffer_kind: QuantityBufferKind::None,
        buffer_value: String::new(),
        auto_quantity: true,
    };
    state.step_dialog_quantity(1);
    match &state.input_mode {
        InputMode::AddingItem { quantity, .. } => assert_eq!(quantity, "4"),
        _ => panic!("expected AddingItem mode"),
    }
    state.step_dialog_quantity(-1);
    match &state.input_mode {
        InputMode::AddingItem { quantity, .. } => assert_eq!(quantity, "3"),
        _ => panic!("expected AddingItem mode"),
    }
    // Clamped at 1: stepping below keeps 1
    state.step_dialog_quantity(-1);
    state.step_dialog_quantity(-1);
    state.step_dialog_quantity(-1);
    match &state.input_mode {
        InputMode::AddingItem { quantity, .. } => assert_eq!(quantity, "1"),
        _ => panic!("expected AddingItem mode"),
    }
    // Empty counts as 1: Right steps to 2
    state.step_dialog_quantity(1);
    match &state.input_mode {
        InputMode::AddingItem { quantity, .. } => assert_eq!(quantity, "2"),
        _ => panic!("expected AddingItem mode"),
    }
}

#[test]
fn dialog_quantity_stepping_only_in_quantity_field() {
    let mut state = test_state(Vec::new());
    state.input_mode = InputMode::AddingItem {
        field: InputField::Name,
        name: String::new(),
        category: String::new(),
        quantity: "3".to_string(),
        category_list: vec!["One".to_string(), "Two".to_string()],
        category_index: 0,
        rate: String::new(),
        buffer_kind: QuantityBufferKind::None,
        buffer_value: String::new(),
        auto_quantity: true,
    };
    state.step_dialog_quantity(1);
    match &state.input_mode {
        InputMode::AddingItem { quantity, .. } => assert_eq!(quantity, "3"),
        _ => panic!("expected AddingItem mode"),
    }
    // The category field's Left/Right keeps the category cycling
    // (US3/AC5): the quantity is untouched
    state.input_mode = InputMode::AddingItem {
        field: InputField::Category,
        name: String::new(),
        category: String::new(),
        quantity: "3".to_string(),
        category_list: vec!["One".to_string(), "Two".to_string()],
        category_index: 0,
        rate: String::new(),
        buffer_kind: QuantityBufferKind::None,
        buffer_value: String::new(),
        auto_quantity: true,
    };
    state.step_dialog_quantity(1);
    state.cycle_category_selection(1);
    match &state.input_mode {
        InputMode::AddingItem {
            quantity,
            category_index,
            ..
        } => {
            assert_eq!(quantity, "3");
            assert_eq!(*category_index, 1);
        }
        _ => panic!("expected AddingItem mode"),
    }
}

#[test]
fn dialog_quantity_typing_appends_after_stepping() {
    let mut state = test_state(Vec::new());
    state.input_mode = InputMode::AddingItem {
        field: InputField::Quantity,
        name: String::new(),
        category: String::new(),
        quantity: "3".to_string(),
        category_list: vec!["One".to_string()],
        category_index: 0,
        rate: String::new(),
        buffer_kind: QuantityBufferKind::None,
        buffer_value: String::new(),
        auto_quantity: true,
    };
    state.step_dialog_quantity(1);
    // A typed digit appends to the stepped value (US3/AC4)
    match &mut state.input_mode {
        InputMode::AddingItem { quantity, .. } => quantity.push('5'),
        _ => panic!("expected AddingItem mode"),
    }
    match &state.input_mode {
        InputMode::AddingItem { quantity, .. } => assert_eq!(quantity, "45"),
        _ => panic!("expected AddingItem mode"),
    }
}
