use anyhow::Result;
use std::path::PathBuf;

use crate::config::modifiers::ModifiersConfig;
use crate::models::item::{Item, QuantityBuffer, category_layers};
use crate::models::packing_list::PackingList;
use crate::models::weekdays::Weekdays;
use crate::quantity::calculation::QuantityCalculation;
use crate::quantity::model::{ItemQuantitySpec, QuantityCalculationInput};

#[derive(Debug, Clone, Copy, PartialEq)]
/// The screen currently shown.
pub(crate) enum Screen {
    Welcome,
    Main,
    Help,
    Config,
    /// Raw editor for the config file
    ConfigEditor,
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// The main screen panel the cursor navigates.
pub(crate) enum FocusSide {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// The welcome screen field under focus.
pub(crate) enum Focus {
    TripFile,
    Weekdays,
    TravelStyle,
    Weather,
}

impl Focus {
    /// The next field in the welcome navigation cycle.
    pub(crate) fn next(&self) -> Self {
        match self {
            Focus::TripFile => Focus::Weekdays,
            Focus::Weekdays => Focus::TravelStyle,
            Focus::TravelStyle => Focus::Weather,
            Focus::Weather => Focus::TripFile,
        }
    }

    /// The previous field in the welcome navigation cycle.
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
/// A field of the add/edit item dialog.
pub(crate) enum InputField {
    Name,
    Category,
    Quantity,
    Mode,
    /// The per-day rate (spec 010)
    Rate,
    /// The buffer kind and value (spec 010)
    Buffer,
}

impl InputField {
    /// The next field in the dialog's cycle (wrapping).
    pub(crate) fn next(&self) -> Self {
        match self {
            InputField::Name => InputField::Category,
            InputField::Category => InputField::Quantity,
            InputField::Quantity => InputField::Mode,
            InputField::Mode => InputField::Rate,
            InputField::Rate => InputField::Buffer,
            InputField::Buffer => InputField::Name,
        }
    }

    /// The previous field in the dialog's cycle (wrapping).
    pub(crate) fn prev(&self) -> Self {
        // The predecessor of self: the field whose next() is self.
        // (Not self.next().cycle_back() — that computes the
        // predecessor of the successor, which is self again.)
        let mut f = *self;
        for _ in 0..6 {
            f = f.next();
            if f.next() == *self {
                return f;
            }
        }
        *self
    }
}

/// The buffer kind being edited in the dialog (spec 010); cycles
/// None -> Reserve -> Multiplier. Setting one kind clears the other's
/// value (FR-011) — the exclusivity is enforced by the single
/// `buffer_value` slot.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum QuantityBufferKind {
    None,
    Reserve,
    Multiplier,
}

impl QuantityBufferKind {
    /// The next kind in the cycle (wraps).
    pub(crate) fn next(&self) -> Self {
        match self {
            QuantityBufferKind::None => QuantityBufferKind::Reserve,
            QuantityBufferKind::Reserve => QuantityBufferKind::Multiplier,
            QuantityBufferKind::Multiplier => QuantityBufferKind::None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
/// The main screen's input mode: normal navigation or one of the
/// modal dialogs (add, edit, delete confirmation).
pub(crate) enum InputMode {
    Normal,
    AddingItem {
        field: InputField,
        name: String,
        category: String,
        quantity: String,
        category_list: Vec<String>,
        category_index: usize,
        /// The per-day rate as dialog text (spec 010)
        rate: String,
        /// The buffer kind and value as dialog text (spec 010):
        /// the kind cycles None -> Reserve -> Multiplier; the value
        /// is the reserve (integer) or the multiplier (decimal)
        buffer_kind: QuantityBufferKind,
        buffer_value: String,
        /// The quantity mode for the new item (spec 021): automatic
        /// (default) or fixed — Space toggles, like the edit dialog
        auto_quantity: bool,
    },
    EditingItem {
        index: usize,
        field: InputField,
        name: String,
        category: String,
        quantity: String,
        /// The per-day rate as dialog text (spec 010)
        rate: String,
        /// The buffer kind and value as dialog text (spec 010)
        buffer_kind: QuantityBufferKind,
        buffer_value: String,
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
    /// The edit dialog's save routes here (spec 011): the dialog's
    /// state is consumed into the snapshot; Enter applies it, Esc/N
    /// discards it with nothing written.
    ConfirmEdit {
        pending: PendingEdit,
    },
}

/// The edit dialog's values, snapshotted for the confirmation
/// (spec 011): applying is the only effect; cancelling drops this.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PendingEdit {
    pub(crate) index: usize,
    pub(crate) name: String,
    pub(crate) categories: Vec<String>,
    pub(crate) quantity: String,
    pub(crate) auto_quantity: bool,
    /// The per-day rate as dialog text (spec 010)
    pub(crate) rate: String,
    /// The buffer kind and value as dialog text (spec 010)
    pub(crate) buffer_kind: QuantityBufferKind,
    pub(crate) buffer_value: String,
}

impl InputMode {
    /// The shared dialog surface (spec 021, US3 parity): both dialog
    /// variants expose the same fields to the one renderer. Returns
    /// `None` outside the add/edit dialogs.
    pub(crate) fn popup_fields(&self) -> Option<PopupFields<'_>> {
        match self {
            InputMode::AddingItem {
                field,
                name,
                category,
                quantity,
                category_list,
                category_index,
                rate,
                buffer_kind,
                buffer_value,
                auto_quantity,
            } => Some(PopupFields {
                field: *field,
                name,
                category,
                quantity,
                auto_quantity: Some(*auto_quantity),
                category_list,
                category_index: *category_index,
                selected_categories: None,
                rate,
                buffer_kind: *buffer_kind,
                buffer_value,
            }),
            InputMode::EditingItem {
                field,
                name,
                category,
                quantity,
                auto_quantity,
                category_list,
                category_index,
                selected_categories,
                rate,
                buffer_kind,
                buffer_value,
                ..
            } => Some(PopupFields {
                field: *field,
                name,
                category,
                quantity,
                auto_quantity: Some(*auto_quantity),
                category_list,
                category_index: *category_index,
                selected_categories: Some(selected_categories),
                rate,
                buffer_kind: *buffer_kind,
                buffer_value,
            }),
            InputMode::Normal | InputMode::ConfirmDelete { .. } | InputMode::ConfirmEdit { .. } => {
                None
            }
        }
    }
}

/// The run loop's state: running or requested to exit.
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
    /// The edit dialog's membership set (Space multi-select); None in
    /// the add dialog, which has a single category
    pub(crate) selected_categories: Option<&'a Vec<String>>,
    /// The per-day rate as dialog text (spec 010)
    pub(crate) rate: &'a str,
    /// The buffer kind being edited (spec 010)
    pub(crate) buffer_kind: QuantityBufferKind,
    /// The buffer's value as dialog text (spec 010)
    pub(crate) buffer_value: &'a str,
}

/// A user-visible status message shown in the footer of the active
/// screen; set on failures that must not kill the session.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StatusMessage {
    pub(crate) message: String,
    pub(crate) kind: StatusKind,
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// The kind of a status message; drives the footer styling.
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
    /// Collapsed top-layer groups (spec 024): the part before the
    /// first ": ". Session state, not persisted.
    pub(crate) collapsed_groups: Vec<String>,
    pub(crate) selected_index: usize,
    pub(crate) focus_side: FocusSide,
    /// Fullscreen compiled-list view (spec 007): presentation-only;
    /// not persisted — a restarted app starts in the two-panel view
    pub(crate) fullscreen: bool,
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
/// A section of the config screen.
pub(crate) enum ConfigSection {
    TravelStyles,
    Weather,
}

impl ConfigSection {
    /// The section's display title.
    pub(crate) fn title(&self) -> &'static str {
        match self {
            ConfigSection::TravelStyles => "Travel Styles",
            ConfigSection::Weather => "Weather",
        }
    }

    /// The next section in the tab cycle.
    pub(crate) fn next(&self) -> Self {
        match self {
            ConfigSection::TravelStyles => ConfigSection::Weather,
            ConfigSection::Weather => ConfigSection::TravelStyles,
        }
    }

    /// The previous section in the tab cycle.
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
/// A field of the modifier editor popup.
pub(crate) enum ModifierField {
    Name,
    Multiplier,
}

// A navigable row of a panel: a category header or an item
#[derive(Clone, Debug)]
pub(crate) enum PanelRow {
    /// A top-layer group header (spec 024): the part before the first
    /// ": " of its categories
    Group(String),
    /// A sub-category header; the payload is the FULL category string
    /// (the identity for collapse/selection/recalculation)
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

/// The visible slice of a panel's rows with a sticky category header:
/// when the scroll window starts mid-category, the category's header
/// is pinned to the viewport's first row so no item is ever shown
/// orphaned from its group. Each entry carries the row's position in
/// the full row list (the pinned header keeps its original position,
/// above the window). Pure — recomputed per frame.
pub(crate) fn sticky_rows(
    rows: &[PanelRow],
    scroll: usize,
    viewport_height: usize,
) -> Vec<(usize, PanelRow)> {
    let mut visible: Vec<(usize, PanelRow)> = rows
        .iter()
        .enumerate()
        .skip(scroll)
        .take(viewport_height)
        .map(|(pos, r)| (pos, r.clone()))
        .collect();
    if scroll == 0 || visible.is_empty() {
        return visible;
    }
    // Pin the OUTERMOST scrolled-off header (spec 024, research R5):
    // an Item row pins its nearest off-screen header (sub-category or
    // group); a Category row pins its group when that group is also
    // off-screen. One pinned row maximum.
    let pin_target = match &visible[0].1 {
        PanelRow::Item(_) => rows[..scroll]
            .iter()
            .enumerate()
            .rev()
            .find_map(|(pos, r)| match r {
                PanelRow::Group(g) => Some((pos, PanelRow::Group(g.clone()))),
                PanelRow::Category(c) => Some((pos, PanelRow::Category(c.clone()))),
                _ => None,
            }),
        PanelRow::Category(c) => {
            // Pin only the enclosing group, and only when it is
            // off-screen above
            let (top, _) = category_layers(c);
            rows[..scroll]
                .iter()
                .enumerate()
                .rev()
                .find_map(|(pos, r)| match r {
                    PanelRow::Group(g) if *g == top => Some((pos, PanelRow::Group(g.clone()))),
                    _ => None,
                })
        }
        PanelRow::Group(_) => None,
    };
    if let Some((pos, row)) = pin_target {
        // The pin displaces the window's last row. When the window
        // already ends at the list's final row, displacing it would
        // hide the last item — the item's visibility wins; skip the
        // pin (the header context is recoverable by scrolling up).
        let window_ends_at_list_end = scroll + viewport_height >= rows.len();
        if !window_ends_at_list_end {
            visible.insert(0, (pos, row));
            visible.pop();
        }
    }
    visible
}

/// The scope of an explicit recalculate action (spec 018):
/// the current item, the current item's category, or everything.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum RecalculateScope {
    Item(usize),
    Category(String),
    /// A top-layer group (spec 024): every item whose any category's
    /// top layer matches
    Group(String),
    All,
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

    /// The left panel's rows in display order: each category header,
    /// followed by its items unless the category is collapsed.
    pub(crate) fn left_rows(&self) -> Vec<PanelRow> {
        // Rows of the left panel in display order (spec 024): each
        // top-layer group header, then its sub-category headers with
        // their items — unless the group or the sub-category is
        // collapsed. Separator-free categories render as a group with
        // their items directly beneath (no sub-category row).
        let categories_map =
            group_items_by_category(self.packing_list.items.iter().enumerate(), |item| {
                item.categories.clone()
            });

        // Group the categories by top layer, preserving sorted order
        let mut group_order: Vec<String> = Vec::new();
        let mut groups: std::collections::BTreeMap<String, Vec<&String>> =
            std::collections::BTreeMap::new();
        for category in categories_map.keys() {
            let (top, _) = category_layers(category);
            if !groups.contains_key(top) {
                group_order.push(top.to_string());
            }
            groups.entry(top.to_string()).or_default().push(category);
        }

        let mut rows = Vec::new();
        for top in &group_order {
            rows.push(PanelRow::Group(top.clone()));
            if self.collapsed_groups.contains(top) {
                continue;
            }
            for category in &groups[top] {
                let has_sub = category_layers(category).1.is_some();
                // A separator-free category has no sub-category row:
                // its items sit directly beneath the group
                let sub_collapsed = self.collapsed_categories.contains(*category);
                if has_sub {
                    rows.push(PanelRow::Category((*category).clone()));
                }
                if !sub_collapsed {
                    for (idx, _) in &categories_map[*category] {
                        rows.push(PanelRow::Item(*idx));
                    }
                }
            }
        }
        rows
    }

    /// The selected items in the order they are displayed on the right
    /// panel (grouped by primary category).
    /// The right panel's rows (spec 024): the two-layer tree over
    /// the selected items — group headers, sub-category headers,
    /// items. The cursor walks the same rows the panel renders, so
    /// every header is reachable by navigation.
    pub(crate) fn right_rows(&self) -> Vec<PanelRow> {
        let categories_map = group_items_by_category(
            self.packing_list
                .items
                .iter()
                .enumerate()
                .filter(|(_, item)| item.selected),
            |item| vec![item.primary_category().to_string()],
        );

        // Group the categories by top layer, preserving sorted order
        let mut group_order: Vec<String> = Vec::new();
        let mut groups: std::collections::BTreeMap<String, Vec<&String>> =
            std::collections::BTreeMap::new();
        for category in categories_map.keys() {
            let (top, _) = category_layers(category);
            if !groups.contains_key(top) {
                group_order.push(top.to_string());
            }
            groups.entry(top.to_string()).or_default().push(category);
        }

        let mut rows = Vec::new();
        for top in &group_order {
            rows.push(PanelRow::Group(top.clone()));
            for category in &groups[top] {
                let has_sub = category_layers(category).1.is_some();
                if has_sub {
                    rows.push(PanelRow::Category((*category).clone()));
                }
                for (idx, _) in &categories_map[*category] {
                    rows.push(PanelRow::Item(*idx));
                }
            }
        }
        rows
    }

    // Rows the cursor moves over on the focused side: the same tree
    // the focused panel renders (spec 024 — every header reachable)
    pub(crate) fn active_rows(&self) -> Vec<PanelRow> {
        match self.focus_side {
            FocusSide::Left => self.left_rows(),
            FocusSide::Right => self.right_rows(),
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

    // The category the cursor is on (header row or one of its items).
    // A group header resolves to the group's name (the top layer).
    pub(crate) fn cursor_category(&self) -> Option<String> {
        let rows = self.active_rows();
        match rows.get(self.selected_index) {
            Some(PanelRow::Group(g)) => Some(g.clone()),
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
    /// The recalculate scope for the cursor's row (spec 024): a Group
    /// row resolves to the group scope; a Category row to its
    /// full-string category; an item to its primary category.
    pub(crate) fn cursor_recalc_scope(&self) -> Option<RecalculateScope> {
        let rows = self.active_rows();
        match rows.get(self.selected_index) {
            Some(PanelRow::Group(g)) => Some(RecalculateScope::Group(g.clone())),
            Some(PanelRow::Category(c)) => Some(RecalculateScope::Category(c.clone())),
            Some(PanelRow::Item(index)) => Some(RecalculateScope::Item(*index)),
            None => None,
        }
    }

    // Collapse (true) or expand (false) the header under the cursor,
    // resolving by row (spec 024): a Group row toggles the group
    // (hiding its whole sub-tree); a Category row toggles the
    // sub-category (hiding its items only); an item resolves to its
    // primary category's sub-category. When collapsing, the cursor
    // moves to the header it collapsed so it stays visible.
    pub(crate) fn set_category_collapsed(&mut self, collapsed: bool) {
        let rows = self.active_rows();
        let Some(row) = rows.get(self.selected_index) else {
            return;
        };
        // The header this action targets: (name, is_group)
        let (target, is_group) = match row {
            PanelRow::Group(g) => (g.clone(), true),
            PanelRow::Category(c) => (c.clone(), false),
            PanelRow::Item(index) => (
                self.packing_list.items[*index]
                    .primary_category()
                    .to_string(),
                false,
            ),
        };

        if is_group {
            // A group whose categories are all separator-free has no
            // sub-level: collapsing it collapses each category's
            // items (the group row is their only header, spec 024)
            let group_categories: Vec<String> = self
                .packing_list
                .items
                .iter()
                .flat_map(|item| item.categories.iter().cloned())
                .filter(|c| category_layers(c).0 == target)
                .collect();
            let has_sub = group_categories
                .iter()
                .any(|c| category_layers(c).1.is_some());
            if !has_sub {
                if collapsed {
                    for category in &group_categories {
                        if !self.collapsed_categories.contains(category) {
                            self.collapsed_categories.push(category.clone());
                        }
                    }
                    // The cursor stays on the group's header row
                    let rows = self.active_rows();
                    if let Some(row) = rows
                        .iter()
                        .position(|r| matches!(r, PanelRow::Group(g) if *g == target))
                    {
                        self.selected_index = row;
                    }
                } else {
                    for category in &group_categories {
                        self.collapsed_categories.retain(|c| c != category);
                    }
                }
                return;
            }
            if collapsed {
                if !self.collapsed_groups.contains(&target) {
                    self.collapsed_groups.push(target.clone());
                }
                // The cursor stays on the group's header row
                let rows = self.active_rows();
                if let Some(row) = rows
                    .iter()
                    .position(|r| matches!(r, PanelRow::Group(g) if *g == target))
                {
                    self.selected_index = row;
                }
            } else {
                self.collapsed_groups.retain(|g| g != &target);
            }
            return;
        }

        if collapsed {
            if !self.collapsed_categories.contains(&target) {
                self.collapsed_categories.push(target.clone());
            }
            // Move the cursor to the category's header row: the
            // sub-category row when one exists, otherwise the group
            // header (separator-free categories have no sub-row)
            let rows = self.active_rows();
            let header_row = rows
                .iter()
                .position(|r| matches!(r, PanelRow::Category(c) if *c == target))
                .or_else(|| {
                    let (top, _) = category_layers(&target);
                    rows.iter()
                        .position(|r| matches!(r, PanelRow::Group(g) if *g == top))
                });
            if let Some(row) = header_row {
                self.selected_index = row;
            }
        } else {
            self.collapsed_categories.retain(|c| c != &target);
        }
    }

    /// Toggle collapse of the header under the cursor (spec 024):
    /// the group level on a Group row, the sub-category level on a
    /// Category row or an item. A group whose categories are all
    /// separator-free collapses at the category level (its group
    /// row is the only header those categories have).
    pub(crate) fn toggle_category_collapse(&mut self) {
        let rows = self.active_rows();
        let Some(row) = rows.get(self.selected_index) else {
            return;
        };
        let collapsed = match row {
            PanelRow::Group(g) => {
                // Separator-free group: the category level is the
                // effective collapse state
                let has_sub = self.packing_list.items.iter().any(|item| {
                    item.categories
                        .iter()
                        .any(|c| category_layers(c).0 == *g && category_layers(c).1.is_some())
                });
                if has_sub {
                    self.collapsed_groups.contains(g)
                } else {
                    self.packing_list.items.iter().any(|item| {
                        let primary = item.primary_category();
                        category_layers(primary).0 == *g
                            && self.collapsed_categories.iter().any(|c| c == primary)
                    })
                }
            }
            PanelRow::Category(c) => self.collapsed_categories.contains(c),
            PanelRow::Item(index) => {
                let primary = self.packing_list.items[*index].primary_category();
                self.collapsed_categories.iter().any(|c| c == primary)
            }
        };
        self.set_category_collapsed(!collapsed);
    }

    /// Toggle selection of the item under the cursor.
    pub(crate) fn toggle_item_selection(&mut self) {
        // Toggle selection of the currently selected item
        if let Some(item_index) = self.cursor_item_index() {
            let item = &mut self.packing_list.items[item_index];
            item.selected = !item.selected;
        }
    }

    /// Toggle selection of every item in the cursor's category, then
    /// collapse the group when added and expand it when removed.
    pub(crate) fn toggle_category_selection(&mut self) {
        // Toggle selection of the header under the cursor, resolving
        // by row (spec 024): a Group row flips every item whose any
        // category's top layer matches; a Category row (or an item)
        // flips the full-string category's items. The group collapses
        // when items were added to the trip and expands when removed.
        let rows = self.active_rows();
        let Some(row) = rows.get(self.selected_index) else {
            return;
        };
        let (group, category) = match row {
            PanelRow::Group(g) => (Some(g.clone()), None),
            PanelRow::Category(c) => (None, Some(c.clone())),
            PanelRow::Item(index) => (
                None,
                Some(
                    self.packing_list.items[*index]
                        .primary_category()
                        .to_string(),
                ),
            ),
        };

        let matches = |item: &Item| match (&group, &category) {
            (Some(top), _) => item.categories.iter().any(|c| category_layers(c).0 == *top),
            (None, Some(c)) => item.in_category(c),
            (None, None) => false,
        };

        let was_selected = self
            .packing_list
            .items
            .iter()
            .any(|item| matches(item) && item.selected);

        for item in &mut self.packing_list.items {
            if matches(item) {
                item.selected = !item.selected;
            }
        }

        // Follow the selection state: added -> collapse, removed -> expand
        let now_selected = !was_selected;
        self.set_category_collapsed(!now_selected);
    }

    /// Toggle the packed state of the item under the cursor (right
    /// panel only).
    pub(crate) fn toggle_item_packed(&mut self) {
        // Only toggle packed status when on right side
        if self.focus_side == FocusSide::Right
            && let Some(item_index) = self.cursor_item_index()
        {
            self.packing_list.items[item_index].packed =
                !self.packing_list.items[item_index].packed;
        }
    }

    /// Toggle the quantity mode (automatic/fixed) in the add or edit
    /// dialog (spec 021: both dialogs share the Mode field).
    pub(crate) fn toggle_edit_mode(&mut self) {
        match &mut self.input_mode {
            InputMode::EditingItem { auto_quantity, .. }
            | InputMode::AddingItem { auto_quantity, .. } => {
                *auto_quantity = !*auto_quantity;
            }
            _ => {}
        }
    }

    // Cycle the category selection in the add/edit dialog (Left/Right);
    // clamped at the list bounds, only when the Category field is focused
    pub(crate) fn cycle_category_selection(&mut self, direction: i32) {
        let (field, category_list, category_index, selected_categories) = match &mut self.input_mode
        {
            InputMode::AddingItem {
                field,
                category_list,
                category_index,
                ..
            } => (field, category_list, category_index, None),
            InputMode::EditingItem {
                field,
                category_list,
                category_index,
                selected_categories,
                ..
            } => (
                field,
                category_list,
                category_index,
                Some(selected_categories),
            ),
            _ => return,
        };
        if *field != InputField::Category {
            return;
        }
        let target = (*category_index as i32 + direction)
            .clamp(0, category_list.len().saturating_sub(1) as i32);
        *category_index = target as usize;
        // The highlighted category becomes the selection (spec 014:
        // Left/Right adjust the value): the item's category follows
        // the highlight, so the change persists on save. In the edit
        // dialog the membership set follows too — cycling replaces
        // the memberships with the highlighted category.
        if let Some(selected) = selected_categories
            && let Some(highlighted) = category_list.get(*category_index)
        {
            *selected = vec![highlighted.clone()];
        }
    }

    /// Step the Quantity field's value in the add/edit dialog
    /// (spec 020, US3): Left/Right adjust the parsed value by ±1,
    /// clamped at 1 (empty or unparsable counts as 1); typing still
    /// appends. Only acts when the Quantity field is focused.
    pub(crate) fn step_dialog_quantity(&mut self, direction: i32) {
        let quantity = match &mut self.input_mode {
            InputMode::AddingItem {
                field: InputField::Quantity,
                quantity,
                ..
            }
            | InputMode::EditingItem {
                field: InputField::Quantity,
                quantity,
                ..
            } => quantity,
            _ => return,
        };
        let current = quantity.parse::<u32>().unwrap_or(1);
        let stepped = (current as i32 + direction).max(1) as u32;
        *quantity = stepped.to_string();
    }

    /// Recalculate every automatic item's quantity from the current
    /// trip parameters and per-item configuration. Fixed items keep
    /// their values; packed, selection, and collapse states are
    /// untouched; nothing is written to disk. Idempotent for
    /// unchanged inputs.
    pub(crate) fn recalculate_quantities(&mut self, calc: &QuantityCalculation) -> Result<()> {
        self.recalculate_quantities_scoped(calc, RecalculateScope::All)
    }

    /// Recalculate the automatic items in the given scope (spec 018).
    /// Items outside the scope — and fixed-mode items inside it —
    /// keep their quantities exactly as they are.
    pub(crate) fn recalculate_quantities_scoped(
        &mut self,
        calc: &QuantityCalculation,
        scope: RecalculateScope,
    ) -> Result<()> {
        // Resolve multipliers from the config; fall back to 1.0 for unknown names
        let style_multiplier = self
            .modifiers
            .travel_style_multiplier(&self.travel_style)
            .unwrap_or(1.0);
        let weather_multiplier = self
            .modifiers
            .weather_multiplier(&self.weather)
            .unwrap_or(1.0);

        // Items outside the scope pass as Fixed(current) so the
        // calculation leaves them exactly as they are; fixed-mode
        // items inside the scope also pass as Fixed (FR-004)
        let item_specs: Vec<ItemQuantitySpec> = self
            .packing_list
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let in_scope = match &scope {
                    RecalculateScope::All => true,
                    RecalculateScope::Item(i) => *i == index,
                    RecalculateScope::Category(c) => item.in_category(c),
                    RecalculateScope::Group(top) => {
                        item.categories.iter().any(|c| category_layers(c).0 == *top)
                    }
                };
                if item.auto_quantity && in_scope {
                    ItemQuantitySpec::Auto {
                        rate: item.rate,
                        buffer: item.buffer,
                    }
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
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn apply_item_edit(
        &mut self,
        index: usize,
        name: String,
        categories: Vec<String>,
        quantity: String,
        auto_quantity: Option<bool>,
        rate: String,
        buffer_kind: QuantityBufferKind,
        buffer_value: String,
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
        // Per-item modifiers (spec 010): an empty rate keeps the old
        // value; a rate outside 0.05..=10.0 is rejected (FR-006)
        if !rate.is_empty()
            && let Ok(parsed) = rate.parse::<f32>()
            && (0.05..=10.0).contains(&parsed)
        {
            item.rate = parsed;
        }
        // The buffer: the kind decides the value's meaning; an invalid
        // or out-of-range value keeps the old buffer (FR-006). The
        // single value slot enforces the exclusivity (FR-011).
        match buffer_kind {
            QuantityBufferKind::None => item.buffer = QuantityBuffer::None,
            QuantityBufferKind::Reserve => {
                if let Ok(n) = buffer_value.parse::<u32>()
                    && n <= 10
                {
                    item.buffer = QuantityBuffer::Reserve(n);
                }
            }
            QuantityBufferKind::Multiplier => {
                if let Ok(m) = buffer_value.parse::<f32>()
                    && (1.0..=3.0).contains(&m)
                {
                    item.buffer = QuantityBuffer::Multiplier(m);
                }
            }
        }
    }

    // Build the item described by the add dialog and append it to the
    // list (spec 021): the typed quantity is the new item's quantity
    // (empty or unparsable -> 1), the dialog's mode is the item's mode,
    // and the per-item modifiers are validated like the edit path
    // (FR-006). The cursor moves to the new item, clamped to the
    // visible rows.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn add_item_from_dialog(
        &mut self,
        name: String,
        category: String,
        quantity: String,
        auto_quantity: bool,
        rate: String,
        buffer_kind: QuantityBufferKind,
        buffer_value: String,
    ) {
        let qty = quantity.parse::<u32>().unwrap_or(1);
        let new_name = if name.is_empty() {
            "New Item".to_string()
        } else {
            name
        };
        let new_category = if category.is_empty() {
            // Use first category or default
            if !self.packing_list.items.is_empty() {
                self.packing_list.items[0].primary_category().to_string()
            } else {
                "Other".to_string()
            }
        } else {
            category
        };

        let mut item = Item::new(new_name, vec![new_category], qty);
        // The dialog's mode decides what the typed quantity means
        // (spec 021): fixed keeps it forever; automatic starts there
        item.auto_quantity = auto_quantity;
        // Per-item modifiers (spec 010): validated like the edit path
        // (FR-006); the single value slot enforces exclusivity (FR-011)
        if !rate.is_empty()
            && let Ok(parsed) = rate.parse::<f32>()
            && (0.05..=10.0).contains(&parsed)
        {
            item.rate = parsed;
        }
        match buffer_kind {
            QuantityBufferKind::None => {}
            QuantityBufferKind::Reserve => {
                if let Ok(n) = buffer_value.parse::<u32>()
                    && n <= 10
                {
                    item.buffer = QuantityBuffer::Reserve(n);
                }
            }
            QuantityBufferKind::Multiplier => {
                if let Ok(m) = buffer_value.parse::<f32>()
                    && (1.0..=3.0).contains(&m)
                {
                    item.buffer = QuantityBuffer::Multiplier(m);
                }
            }
        }
        self.packing_list.add_item(item);

        // Reset selection to the new item, clamped to the visible rows
        let rows = self.active_rows();
        self.selected_index = (self.packing_list.items.len() - 1).min(rows.len().saturating_sub(1));
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
                if matches!(self.input_mode, InputMode::Normal)
                    // Navigate up through the visible rows of the focused side
                    && self.selected_index > 0
                {
                    self.selected_index -= 1;
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
                // Navigate down through the visible rows of the focused side
                if matches!(self.input_mode, InputMode::Normal) {
                    let rows = self.active_rows();
                    if self.selected_index < rows.len().saturating_sub(1) {
                        self.selected_index += 1;
                    }
                }
            }
        }
    }
}
