use anyhow::Result;
use std::path::PathBuf;

use crate::config::modifiers::ModifiersConfig;
use crate::models::item::Item;
use crate::models::packing_list::PackingList;
use crate::models::weekdays::Weekdays;
use crate::quantity::calculation::QuantityCalculation;
use crate::quantity::model::{ItemQuantitySpec, QuantityCalculationInput};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Screen {
    Welcome,
    Main,
    Help,
    Config,
    /// Raw editor for the config file
    ConfigEditor,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum FocusSide {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Focus {
    TripFile,
    Weekdays,
    TravelStyle,
    Weather,
}

impl Focus {
    pub(crate) fn next(&self) -> Self {
        match self {
            Focus::TripFile => Focus::Weekdays,
            Focus::Weekdays => Focus::TravelStyle,
            Focus::TravelStyle => Focus::Weather,
            Focus::Weather => Focus::TripFile,
        }
    }

    pub(crate) fn prev(&self) -> Self {
        match self {
            Focus::TripFile => Focus::Weather,
            Focus::Weekdays => Focus::TripFile,
            Focus::TravelStyle => Focus::Weekdays,
            Focus::Weather => Focus::TravelStyle,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum InputField {
    Name,
    Category,
    Quantity,
    Mode,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum InputMode {
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

pub(crate) enum AppState {
    Running,
    Exit,
}

/// Editing state shared by the add-item and edit-item popups, bundled so the
/// popup draw functions take one argument instead of eight.
pub(crate) struct PopupFields<'a> {
    pub(crate) field: InputField,
    pub(crate) name: &'a str,
    pub(crate) category: &'a str,
    pub(crate) quantity: &'a str,
    /// Quantity mode shown in the edit dialog's Mode field
    pub(crate) auto_quantity: Option<bool>,
    pub(crate) category_list: &'a [String],
    pub(crate) category_index: usize,
}

/// A user-visible status message shown in the footer of the active
/// screen; set on failures that must not kill the session.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StatusMessage {
    pub(crate) message: String,
    pub(crate) kind: StatusKind,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum StatusKind {
    Error,
}

// State for drawing (cloneable)
#[derive(Clone)]
pub(crate) struct AppDrawState {
    pub(crate) screen: Screen,
    pub(crate) packing_list: PackingList,
    pub(crate) weekdays: Weekdays,
    pub(crate) travel_style: String,
    pub(crate) weather: String,
    pub(crate) modifiers: ModifiersConfig,
    pub(crate) focus: Focus,
    pub(crate) collapsed_categories: Vec<String>,
    pub(crate) selected_index: usize,
    pub(crate) focus_side: FocusSide,
    pub(crate) input_mode: InputMode,
    pub(crate) left_scroll: usize,
    pub(crate) right_scroll: usize,
    /// Path of the packing list file to persist (where the trip is saved on exit)
    pub(crate) trip_file_path: PathBuf,
    /// Text input for the trip file path on the welcome screen
    pub(crate) trip_file_input: String,
    /// Whether the trip file name is being edited
    pub(crate) editing_trip_file: bool,
    /// Whether the user is being asked to resume the previous trip file
    pub(crate) resume_prompt: bool,
    /// Trip file remembered from the previous session, offered for resume
    pub(crate) remembered_trip_file: Option<PathBuf>,
    /// Config screen state
    pub(crate) config_section: ConfigSection,
    pub(crate) config_cursor: usize,
    pub(crate) config_scroll: usize,
    /// Editor for adding/editing a modifier on the config screen
    pub(crate) modifier_editor: Option<ModifierEditor>,
    /// Screen to return to when leaving Config/ConfigEditor/Help
    pub(crate) previous_screen: Screen,
    /// Raw text of the config file being edited in the ConfigEditor screen
    pub(crate) config_editor_text: Vec<String>,
    /// Cursor (line, column) of the raw config editor
    pub(crate) config_editor_cursor: (usize, usize),
    /// Scroll offset of the raw config editor
    pub(crate) config_editor_scroll: usize,
    /// Error message shown in the raw config editor after a failed save
    pub(crate) config_editor_error: Option<String>,
    /// Status message shown in the footer of the active screen
    pub(crate) status: Option<StatusMessage>,
}

/// Sections of the config screen
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ConfigSection {
    TravelStyles,
    Weather,
}

impl ConfigSection {
    pub(crate) fn title(&self) -> &'static str {
        match self {
            ConfigSection::TravelStyles => "Travel Styles",
            ConfigSection::Weather => "Weather",
        }
    }

    pub(crate) fn next(&self) -> Self {
        match self {
            ConfigSection::TravelStyles => ConfigSection::Weather,
            ConfigSection::Weather => ConfigSection::TravelStyles,
        }
    }

    pub(crate) fn prev(&self) -> Self {
        match self {
            ConfigSection::TravelStyles => ConfigSection::Weather,
            ConfigSection::Weather => ConfigSection::TravelStyles,
        }
    }
}

/// In-progress modifier being added or edited on the config screen
#[derive(Debug, Clone)]
pub(crate) struct ModifierEditor {
    /// Index into the section's list; None when adding a new modifier
    pub(crate) index: Option<usize>,
    pub(crate) section: ConfigSection,
    pub(crate) name: String,
    pub(crate) multiplier: String,
    pub(crate) field: ModifierField,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ModifierField {
    Name,
    Multiplier,
}

// A navigable row of a panel: a category header or an item
#[derive(Clone, Debug)]
pub(crate) enum PanelRow {
    Category(String),
    Item(usize), // index into packing_list.items
}

/// Group the given items by category, in sorted category order. Each
/// entry carries the item's index and a reference to the item.
pub(crate) fn group_items_by_category<'a>(
    items: impl Iterator<Item = (usize, &'a Item)>,
    category_of: impl Fn(&'a Item) -> Vec<String>,
) -> std::collections::BTreeMap<String, Vec<(usize, &'a Item)>> {
    let mut map: std::collections::BTreeMap<String, Vec<(usize, &'a Item)>> =
        std::collections::BTreeMap::new();
    for (idx, item) in items {
        for category in category_of(item) {
            map.entry(category).or_default().push((idx, item));
        }
    }
    map
}

impl AppDrawState {
    /// Set the footer status message.
    pub(crate) fn set_status(&mut self, kind: StatusKind, message: impl Into<String>) {
        self.status = Some(StatusMessage {
            message: message.into(),
            kind,
        });
    }

    /// Clear the footer status message.
    pub(crate) fn clear_status(&mut self) {
        self.status = None;
    }

    pub(crate) fn left_rows(&self) -> Vec<PanelRow> {
        // Rows of the left panel in display order: each category header,
        // followed by its items unless the category is collapsed
        let categories_map =
            group_items_by_category(self.packing_list.items.iter().enumerate(), |item| {
                item.categories.clone()
            });

        let mut rows = Vec::new();
        for (category, items) in &categories_map {
            rows.push(PanelRow::Category(category.clone()));
            if !self.collapsed_categories.contains(category) {
                for (idx, _) in items {
                    rows.push(PanelRow::Item(*idx));
                }
            }
        }
        rows
    }

    pub(crate) fn right_item_order(&self) -> Vec<usize> {
        // Selected items in the order they are displayed on the right
        // panel (grouped by category)
        let categories_map = group_items_by_category(
            self.packing_list
                .items
                .iter()
                .enumerate()
                .filter(|(_, item)| item.selected),
            |item| vec![item.primary_category().to_string()],
        );

        let mut order = Vec::new();
        for items in categories_map.values() {
            order.extend(items.iter().map(|(idx, _)| *idx));
        }
        order
    }

    // Rows the cursor moves over on the focused side
    pub(crate) fn active_rows(&self) -> Vec<PanelRow> {
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
    pub(crate) fn cursor_item_index(&self) -> Option<usize> {
        let rows = self.active_rows();
        match rows.get(self.selected_index) {
            Some(PanelRow::Item(index)) => Some(*index),
            _ => None,
        }
    }

    // The category the cursor is on (header row or one of its items)
    pub(crate) fn cursor_category(&self) -> Option<String> {
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
    pub(crate) fn set_category_collapsed(&mut self, collapsed: bool) {
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

    pub(crate) fn toggle_category_collapse(&mut self) {
        // Toggle collapse of the category the cursor is on
        let collapsed = self
            .cursor_category()
            .map(|c| self.collapsed_categories.contains(&c))
            .unwrap_or(false);
        self.set_category_collapsed(!collapsed);
    }

    pub(crate) fn toggle_item_selection(&mut self) {
        // Toggle selection of the currently selected item
        if let Some(item_index) = self.cursor_item_index() {
            let item = &mut self.packing_list.items[item_index];
            item.selected = !item.selected;
        }
    }

    pub(crate) fn toggle_category_selection(&mut self) {
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
            .any(|item| item.in_category(&category) && item.selected);

        for item in &mut self.packing_list.items {
            if item.in_category(&category) {
                item.selected = !item.selected;
            }
        }

        // Follow the selection state: added -> collapse, removed -> expand
        let now_selected = !was_selected;
        self.set_category_collapsed(!now_selected);
    }

    pub(crate) fn toggle_item_packed(&mut self) {
        // Only toggle packed status when on right side
        if self.focus_side == FocusSide::Right
            && let Some(item_index) = self.cursor_item_index()
        {
            self.packing_list.items[item_index].packed =
                !self.packing_list.items[item_index].packed;
        }
    }

    // Toggle the quantity mode (automatic/fixed) in the edit dialog
    pub(crate) fn toggle_edit_mode(&mut self) {
        if let InputMode::EditingItem { auto_quantity, .. } = &mut self.input_mode {
            *auto_quantity = !*auto_quantity;
        }
    }

    // Cycle the category selection in the add/edit dialog (Left/Right);
    // clamped at the list bounds, only when the Category field is focused
    pub(crate) fn cycle_category_selection(&mut self, direction: i32) {
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
    pub(crate) fn recalculate_quantities(&mut self, calc: &QuantityCalculation) -> Result<()> {
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
    pub(crate) fn apply_item_edit(
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
    pub(crate) fn move_cursor_up(&mut self) {
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
    pub(crate) fn move_cursor_down(&mut self) {
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
