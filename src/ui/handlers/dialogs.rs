use crate::models::item::Item;
use crate::ui::input::App;
use crate::ui::state::{InputField, InputMode};

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
                self.save_new_item(name, selected_category, quantity);
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
                self.save_edited_item(index, name, categories, quantity, auto_quantity);
            }
            _ => {}
        }
    }

    /// Cycle the dialog's focused field: Name -> Category -> Quantity -> Mode.
    pub(crate) fn cycle_dialog_field(&mut self) {
        fn next(field: InputField) -> InputField {
            match field {
                InputField::Name => InputField::Category,
                InputField::Category => InputField::Quantity,
                InputField::Quantity => InputField::Mode,
                InputField::Mode => InputField::Name,
            }
        }
        match &mut self.draw_state.input_mode {
            InputMode::AddingItem { field, .. } | InputMode::EditingItem { field, .. } => {
                *field = next(*field);
            }
            _ => {}
        }
    }

    /// Space in the add/edit dialog: toggle the quantity mode (Mode
    /// field) or the membership of the highlighted category (Category
    /// field).
    pub(crate) fn handle_dialog_space(&mut self) {
        if let InputMode::EditingItem {
            field,
            category_list,
            category_index,
            selected_categories,
            ..
        } = &mut self.draw_state.input_mode
        {
            match field {
                InputField::Mode => self.draw_state.toggle_edit_mode(),
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
            }
        }
    }

    /// A character typed into the add/edit dialog's focused text field.
    pub(crate) fn handle_dialog_char(&mut self, c: char) {
        match &mut self.draw_state.input_mode {
            InputMode::AddingItem {
                field,
                name,
                category,
                quantity,
                ..
            }
            | InputMode::EditingItem {
                field,
                name,
                category,
                quantity,
                ..
            } => match field {
                InputField::Name => name.push(c),
                InputField::Category => category.push(c),
                InputField::Quantity => quantity.push(c),
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
                ..
            }
            | InputMode::EditingItem {
                field,
                name,
                category,
                quantity,
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
                InputField::Mode => {}
            },
            _ => {}
        }
    }

    pub(crate) fn start_editing_item(&mut self) {
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

    pub(crate) fn save_new_item(&mut self, name: String, category: String, quantity: String) {
        // Parse quantity
        let qty = quantity.parse::<u32>().unwrap_or(1);

        // Create new item
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
            .add_item(Item::new(new_name, vec![new_category], qty));

        // Update quantities for the new item
        self.update_quantities_or_report();

        // Save config after adding
        self.save_config_or_report();

        // Reset selection to the new item, clamped to the visible rows
        let rows = self.draw_state.active_rows();
        self.draw_state.selected_index =
            (self.draw_state.packing_list.items.len() - 1).min(rows.len().saturating_sub(1));
    }

    pub(crate) fn save_edited_item(
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
        self.save_config_or_report();
        // When the item stays automatic, recalculate so the quantity
        // reflects the trip parameters rather than a stale dialog value
        if was_auto && auto_quantity {
            self.update_quantities_or_report();
        }
    }

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
