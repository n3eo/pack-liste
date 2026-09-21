use anyhow::Result;
use std::path::PathBuf;

use crate::config::modifiers::ModifiersConfig;
use crate::models::item::{Item, QuantityBuffer};
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
    /// The labels multi-select + free text (spec 025)
    Labels,
    /// The singleton flag (spec 025, US5)
    Singleton,
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
            InputField::Name => InputField::Labels,
            InputField::Labels => InputField::Singleton,
            InputField::Singleton => InputField::Quantity,
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
        for _ in 0..7 {
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
        /// Free-text label entry (spec 025): typed text joins the
        /// multi-select's set on save
        label_input: String,
        quantity: String,
        /// The distinct labels (the picker's data source)
        label_list: Vec<String>,
        /// The highlighted picker entry (Left/Right cycles)
        label_index: usize,
        /// The multi-select's label set; Space toggles membership
        selected_labels: Vec<String>,
        /// The singleton flag (spec 025, US5)
        singleton: bool,
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
        /// Free-text label entry (spec 025)
        label_input: String,
        quantity: String,
        /// The per-day rate as dialog text (spec 010)
        rate: String,
        /// The buffer kind and value as dialog text (spec 010)
        buffer_kind: QuantityBufferKind,
        buffer_value: String,
        /// Quantity mode carried from the item; toggled in the dialog's
        /// Mode field and written back on save
        auto_quantity: bool,
        /// The distinct labels (the picker's data source)
        label_list: Vec<String>,
        /// The highlighted picker entry (Left/Right cycles)
        label_index: usize,
        /// The item's labels; Space toggles the highlighted entry
        selected_labels: Vec<String>,
        /// The singleton flag (spec 025, US5)
        singleton: bool,
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
    pub(crate) labels: Vec<String>,
    pub(crate) singleton: bool,
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
                label_input,
                quantity,
                label_list,
                label_index,
                selected_labels,
                singleton,
                rate,
                buffer_kind,
                buffer_value,
                auto_quantity,
            } => Some(PopupFields {
                field: *field,
                name,
                label_input,
                quantity,
                auto_quantity: Some(*auto_quantity),
                label_list,
                label_index: *label_index,
                selected_labels,
                singleton: *singleton,
                rate,
                buffer_kind: *buffer_kind,
                buffer_value,
            }),
            InputMode::EditingItem {
                field,
                name,
                label_input,
                quantity,
                auto_quantity,
                label_list,
                label_index,
                selected_labels,
                singleton,
                rate,
                buffer_kind,
                buffer_value,
                ..
            } => Some(PopupFields {
                field: *field,
                name,
                label_input,
                quantity,
                auto_quantity: Some(*auto_quantity),
                label_list,
                label_index: *label_index,
                selected_labels,
                singleton: *singleton,
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
    /// The free-text label entry (spec 025)
    pub(crate) label_input: &'a str,
    pub(crate) quantity: &'a str,
    /// Quantity mode shown in the dialog's Mode field
    pub(crate) auto_quantity: Option<bool>,
    /// The distinct labels (the picker's data source)
    pub(crate) label_list: &'a [String],
    /// The highlighted picker entry
    pub(crate) label_index: usize,
    /// The multi-select's label set (Space toggles)
    pub(crate) selected_labels: &'a [String],
    /// The singleton flag (spec 025, US5)
    pub(crate) singleton: bool,
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
    /// Collapsed labels (spec 025): session state, not persisted.
    pub(crate) collapsed_labels: Vec<String>,
    /// Per-occurrence toggle overrides for NON-singleton multi-label
    /// items (spec 025, US5): (item index, label) -> the overridden
    /// state. The effective occurrence state = the override if
    /// present, else the item's own. Session state, not persisted.
    pub(crate) occurrence_overrides: std::collections::BTreeMap<(usize, String), bool>,
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
    /// A flat label header (spec 025): one per distinct label
    Label(String),
    Item(usize), // index into packing_list.items
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
    // Pin the scrolled-off label header (spec 025): an Item row
    // pins its nearest off-screen header. One pinned row maximum.
    let pin_target = match &visible[0].1 {
        PanelRow::Item(_) => rows[..scroll]
            .iter()
            .enumerate()
            .rev()
            .find_map(|(pos, r)| match r {
                PanelRow::Label(l) => Some((pos, PanelRow::Label(l.clone()))),
                _ => None,
            }),
        PanelRow::Label(_) => None,
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
    /// A label (spec 025): every item carrying it
    Label(String),
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
        // Rows of the left panel in display order (spec 025): one
        // flat header per distinct label (sorted), the items carrying
        // it beneath — an item appears under EVERY label it carries.
        // A collapsed label contributes only its header.
        let labels: Vec<String> = self
            .packing_list
            .items
            .iter()
            .flat_map(|item| item.labels.iter().cloned())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();

        let mut rows = Vec::new();
        for label in &labels {
            rows.push(PanelRow::Label(label.clone()));
            if self.collapsed_labels.contains(label) {
                continue;
            }
            for (idx, item) in self.packing_list.items.iter().enumerate() {
                if item.has_label(label) {
                    rows.push(PanelRow::Item(idx));
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
        // The right panel's rows (spec 025): the same flat label
        // grouping over the selected items. The cursor walks the same
        // rows the panel renders, so every header is reachable.
        let labels: Vec<String> = self
            .packing_list
            .items
            .iter()
            .filter(|item| item.selected)
            .flat_map(|item| item.labels.iter().cloned())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();

        let mut rows = Vec::new();
        for label in &labels {
            rows.push(PanelRow::Label(label.clone()));
            for (idx, item) in self.packing_list.items.iter().enumerate() {
                if item.selected && item.has_label(label) {
                    rows.push(PanelRow::Item(idx));
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

    // The label the cursor is on (a header row, or the first label
    // of the item under the cursor).
    pub(crate) fn cursor_label(&self) -> Option<String> {
        let rows = self.active_rows();
        match rows.get(self.selected_index) {
            Some(PanelRow::Label(l)) => Some(l.clone()),
            Some(PanelRow::Item(index)) => self.packing_list.items[*index].labels.first().cloned(),
            None => None,
        }
    }

    // Collapse (true) or expand (false) the category under the cursor.
    // When collapsing, the cursor moves to the category's header row so it
    // stays visible.
    /// The recalculate scope for the cursor's row (spec 025): a
    /// Label row resolves to the label scope; an item to itself.
    pub(crate) fn cursor_recalc_scope(&self) -> Option<RecalculateScope> {
        let rows = self.active_rows();
        match rows.get(self.selected_index) {
            Some(PanelRow::Label(l)) => Some(RecalculateScope::Label(l.clone())),
            Some(PanelRow::Item(index)) => Some(RecalculateScope::Item(*index)),
            None => None,
        }
    }

    // Collapse (true) or expand (false) the label under the cursor
    // (spec 025): a Label row (or an item's first label) toggles the
    // label's collapse. When collapsing, the cursor moves to the
    // label's header row so it stays visible.
    pub(crate) fn set_label_collapsed(&mut self, collapsed: bool) {
        let Some(label) = self.cursor_label() else {
            return;
        };
        if collapsed {
            if !self.collapsed_labels.contains(&label) {
                self.collapsed_labels.push(label.clone());
            }
            // Move the cursor to the label's header row
            let rows = self.active_rows();
            if let Some(row) = rows
                .iter()
                .position(|r| matches!(r, PanelRow::Label(l) if *l == label))
            {
                self.selected_index = row;
            }
        } else {
            self.collapsed_labels.retain(|l| l != &label);
        }
    }

    /// Toggle collapse of the label under the cursor (spec 025).
    pub(crate) fn toggle_label_collapse(&mut self) {
        let Some(label) = self.cursor_label() else {
            return;
        };
        let collapsed = self.collapsed_labels.contains(&label);
        self.set_label_collapsed(!collapsed);
    }

    /// The effective selection state of an item's occurrence under a
    /// label (spec 025, US5): the override if present, else the
    /// item's own state.
    pub(crate) fn effective_selected(&self, item_index: usize, label: &str) -> bool {
        match self
            .occurrence_overrides
            .get(&(item_index, label.to_string()))
        {
            Some(state) => *state,
            None => self.packing_list.items[item_index].selected,
        }
    }

    /// The effective packed state of an item's occurrence under a
    /// label (spec 025, US5): the override if present, else the
    /// item's own state.
    pub(crate) fn effective_packed(&self, item_index: usize, label: &str) -> bool {
        match self
            .occurrence_overrides
            .get(&(item_index, label.to_string()))
        {
            Some(state) => *state,
            None => self.packing_list.items[item_index].packed,
        }
    }

    /// Toggle the selection of the item occurrence under the cursor
    /// (spec 025, US5): a SINGLETON flips the item's own state and
    /// clears its overrides (every label follows — FR-011); a
    /// non-singleton writes the per-occurrence override (FR-012).
    pub(crate) fn toggle_item_selection(&mut self) {
        let Some((item_index, label)) = self.cursor_occurrence() else {
            return;
        };
        if self.packing_list.items[item_index].singleton {
            let item = &mut self.packing_list.items[item_index];
            item.selected = !item.selected;
            // Synchronized: no per-occurrence divergence remains
            self.occurrence_overrides
                .retain(|(idx, _), _| *idx != item_index);
        } else {
            let now = !self.effective_selected(item_index, &label);
            self.occurrence_overrides.insert((item_index, label), now);
        }
    }

    /// Toggle the packed state of the item occurrence under the
    /// cursor (right panel only; spec 025, US5): a SINGLETON flips
    /// the item's own state and clears its overrides (FR-011); a
    /// non-singleton writes the per-occurrence override (FR-012).
    pub(crate) fn toggle_item_packed(&mut self) {
        if self.focus_side != FocusSide::Right {
            return;
        }
        let Some((item_index, label)) = self.cursor_occurrence() else {
            return;
        };
        if self.packing_list.items[item_index].singleton {
            let item = &mut self.packing_list.items[item_index];
            item.packed = !item.packed;
            self.occurrence_overrides
                .retain(|(idx, _), _| *idx != item_index);
        } else {
            let now = !self.effective_packed(item_index, &label);
            self.occurrence_overrides.insert((item_index, label), now);
        }
    }

    /// The (item, label) occurrence under the cursor: the label of the
    /// row's group context. On a Label header there is no occurrence
    /// (None); on an Item row, the label of the group it appears in.
    pub(crate) fn cursor_occurrence(&self) -> Option<(usize, String)> {
        let rows = self.active_rows();
        let mut current_label: Option<String> = None;
        for (pos, row) in rows.iter().enumerate().take(self.selected_index + 1) {
            match row {
                PanelRow::Label(l) => current_label = Some(l.clone()),
                PanelRow::Item(index) if pos == self.selected_index => {
                    return current_label.map(|l| (*index, l));
                }
                _ => {}
            }
        }
        None
    }

    /// Toggle selection of every item carrying the cursor's label,
    /// then collapse the label when items were added to the trip and
    /// expand it when removed (spec 025).
    pub(crate) fn toggle_label_selection(&mut self) {
        let Some(label) = self.cursor_label() else {
            return;
        };
        let was_selected = self
            .packing_list
            .items
            .iter()
            .any(|item| item.has_label(&label) && item.selected);

        for (idx, item) in self.packing_list.items.iter_mut().enumerate() {
            if item.has_label(&label) {
                item.selected = !item.selected;
                // Group actions act on the item (spec 025): clear any
                // per-occurrence overrides so the new state shows
                self.occurrence_overrides.retain(|(i, _), _| *i != idx);
            }
        }

        // Follow the selection state: added -> collapse, removed -> expand
        let now_selected = !was_selected;
        self.set_label_collapsed(!now_selected);
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
    /// Cycle the label picker's highlight (Left/Right in the Labels
    /// field, spec 025): clamped at the list bounds. The membership
    /// toggling is Space's job (the multi-select).
    pub(crate) fn cycle_label_highlight(&mut self, direction: i32) {
        let (field, label_list, label_index) = match &mut self.input_mode {
            InputMode::AddingItem {
                field,
                label_list,
                label_index,
                ..
            }
            | InputMode::EditingItem {
                field,
                label_list,
                label_index,
                ..
            } => (field, label_list, label_index),
            _ => return,
        };
        if *field != InputField::Labels {
            return;
        }
        let target =
            (*label_index as i32 + direction).clamp(0, label_list.len().saturating_sub(1) as i32);
        *label_index = target as usize;
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
                    RecalculateScope::Label(l) => item.has_label(l),
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
        labels: Vec<String>,
        singleton: bool,
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
        if !labels.is_empty() {
            // Deduplicate labels, preserving order
            let mut seen = std::collections::BTreeSet::new();
            item.labels = labels
                .into_iter()
                .filter(|l| seen.insert(l.clone()))
                .collect();
        }
        item.singleton = singleton;
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
        labels: Vec<String>,
        singleton: bool,
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
        // At least one label (FR-009): fall back to the first item's
        // first label, or a default
        let new_labels = if labels.is_empty() {
            if let Some(first) = self.packing_list.items.first() {
                vec![
                    first
                        .labels
                        .first()
                        .cloned()
                        .unwrap_or_else(|| "Other".to_string()),
                ]
            } else {
                vec!["Other".to_string()]
            }
        } else {
            labels
        };

        let mut item = Item::new(new_name, new_labels, qty);
        item.singleton = singleton;
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
