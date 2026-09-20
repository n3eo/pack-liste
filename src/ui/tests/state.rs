use super::{test_item, test_state};
use crate::quantity::calculation::QuantityCalculation;
use crate::ui::screens::main::category_all_packed;
use crate::ui::state::{Focus, FocusSide, PanelRow, Screen, StatusKind};

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
    assert!(state.packing_list.items[0].selected);
    state.toggle_item_selection();
    assert!(!state.packing_list.items[0].selected);
    state.toggle_item_selection();
    assert!(state.packing_list.items[0].selected);
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
    assert!(!state.packing_list.items[0].selected);
    assert!(!state.packing_list.items[1].selected);
    assert!(state.packing_list.items[2].selected);
    assert!(state.collapsed_categories.contains(&"Cat".to_string()));
    // Cursor stays on the header row after collapsing
    assert_eq!(state.selected_index, 0);
    // Toggling back reselects and expands
    state.toggle_category_selection();
    assert!(state.packing_list.items[0].selected);
    assert!(state.packing_list.items[1].selected);
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
fn recalculate_updates_auto_and_preserves_fixed() {
    let mut auto_item = test_item("Auto", "Cat");
    auto_item.quantity = 99; // stale bogus value
    let mut fixed_item = test_item("Fixed", "Cat");
    fixed_item.quantity = 3;
    fixed_item.auto_quantity = false;
    let mut state = test_state(vec![auto_item, fixed_item]);
    // Non-quantity state the action must not touch
    state.packing_list.items[0].selected = true;
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
    assert!(state.packing_list.items[0].selected);
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
    assert!(!state.packing_list.items[0].selected);
    // The compiled list shows the item at most once (primary category)
    let right = state.right_item_order();
    assert_eq!(right.len(), 0, "deselected item gone from compiled list");
    // Reselect through the Gear occurrence
    state.selected_index = 2; // Gear header is row 2, item row 3
    state.selected_index = 3;
    state.toggle_item_selection();
    assert!(state.packing_list.items[0].selected);
    let right = state.right_item_order();
    assert_eq!(right.len(), 1, "item appears once in compiled list");
}

// ---- US5 (spec 022): the status line ----

#[test]
fn status_set_and_cleared() {
    let mut state = test_state(vec![test_item("A", "Cat")]);
    assert!(state.status.is_none(), "no status initially");
    state.set_status(StatusKind::Error, "Save failed: disk full");
    let status = state.status.as_ref().expect("status set");
    assert_eq!(status.message, "Save failed: disk full");
    assert_eq!(status.kind, StatusKind::Error);
    state.clear_status();
    assert!(state.status.is_none(), "status cleared");
}

#[test]
fn status_latest_message_wins() {
    let mut state = test_state(Vec::new());
    state.set_status(StatusKind::Error, "first");
    state.set_status(StatusKind::Error, "second");
    assert_eq!(
        state.status.as_ref().expect("status set").message,
        "second",
        "the most recent message is shown"
    );
}
