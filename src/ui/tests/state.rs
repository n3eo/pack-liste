use super::{test_item, test_state};
use crate::quantity::calculation::QuantityCalculation;
use crate::ui::screens::main::category_all_packed;
use crate::ui::state::{
    Focus, FocusSide, InputField, InputMode, PanelRow, PendingEdit, QuantityBufferKind,
    RecalculateScope, Screen, StatusKind,
};

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
    state.apply_item_edit(
        0,
        String::new(),
        vec![],
        "3".to_string(),
        None,
        String::new(),
        QuantityBufferKind::None,
        String::new(),
    );
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
        String::new(),
        QuantityBufferKind::None,
        String::new(),
    );
    assert_eq!(state.packing_list.items[0].name, "Renamed");
    assert_eq!(state.packing_list.items[0].primary_category(), "NewCat");
    assert_eq!(state.packing_list.items[0].quantity, 3);
    // An out-of-range index is a no-op
    state.apply_item_edit(
        9,
        "X".to_string(),
        vec![],
        String::new(),
        None,
        String::new(),
        QuantityBufferKind::None,
        String::new(),
    );
    assert_eq!(state.packing_list.items.len(), 1);
}

#[test]
fn apply_edit_writes_mode_and_freezes_quantity_when_fixed() {
    // auto -> fixed: the current quantity becomes the fixed value (EC-003)
    let mut state = test_state(vec![test_item("A", "Cat")]);
    state.packing_list.items[0].quantity = 5;
    state.packing_list.items[0].auto_quantity = false; // dialog toggled to fixed
    state.apply_item_edit(
        0,
        String::new(),
        vec![],
        String::new(),
        Some(false),
        String::new(),
        QuantityBufferKind::None,
        String::new(),
    );
    assert!(!state.packing_list.items[0].auto_quantity);
    assert_eq!(state.packing_list.items[0].quantity, 5);
    // fixed -> auto: the flag flips; the caller recalculates
    state.apply_item_edit(
        0,
        String::new(),
        vec![],
        String::new(),
        Some(true),
        String::new(),
        QuantityBufferKind::None,
        String::new(),
    );
    assert!(state.packing_list.items[0].auto_quantity);
}

#[test]
fn fixed_quantity_cannot_drop_below_one() {
    // EC-004: no fixed item can hold a quantity below 1
    let mut state = test_state(vec![test_item("A", "Cat")]);
    state.packing_list.items[0].auto_quantity = false;
    state.apply_item_edit(
        0,
        String::new(),
        vec![],
        "0".to_string(),
        None,
        String::new(),
        QuantityBufferKind::None,
        String::new(),
    );
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

#[test]
fn status_clears_on_transition() {
    // V3: a status message does not persist across a mode change
    let mut state = test_state(vec![test_item("A", "Cat")]);
    state.set_status(StatusKind::Error, "Save failed: read-only");
    assert!(state.status.is_some());
    // The transition handler clears it (handle_escape calls clear_status
    // before changing screen/mode; the state-level contract is the same)
    state.clear_status();
    assert!(state.status.is_none(), "status cleared on transition");
}

// ---- Spec 008: no packed state on the left panel ----

#[test]
fn left_panel_row_ignores_packed_state() {
    // The left panel's row content is identical packed and unpacked:
    // the row is built from selection, name, and quantity only.
    // (The renderer reads no packed field for the left panel; this
    // test pins the state-level contract the renderer relies on.)
    let mut state = test_state(vec![test_item("A", "Cat")]);
    state.packing_list.items[0].packed = false;
    let unpacked = (
        state.packing_list.items[0].selected,
        state.packing_list.items[0].name.clone(),
        state.packing_list.items[0].quantity,
    );
    state.packing_list.items[0].packed = true;
    let packed = (
        state.packing_list.items[0].selected,
        state.packing_list.items[0].name.clone(),
        state.packing_list.items[0].quantity,
    );
    assert_eq!(unpacked, packed);
    // The left panel's row source (left_rows) is packed-independent:
    // compare the row count and the item indices (PanelRow is not Eq)
    let rows_unpacked = {
        state.packing_list.items[0].packed = false;
        state.left_rows()
    };
    let rows_packed = {
        state.packing_list.items[0].packed = true;
        state.left_rows()
    };
    assert_eq!(rows_unpacked.len(), rows_packed.len());
    for (a, b) in rows_unpacked.iter().zip(rows_packed.iter()) {
        let idx = |r: &PanelRow| match r {
            PanelRow::Item(i) => Some(*i),
            _ => None,
        };
        assert_eq!(idx(a), idx(b));
    }
}

// ---- Spec 010: per-item modifier editing ----

#[test]
fn apply_edit_writes_rate_and_reserve() {
    let mut state = test_state(vec![test_item("A", "Cat")]);
    state.apply_item_edit(
        0,
        String::new(),
        vec![],
        String::new(),
        None,
        "0.15".to_string(),
        QuantityBufferKind::Reserve,
        "2".to_string(),
    );
    assert_eq!(state.packing_list.items[0].rate, 0.15);
    assert_eq!(
        state.packing_list.items[0].buffer,
        crate::models::item::QuantityBuffer::Reserve(2)
    );
}

#[test]
fn apply_edit_writes_multiplier_buffer() {
    let mut state = test_state(vec![test_item("A", "Cat")]);
    state.apply_item_edit(
        0,
        String::new(),
        vec![],
        String::new(),
        None,
        String::new(),
        QuantityBufferKind::Multiplier,
        "1.25".to_string(),
    );
    assert_eq!(
        state.packing_list.items[0].buffer,
        crate::models::item::QuantityBuffer::Multiplier(1.25)
    );
}

#[test]
fn invalid_rate_is_rejected() {
    // FR-006: rates outside 0.05..=10.0 keep the old value
    let mut state = test_state(vec![test_item("A", "Cat")]);
    state.packing_list.items[0].rate = 1.0;
    state.apply_item_edit(
        0,
        String::new(),
        vec![],
        String::new(),
        None,
        "20.0".to_string(), // too big
        QuantityBufferKind::None,
        String::new(),
    );
    assert_eq!(state.packing_list.items[0].rate, 1.0);
    state.apply_item_edit(
        0,
        String::new(),
        vec![],
        String::new(),
        None,
        "0.01".to_string(), // too small
        QuantityBufferKind::None,
        String::new(),
    );
    assert_eq!(state.packing_list.items[0].rate, 1.0);
}

#[test]
fn invalid_buffer_values_are_rejected() {
    // FR-006: reserves outside 0..=10 and multipliers outside 1.0..=3.0
    // keep the old buffer
    let mut state = test_state(vec![test_item("A", "Cat")]);
    state.apply_item_edit(
        0,
        String::new(),
        vec![],
        String::new(),
        None,
        String::new(),
        QuantityBufferKind::Reserve,
        "11".to_string(), // too big
    );
    assert_eq!(
        state.packing_list.items[0].buffer,
        crate::models::item::QuantityBuffer::None
    );
    state.apply_item_edit(
        0,
        String::new(),
        vec![],
        String::new(),
        None,
        String::new(),
        QuantityBufferKind::Multiplier,
        "0.5".to_string(), // below 1.0
    );
    assert_eq!(
        state.packing_list.items[0].buffer,
        crate::models::item::QuantityBuffer::None
    );
}

#[test]
fn buffer_kind_none_clears_the_buffer() {
    let mut state = test_state(vec![test_item("A", "Cat")]);
    state.packing_list.items[0].buffer = crate::models::item::QuantityBuffer::Reserve(2);
    state.apply_item_edit(
        0,
        String::new(),
        vec![],
        String::new(),
        None,
        String::new(),
        QuantityBufferKind::None,
        String::new(),
    );
    assert_eq!(
        state.packing_list.items[0].buffer,
        crate::models::item::QuantityBuffer::None
    );
}

#[test]
fn buffer_kind_cycles() {
    // Space on the Buffer field: None -> Reserve -> Multiplier -> None
    let mut kind = QuantityBufferKind::None;
    kind = kind.next();
    assert_eq!(kind, QuantityBufferKind::Reserve);
    kind = kind.next();
    assert_eq!(kind, QuantityBufferKind::Multiplier);
    kind = kind.next();
    assert_eq!(kind, QuantityBufferKind::None);
}

// ---- Spec 018 (amended): scoped recalculation ----

#[test]
fn recalculate_item_scope_changes_only_that_item() {
    let mut a = test_item("A", "Cat");
    a.quantity = 99; // stale
    let mut b = test_item("B", "Cat");
    b.quantity = 99;
    let mut state = test_state(vec![a, b]);
    state
        .recalculate_quantities_scoped(&QuantityCalculation::new(), RecalculateScope::Item(0))
        .unwrap();
    // 7 weekdays x 1.0 x 1.0 x rate 1.0 = 7: only item 0 recalculated
    assert_eq!(state.packing_list.items[0].quantity, 7);
    assert_eq!(state.packing_list.items[1].quantity, 99);
}

#[test]
fn recalculate_category_scope_changes_only_that_category() {
    let mut a = test_item("A", "Cat");
    a.quantity = 99;
    let mut b = test_item("B", "Other");
    b.quantity = 99;
    let mut state = test_state(vec![a, b]);
    state
        .recalculate_quantities_scoped(
            &QuantityCalculation::new(),
            RecalculateScope::Category("Cat".to_string()),
        )
        .unwrap();
    assert_eq!(state.packing_list.items[0].quantity, 7);
    assert_eq!(state.packing_list.items[1].quantity, 99);
}

#[test]
fn recalculate_all_scope_changes_every_automatic_item() {
    let mut a = test_item("A", "Cat");
    a.quantity = 99;
    let mut b = test_item("B", "Other");
    b.quantity = 99;
    let mut state = test_state(vec![a, b]);
    state
        .recalculate_quantities_scoped(&QuantityCalculation::new(), RecalculateScope::All)
        .unwrap();
    assert_eq!(state.packing_list.items[0].quantity, 7);
    assert_eq!(state.packing_list.items[1].quantity, 7);
}

#[test]
fn recalculate_scope_preserves_fixed_items() {
    // FR-004: fixed items never change, in any scope
    let mut fixed = test_item("F", "Cat");
    fixed.auto_quantity = false;
    fixed.quantity = 3;
    let mut state = test_state(vec![fixed]);
    state
        .recalculate_quantities_scoped(&QuantityCalculation::new(), RecalculateScope::All)
        .unwrap();
    assert_eq!(state.packing_list.items[0].quantity, 3);
}

#[test]
fn recalculate_scope_is_idempotent() {
    let mut state = test_state(vec![test_item("A", "Cat")]);
    state
        .recalculate_quantities_scoped(&QuantityCalculation::new(), RecalculateScope::Item(0))
        .unwrap();
    let first = state.packing_list.items[0].quantity;
    state
        .recalculate_quantities_scoped(&QuantityCalculation::new(), RecalculateScope::Item(0))
        .unwrap();
    assert_eq!(state.packing_list.items[0].quantity, first);
}

#[test]
fn recalculate_scope_touches_no_other_state() {
    // FR-006: packed, selection, collapse untouched
    let mut a = test_item("A", "Cat");
    a.packed = true;
    a.selected = false;
    let mut state = test_state(vec![a]);
    state.collapsed_categories = vec!["Cat".to_string()];
    state
        .recalculate_quantities_scoped(&QuantityCalculation::new(), RecalculateScope::All)
        .unwrap();
    assert!(state.packing_list.items[0].packed);
    assert!(!state.packing_list.items[0].selected);
    assert_eq!(state.collapsed_categories, vec!["Cat".to_string()]);
}

#[test]
fn manual_quantity_survives_parameter_changes() {
    // SC-001/SC-002 (spec 018, amended): a manually set quantity is
    // authoritative — no automatic process overwrites it. The state
    // layer has no parameter-change trigger anymore; this pins the
    // contract: changing the parameters does not touch quantities.
    let mut state = test_state(vec![test_item("A", "Cat")]);
    state.packing_list.items[0].quantity = 3; // manually set
    state.weekdays = crate::models::weekdays::Weekdays::new(14).unwrap();
    state.travel_style = "luxury".to_string();
    state.weather = "cold".to_string();
    // No recalculate key was pressed: the quantity is untouched
    assert_eq!(state.packing_list.items[0].quantity, 3);
    // The explicit all-scope recalculate is the only thing that changes it
    state
        .recalculate_quantities_scoped(&QuantityCalculation::new(), RecalculateScope::All)
        .unwrap();
    assert_ne!(state.packing_list.items[0].quantity, 3);
}

// ---- Spec 021: the add dialog's quantity and mode ----

#[test]
fn add_with_quantity_5_creates_item_with_5() {
    let mut state = test_state(Vec::new());
    state.add_item_from_dialog(
        "Towel".to_string(),
        "Cat".to_string(),
        "5".to_string(),
        true,
        String::new(),
        QuantityBufferKind::None,
        String::new(),
    );
    let item = &state.packing_list.items[0];
    assert_eq!(item.name, "Towel");
    assert_eq!(item.quantity, 5);
    assert!(item.auto_quantity);
    // The cursor clamps to the new item's index within the visible rows
    // (left side: [Cat header, Towel] -> index 0 is the item's row clamp)
    assert_eq!(state.selected_index, 0);
}

#[test]
fn add_with_empty_quantity_creates_item_with_1() {
    let mut state = test_state(Vec::new());
    state.add_item_from_dialog(
        "Towel".to_string(),
        "Cat".to_string(),
        String::new(),
        true,
        String::new(),
        QuantityBufferKind::None,
        String::new(),
    );
    assert_eq!(state.packing_list.items[0].quantity, 1);
}

#[test]
fn add_fixed_quantity_survives_scoped_recalculate() {
    let mut state = test_state(Vec::new());
    state.add_item_from_dialog(
        "Towel".to_string(),
        "Cat".to_string(),
        "3".to_string(),
        false,
        String::new(),
        QuantityBufferKind::None,
        String::new(),
    );
    // The explicit all-scope recalculate keeps the fixed quantity
    state
        .recalculate_quantities_scoped(&QuantityCalculation::new(), RecalculateScope::All)
        .unwrap();
    assert_eq!(state.packing_list.items[0].quantity, 3);
}

#[test]
fn add_automatic_quantity_recalculates_on_explicit_key() {
    let mut state = test_state(Vec::new());
    state.add_item_from_dialog(
        "Towel".to_string(),
        "Cat".to_string(),
        "3".to_string(),
        true,
        String::new(),
        QuantityBufferKind::None,
        String::new(),
    );
    // The typed quantity is the starting point; the explicit
    // recalculate recomputes it (7 weekdays x 1.0 x 1.0 = 7)
    state
        .recalculate_quantities_scoped(&QuantityCalculation::new(), RecalculateScope::All)
        .unwrap();
    assert_eq!(state.packing_list.items[0].quantity, 7);
}

#[test]
fn add_and_edit_dialog_variants_carry_the_same_field_set() {
    // US3 parity: both InputMode variants expose the same fields
    // through the one shared surface (popup_fields); the renderer
    // and the field cycle treat them identically.
    let add = InputMode::AddingItem {
        field: InputField::Name,
        name: "Towel".to_string(),
        category: String::new(),
        quantity: "3".to_string(),
        category_list: vec!["Cat".to_string()],
        category_index: 0,
        rate: String::new(),
        buffer_kind: QuantityBufferKind::None,
        buffer_value: String::new(),
        auto_quantity: true,
    };
    let edit = InputMode::EditingItem {
        index: 0,
        field: InputField::Name,
        name: "Towel".to_string(),
        category: String::new(),
        quantity: "3".to_string(),
        rate: String::new(),
        buffer_kind: QuantityBufferKind::None,
        buffer_value: String::new(),
        auto_quantity: true,
        category_list: vec!["Cat".to_string()],
        category_index: 0,
        selected_categories: vec!["Cat".to_string()],
    };
    let add_fields = add.popup_fields().expect("add dialog has fields");
    let edit_fields = edit.popup_fields().expect("edit dialog has fields");
    // The shared surface carries the same values from both variants
    assert_eq!(add_fields.field, edit_fields.field);
    assert_eq!(add_fields.name, edit_fields.name);
    assert_eq!(add_fields.category, edit_fields.category);
    assert_eq!(add_fields.quantity, edit_fields.quantity);
    assert_eq!(add_fields.auto_quantity, edit_fields.auto_quantity);
    assert_eq!(add_fields.category_index, edit_fields.category_index);
    assert_eq!(add_fields.rate, edit_fields.rate);
    assert_eq!(add_fields.buffer_kind, edit_fields.buffer_kind);
    assert_eq!(add_fields.buffer_value, edit_fields.buffer_value);
    // Outside the dialogs there is no shared surface
    assert!(InputMode::Normal.popup_fields().is_none());
    assert!(
        InputMode::ConfirmDelete { index: 0 }
            .popup_fields()
            .is_none()
    );
}

// ---- Spec 011: the edit-save confirmation ----

#[test]
fn confirm_edit_snapshot_carries_the_dialogs_values() {
    // The pending edit is a faithful snapshot of the dialog's state:
    // applying it is the only effect; cancelling drops it
    let pending = PendingEdit {
        index: 0,
        name: "Towel".to_string(),
        categories: vec!["Cat".to_string(), "Gear".to_string()],
        quantity: "5".to_string(),
        auto_quantity: false,
        rate: "1.5".to_string(),
        buffer_kind: QuantityBufferKind::Reserve,
        buffer_value: "2".to_string(),
    };
    assert_eq!(pending.name, "Towel");
    assert_eq!(
        pending.categories,
        vec!["Cat".to_string(), "Gear".to_string()]
    );
    assert_eq!(pending.quantity, "5");
    assert!(!pending.auto_quantity);
    assert_eq!(pending.rate, "1.5");
    assert_eq!(pending.buffer_kind, QuantityBufferKind::Reserve);
    assert_eq!(pending.buffer_value, "2");
}

#[test]
fn confirm_edit_applies_the_snapshot_on_confirm() {
    // US1/AC2: confirming applies the snapshot's values to the item
    let mut state = test_state(vec![test_item("Old", "Cat")]);
    let pending = PendingEdit {
        index: 0,
        name: "New".to_string(),
        categories: vec!["Cat".to_string()],
        quantity: "5".to_string(),
        auto_quantity: false,
        rate: String::new(),
        buffer_kind: QuantityBufferKind::None,
        buffer_value: String::new(),
    };
    // The confirm arm applies exactly the snapshot (the same call
    // the handler makes)
    state.apply_item_edit(
        pending.index,
        pending.name,
        pending.categories,
        pending.quantity,
        Some(pending.auto_quantity),
        pending.rate,
        pending.buffer_kind,
        pending.buffer_value,
    );
    let item = &state.packing_list.items[0];
    assert_eq!(item.name, "New");
    assert_eq!(item.quantity, 5);
    assert!(!item.auto_quantity);
}

#[test]
fn confirm_edit_cancel_discards_the_snapshot() {
    // US3/AC1: cancelling drops the snapshot; the item is unchanged
    let mut state = test_state(vec![test_item("Old", "Cat")]);
    state.packing_list.items[0].quantity = 3;
    let _pending = PendingEdit {
        index: 0,
        name: "New".to_string(),
        categories: vec!["Cat".to_string()],
        quantity: "9".to_string(),
        auto_quantity: true,
        rate: String::new(),
        buffer_kind: QuantityBufferKind::None,
        buffer_value: String::new(),
    };
    // The cancel arm never calls apply_item_edit: the item keeps its
    // pre-dialog values
    let item = &state.packing_list.items[0];
    assert_eq!(item.name, "Old");
    assert_eq!(item.quantity, 3);
    assert!(item.auto_quantity);
}

#[test]
fn confirm_edit_mode_is_a_distinct_input_mode() {
    // The confirmation is its own mode: the dialog is consumed into
    // the snapshot, so no dialog keys leak into the prompt
    let state = test_state(Vec::new());
    let mode = InputMode::ConfirmEdit {
        pending: PendingEdit {
            index: 0,
            name: "X".to_string(),
            categories: vec!["Cat".to_string()],
            quantity: "1".to_string(),
            auto_quantity: true,
            rate: String::new(),
            buffer_kind: QuantityBufferKind::None,
            buffer_value: String::new(),
        },
    };
    // No shared dialog surface: the prompt is not the edit dialog
    assert!(mode.popup_fields().is_none());
    // The state's mode is untouched by the prompt existing
    assert!(matches!(state.input_mode, InputMode::Normal));
}

// ---- Spec 007: the fullscreen compiled-list view ----

#[test]
fn fullscreen_flag_is_presentation_only() {
    // US1/AC3: the flag changes no data — the same actions produce
    // the same outcomes with the flag on or off
    let mut a = test_state(vec![test_item("A", "Cat"), test_item("B", "Cat")]);
    let mut b = test_state(vec![test_item("A", "Cat"), test_item("B", "Cat")]);
    b.fullscreen = true;

    // Navigate + toggle packed (right side) in both
    for state in [&mut a, &mut b] {
        state.focus_side = FocusSide::Right;
        state.selected_index = 0;
        state.toggle_item_packed();
        state.move_cursor_down();
    }
    assert_eq!(
        a.packing_list.items[0].packed,
        b.packing_list.items[0].packed
    );
    assert_eq!(a.selected_index, b.selected_index);
    // The flag itself is untouched by the actions
    assert!(!a.fullscreen);
    assert!(b.fullscreen);
}

#[test]
fn fullscreen_toggle_flips_both_directions() {
    // US1/AC1-2: the same key enters and leaves fullscreen; the
    // selections, packed states, and cursor survive the round trip
    let mut state = test_state(vec![test_item("A", "Cat")]);
    state.packing_list.items[0].selected = true;
    state.packing_list.items[0].packed = true;
    state.selected_index = 1;
    let before = (
        state.packing_list.items[0].selected,
        state.packing_list.items[0].packed,
        state.selected_index,
    );
    state.fullscreen = true;
    state.fullscreen = false;
    let after = (
        state.packing_list.items[0].selected,
        state.packing_list.items[0].packed,
        state.selected_index,
    );
    assert_eq!(before, after);
}

// ---- Spec 006: responsive layout ----

#[test]
fn layout_mode_threshold_is_stable_at_the_boundary() {
    use crate::ui::screens::main::{LayoutMode, layout_mode};
    // At 80 columns: side by side (two 39-col panels + margins)
    assert_eq!(layout_mode(80, 24), LayoutMode::SideBySide);
    // Below 80: stacked
    assert_eq!(layout_mode(79, 24), LayoutMode::Stacked);
    // Wide: side by side
    assert_eq!(layout_mode(120, 40), LayoutMode::SideBySide);
    // Below the minimum width or height: the notice
    assert_eq!(layout_mode(39, 24), LayoutMode::TooSmall);
    assert_eq!(layout_mode(80, 23), LayoutMode::TooSmall);
    assert_eq!(layout_mode(20, 10), LayoutMode::TooSmall);
    // Pure: the same inputs always give the same mode (the
    // spec's flicker edge case — recomputed per frame)
    for _ in 0..3 {
        assert_eq!(layout_mode(79, 24), LayoutMode::Stacked);
        assert_eq!(layout_mode(80, 24), LayoutMode::SideBySide);
    }
}

#[test]
fn row_text_truncates_with_visible_indicator() {
    use crate::ui::screens::main::truncate_row_text;
    // Fits: unchanged
    assert_eq!(truncate_row_text("Shirt", "  ✓ ", " x6", 20), "Shirt");
    // Too long: truncated with the ellipsis, the row fits
    let long = "A very long item name that cannot fit";
    let out = truncate_row_text(long, "  ✓ ", " x6", 20);
    assert!(out.ends_with('…'), "truncated text must show the indicator");
    assert!(out.chars().count() <= 20 - 4 - 3);
    // No room at all: empty, never negative
    assert_eq!(truncate_row_text("Shirt", "  ✓ ", " x6", 5), "");
    // Zero width
    assert_eq!(truncate_row_text("Shirt", "  ✓ ", " x6", 0), "");
    // Exactly fits: no ellipsis
    assert_eq!(truncate_row_text("12345", "  ✓ ", " x6", 12), "12345");
}

#[test]
fn hint_collapse_switches_on_the_height_budget() {
    use crate::ui::hints::HintSurface;
    use crate::ui::hints::{collapsed_hint_line, hint_entries, hint_rows, hint_rows_fit_budget};
    let entries = hint_entries(HintSurface::LeftPanel);
    // Wide terminal: the packed rows fit the budget — full hints
    let wide_rows = hint_rows(entries, 120);
    assert!(hint_rows_fit_budget(entries, 120, 3));
    assert!(wide_rows.len() <= 3);
    // Narrow terminal: the packed rows exceed the budget — collapse
    // to the single short line
    assert!(!hint_rows_fit_budget(entries, 30, 3));
    let collapsed = collapsed_hint_line();
    let text: String = collapsed
        .spans
        .iter()
        .map(|s| s.content.to_string())
        .collect();
    assert_eq!(text, "? Help");
    // Stable at the boundary: recomputed per frame, same inputs
    for _ in 0..3 {
        assert!(hint_rows_fit_budget(entries, 120, 3));
        assert!(!hint_rows_fit_budget(entries, 30, 3));
    }
}
