use crate::models::item::QuantityBuffer;
use crate::ui::input::App;
use crate::ui::state::{InputField, InputMode, PendingEdit, QuantityBufferKind};
use anyhow::Result;

impl App {
    /// Start adding a new item with the current category preselected.
    pub(crate) fn start_adding_item(&mut self) {
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
            // New items start with the default modifiers (spec 010)
            // and the default mode: automatic (spec 021)
            rate: String::new(),
            buffer_kind: QuantityBufferKind::None,
            buffer_value: String::new(),
            auto_quantity: true,
        };
    }

    /// Save the currently open add/edit dialog and return to normal mode.
    pub(crate) fn save_open_dialog(&mut self) {
        match self.draw_state.input_mode.clone() {
            InputMode::AddingItem {
                name,
                category,
                quantity,
                category_index,
                rate,
                buffer_kind,
                buffer_value,
                auto_quantity,
                ..
            } => {
                // Get the selected category from the list
                let categories = self.get_all_categories();
                let selected_category = if category_index < categories.len() {
                    categories[category_index].clone()
                } else if !categories.is_empty() {
                    categories[0].clone()
                } else {
                    category
                };
                self.save_new_item(
                    name,
                    selected_category,
                    quantity,
                    rate,
                    buffer_kind,
                    buffer_value,
                    auto_quantity,
                );
            }
            InputMode::EditingItem { .. } => {
                // The edit dialog never saves directly (spec 011):
                // Enter routes through the confirmation
            }
            _ => {}
        }
    }

    /// Consume the edit dialog's state into the pending-edit
    /// snapshot and enter the confirmation mode (spec 011): the
    /// dialog's values are captured; applying happens only on
    /// confirm.
    pub(crate) fn route_edit_to_confirmation(&mut self) {
        let snapshot = match self.draw_state.input_mode.clone() {
            InputMode::EditingItem {
                index,
                name,
                category,
                quantity,
                rate,
                buffer_kind,
                buffer_value,
                auto_quantity,
                category_index,
                selected_categories,
                ..
            } => {
                // The membership set from the dialog's multi-select; fall
                // back to the single category when empty (never saved empty)
                let categories = if !selected_categories.is_empty() {
                    selected_categories
                } else {
                    let all = self.get_all_categories();
                    if category_index < all.len() {
                        vec![all[category_index].clone()]
                    } else {
                        vec![category]
                    }
                };
                PendingEdit {
                    index,
                    name,
                    categories,
                    quantity,
                    auto_quantity,
                    rate,
                    buffer_kind,
                    buffer_value,
                }
            }
            _ => return,
        };
        self.draw_state.input_mode = InputMode::ConfirmEdit { pending: snapshot };
    }

    /// Apply the confirmed pending edit and write the config
    /// (spec 011, US1/AC2): called only from the confirmation's
    /// Enter arm.
    pub(crate) fn apply_pending_edit(&mut self) -> Result<()> {
        let pending = match self.draw_state.input_mode.clone() {
            InputMode::ConfirmEdit { pending } => pending,
            _ => return Ok(()),
        };
        self.save_edited_item(
            pending.index,
            pending.name,
            pending.categories,
            pending.quantity,
            pending.auto_quantity,
            pending.rate,
            pending.buffer_kind,
            pending.buffer_value,
        );
        self.save_config()
    }

    /// Cycle the dialog's focused field:
    /// Name -> Category -> Quantity -> Mode -> Rate -> Buffer -> Name.
    pub(crate) fn cycle_dialog_field(&mut self) {
        match &mut self.draw_state.input_mode {
            InputMode::AddingItem { field, .. } | InputMode::EditingItem { field, .. } => {
                *field = field.next();
            }
            _ => {}
        }
    }

    /// Space in the add/edit dialog: toggle the quantity mode (Mode
    /// field) or the membership of the highlighted category (Category
    /// field).
    pub(crate) fn handle_dialog_space(&mut self) {
        match &mut self.draw_state.input_mode {
            InputMode::EditingItem {
                field,
                category_list,
                category_index,
                selected_categories,
                buffer_kind,
                ..
            } => match field {
                InputField::Mode => self.draw_state.toggle_edit_mode(),
                InputField::Buffer => {
                    // Space cycles the buffer kind (spec 010); the value
                    // slot serves exactly one kind (FR-011)
                    *buffer_kind = buffer_kind.next();
                }
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
            InputMode::AddingItem {
                field, buffer_kind, ..
            } => match field {
                InputField::Mode => self.draw_state.toggle_edit_mode(),
                InputField::Buffer => {
                    *buffer_kind = buffer_kind.next();
                }
                _ => {}
            },
            _ => {}
        }
    }

    /// A character typed into the add/edit dialog's focused text field.
    /// Digits and the decimal point are accepted for the numeric fields
    /// (Rate, Buffer value); validation happens on save (FR-006).
    pub(crate) fn handle_dialog_char(&mut self, c: char) {
        match &mut self.draw_state.input_mode {
            InputMode::AddingItem {
                field,
                name,
                category,
                quantity,
                rate,
                buffer_value,
                ..
            }
            | InputMode::EditingItem {
                field,
                name,
                category,
                quantity,
                rate,
                buffer_value,
                ..
            } => match field {
                InputField::Name => name.push(c),
                InputField::Category => category.push(c),
                InputField::Quantity => quantity.push(c),
                InputField::Rate => {
                    if c.is_ascii_digit() || c == '.' {
                        rate.push(c);
                    }
                }
                InputField::Buffer => {
                    if c.is_ascii_digit() || c == '.' {
                        buffer_value.push(c);
                    }
                }
                InputField::Mode => {}
            },
            _ => {}
        }
    }

    /// Backspace in the add/edit dialog's focused text field.
    pub(crate) fn handle_dialog_backspace(&mut self) {
        match &mut self.draw_state.input_mode {
            InputMode::AddingItem {
                field,
                name,
                category,
                quantity,
                rate,
                buffer_value,
                ..
            }
            | InputMode::EditingItem {
                field,
                name,
                category,
                quantity,
                rate,
                buffer_value,
                ..
            } => match field {
                InputField::Name => {
                    name.pop();
                }
                InputField::Category => {
                    category.pop();
                }
                InputField::Quantity => {
                    quantity.pop();
                }
                InputField::Rate => {
                    rate.pop();
                }
                InputField::Buffer => {
                    buffer_value.pop();
                }
                InputField::Mode => {}
            },
            _ => {}
        }
    }

    /// Open the edit dialog for the item under the cursor.
    pub(crate) fn start_editing_item(&mut self) {
        if let Some(item_index) = self.current_item_index() {
            let item = &self.draw_state.packing_list.items[item_index];
            let categories = self.get_all_categories();
            let category_index = categories
                .iter()
                .position(|c| c == item.primary_category())
                .unwrap_or(0);

            // The dialog's buffer state mirrors the item's buffer
            // (spec 010); the value slot holds the active kind's value
            let (buffer_kind, buffer_value) = match item.buffer {
                QuantityBuffer::None => (QuantityBufferKind::None, String::new()),
                QuantityBuffer::Reserve(n) => (QuantityBufferKind::Reserve, n.to_string()),
                QuantityBuffer::Multiplier(m) => {
                    (QuantityBufferKind::Multiplier, format!("{m:.2}"))
                }
            };
            self.draw_state.input_mode = InputMode::EditingItem {
                index: item_index,
                field: InputField::Name,
                name: item.name.clone(),
                category: item.primary_category().to_string(),
                quantity: item.quantity.to_string(),
                rate: format!("{:.2}", item.rate),
                buffer_kind,
                buffer_value,
                auto_quantity: item.auto_quantity,
                category_list: categories,
                category_index,
                selected_categories: item.categories.clone(),
            };
        }
    }

    /// Add the dialog's item to the list, recalculate, save, and move
    /// the cursor to the new item.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn save_new_item(
        &mut self,
        name: String,
        category: String,
        quantity: String,
        rate: String,
        buffer_kind: QuantityBufferKind,
        buffer_value: String,
        auto_quantity: bool,
    ) {
        self.draw_state.add_item_from_dialog(
            name,
            category,
            quantity,
            auto_quantity,
            rate,
            buffer_kind,
            buffer_value,
        );

        // No recalculation follows the add (spec 018 EC-004): the
        // dialog's values are the new item's values; the user
        // recalculates explicitly if they want
        self.save_config_or_report();
    }

    /// Apply the edit dialog's fields to the item, save, and
    /// recalculate when the item stays automatic.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn save_edited_item(
        &mut self,
        index: usize,
        name: String,
        categories: Vec<String>,
        quantity: String,
        auto_quantity: bool,
        rate: String,
        buffer_kind: QuantityBufferKind,
        buffer_value: String,
    ) {
        self.draw_state.apply_item_edit(
            index,
            name,
            categories,
            quantity,
            Some(auto_quantity),
            rate,
            buffer_kind,
            buffer_value,
        );
        // Save config after editing; no recalculation follows (spec
        // 018 EC-003/FR-009): the dialog's values are the item's
        // values, exactly — the user recalculates explicitly
        self.save_config_or_report();
    }

    /// Remove the confirmed item and clamp the cursor to the
    /// remaining rows.
    pub(crate) fn remove_item_confirmed(&mut self, index: usize) {
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
