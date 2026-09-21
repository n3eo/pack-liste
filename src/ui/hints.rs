use ratatui::style::{Color, Stylize};
use ratatui::text::{Line, Span};

/// The surfaces that carry a navigation hint line.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum HintSurface {
    /// The main screen's shared footer: every binding the screen
    /// responds to, stated once for the whole screen (spec 023, US2)
    MainScreen,
    Welcome,
    Config,
    AddEditPopup,
    DeletePopup,
    ConfirmEditPopup,
    ModifierEditor,
    RawEditor,
}

// The canonical binding table (spec 023): every (key, action) pair
// the application displays, defined exactly once. Surfaces compose
// references to these entries — no surface re-types a string. A key
// with genuinely different meanings per surface has one entry per
// meaning (e.g. Space selects on the left panel, marks packed on
// the right); the wording per meaning is canonical everywhere it
// appears.
pub(crate) mod bindings {
    pub(crate) const NAVIGATE: (&str, &str) = ("↑/↓", "Navigate");
    pub(crate) const NAVIGATE_ALL: (&str, &str) = ("↑/↓/←/→", "Navigate");
    pub(crate) const ADJUST: (&str, &str) = ("←/→", "Adjust");
    pub(crate) const WEEKDAYS: (&str, &str) = ("+/-", "Weekdays");
    pub(crate) const TAB_PANEL: (&str, &str) = ("Tab", "Switch panel");
    pub(crate) const TAB_SECTION: (&str, &str) = ("Tab", "Switch section");
    pub(crate) const TAB_FIELD: (&str, &str) = ("Tab", "Next field");
    pub(crate) const SPACE_SELECT: (&str, &str) = ("Space", "Select");
    pub(crate) const SPACE_PACKED: (&str, &str) = ("Space", "Packed");
    pub(crate) const SPACE_TOGGLE: (&str, &str) = ("Space", "Toggle");
    pub(crate) const CTRL_SPACE_CATEGORY: (&str, &str) = ("Ctrl+Space", "Category");
    pub(crate) const ENTER_CONFIRM: (&str, &str) = ("Enter", "Confirm");
    pub(crate) const ENTER_SAVE: (&str, &str) = ("Enter", "Save");
    pub(crate) const ENTER_CONTINUE: (&str, &str) = ("Enter", "Continue");
    pub(crate) const ENTER_NEWLINE: (&str, &str) = ("Enter", "New line");
    pub(crate) const ESC_CANCEL: (&str, &str) = ("Esc", "Cancel");
    pub(crate) const ESC_BACK: (&str, &str) = ("Esc", "Back");
    pub(crate) const F_FULLSCREEN: (&str, &str) = ("F", "Fullscreen");
    pub(crate) const F_EDIT_PATH: (&str, &str) = ("F", "Edit path");
    pub(crate) const RECALC_ITEM: (&str, &str) = ("R", "Recalc item");
    pub(crate) const RECALC_CATEGORY: (&str, &str) = ("Ctrl+R", "Recalc category");
    pub(crate) const RECALC_ALL: (&str, &str) = ("Ctrl+Shift+R", "Recalc all");
    pub(crate) const ADD: (&str, &str) = ("A", "Add");
    pub(crate) const EDIT: (&str, &str) = ("E", "Edit");
    pub(crate) const DELETE: (&str, &str) = ("D", "Delete");
    pub(crate) const COLLAPSE: (&str, &str) = ("C", "Collapse");
    pub(crate) const HELP: (&str, &str) = ("?", "Help");
    pub(crate) const CONFIG: (&str, &str) = ("Ctrl+O", "Config");
    pub(crate) const RAW_EDITOR: (&str, &str) = ("Ctrl+E", "Raw editor");
    pub(crate) const SAVE_APPLY: (&str, &str) = ("Ctrl+S", "Save & apply");
    pub(crate) const QUIT: (&str, &str) = ("q", "Quit");
    pub(crate) const LINE_START_END: (&str, &str) = ("Home/End", "Line start/end");
    pub(crate) const BACKSPACE_DELETE: (&str, &str) = ("Backspace", "Delete");

    /// The full canonical table: every entry, in display order. The
    /// single source for the property tests (uniqueness, usage, and
    /// the help-token agreement).
    #[cfg(test)]
    pub(crate) const ALL: [(&str, &str); 33] = [
        NAVIGATE,
        NAVIGATE_ALL,
        ADJUST,
        WEEKDAYS,
        TAB_PANEL,
        TAB_SECTION,
        TAB_FIELD,
        SPACE_SELECT,
        SPACE_PACKED,
        SPACE_TOGGLE,
        CTRL_SPACE_CATEGORY,
        ENTER_CONFIRM,
        ENTER_SAVE,
        ENTER_CONTINUE,
        ENTER_NEWLINE,
        ESC_CANCEL,
        ESC_BACK,
        F_FULLSCREEN,
        F_EDIT_PATH,
        RECALC_ITEM,
        RECALC_CATEGORY,
        RECALC_ALL,
        ADD,
        EDIT,
        DELETE,
        COLLAPSE,
        HELP,
        CONFIG,
        RAW_EDITOR,
        SAVE_APPLY,
        QUIT,
        LINE_START_END,
        BACKSPACE_DELETE,
    ];
}

/// The hint entries for a surface: (key, action) pairs rendered by
/// `hint_line`. Composed from the canonical table — the single source
/// for rendering and the extraction test.
pub(crate) fn hint_entries(surface: HintSurface) -> &'static [(&'static str, &'static str)] {
    use bindings as b;
    match surface {
        // The main screen's one shared footer: every binding the
        // screen responds to, each stated exactly once (spec 023,
        // US2 — no pair appears twice on the screen)
        HintSurface::MainScreen => &[
            b::NAVIGATE,
            b::SPACE_SELECT,
            b::SPACE_PACKED,
            b::CTRL_SPACE_CATEGORY,
            b::ADD,
            b::EDIT,
            b::DELETE,
            b::COLLAPSE,
            b::RECALC_ITEM,
            b::RECALC_CATEGORY,
            b::RECALC_ALL,
            b::TAB_PANEL,
            b::F_FULLSCREEN,
            b::HELP,
        ],
        HintSurface::Welcome => &[
            b::NAVIGATE,
            b::ADJUST,
            b::WEEKDAYS,
            b::ENTER_CONTINUE,
            b::F_EDIT_PATH,
            b::CONFIG,
            b::RAW_EDITOR,
            b::QUIT,
        ],
        HintSurface::Config => &[
            b::TAB_SECTION,
            b::NAVIGATE,
            b::ADD,
            b::EDIT,
            b::DELETE,
            b::RAW_EDITOR,
            b::ESC_BACK,
        ],
        HintSurface::AddEditPopup => &[
            b::TAB_FIELD,
            b::NAVIGATE,
            b::ADJUST,
            b::SPACE_TOGGLE,
            b::ENTER_SAVE,
            b::ESC_CANCEL,
        ],
        HintSurface::DeletePopup => &[b::ENTER_CONFIRM, b::ESC_CANCEL],
        HintSurface::ConfirmEditPopup => &[b::ENTER_CONFIRM, b::ESC_CANCEL],
        HintSurface::ModifierEditor => &[b::TAB_FIELD, b::ENTER_SAVE, b::ESC_CANCEL],
        HintSurface::RawEditor => &[
            b::NAVIGATE_ALL,
            b::LINE_START_END,
            b::ENTER_NEWLINE,
            b::BACKSPACE_DELETE,
            b::SAVE_APPLY,
            b::ESC_CANCEL,
        ],
    }
}

/// The single hint-line constructor: keys in the highlight color and
/// bold, actions plain, " | " between entries.
pub(crate) fn hint_line(entries: &[(&'static str, &'static str)]) -> Line<'static> {
    let mut spans = Vec::new();
    for (i, (key, action)) in entries.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw(" | "));
        }
        spans.push(Span::raw(*key).bold().fg(Color::Yellow));
        spans.push(Span::raw(" "));
        spans.push(Span::raw(*action));
    }
    Line::from(spans)
}

/// The rendered width of one hint entry ("KEY Action").
fn hint_entry_width(key: &str, action: &str) -> usize {
    key.chars().count() + 1 + action.chars().count()
}

/// Split a surface's hint entries into rows that each fit `width`
/// columns, packing entries greedily and breaking only at entry
/// boundaries so no binding is ever cut off. Every entry appears
/// exactly once across the rows.
pub(crate) fn hint_rows(
    entries: &[(&'static str, &'static str)],
    width: usize,
) -> Vec<Line<'static>> {
    let mut rows = Vec::new();
    let mut current: Vec<(&'static str, &'static str)> = Vec::new();
    let mut current_width = 0;
    for entry in entries {
        let entry_width = hint_entry_width(entry.0, entry.1);
        let separator = if current.is_empty() { 0 } else { 3 };
        if !current.is_empty() && current_width + separator + entry_width > width {
            rows.push(hint_line(&current));
            current = Vec::new();
            current_width = 0;
        }
        current_width += if current.is_empty() { 0 } else { 3 };
        current_width += entry_width;
        current.push(*entry);
    }
    if !current.is_empty() {
        rows.push(hint_line(&current));
    }
    if rows.is_empty() {
        // An empty entry list still renders one (blank) hint line so
        // the footer area is never zero-height.
        rows.push(Line::from(""));
    }
    rows
}

/// The collapsed hint line (spec 006, US5): a single short pointer
/// when the full rows would squeeze the lists.
pub(crate) fn collapsed_hint_line() -> Line<'static> {
    hint_line(&[("?", "Help")])
}

/// The collapse decision (spec 006, US5, plan D3): when the packed
/// hint rows would squeeze the lists beyond the height budget, the
/// hints collapse to the single short `? Help` line. A pure
/// function of (entries, width, budget) — stable at the boundary
/// because it is recomputed per frame from the same inputs.
pub(crate) fn hint_rows_fit_budget(
    entries: &[(&'static str, &'static str)],
    width: usize,
    height_budget: usize,
) -> bool {
    hint_rows(entries, width).len() <= height_budget
}

// The binding tokens the help screen must list; the extraction test
// compares this against the handler bindings (SC-004).
#[cfg(test)]
pub(crate) fn help_binding_tokens() -> Vec<&'static str> {
    vec![
        // Global
        "Ctrl+C",
        "Ctrl+E",
        "q",
        "?",
        "Ctrl+O",
        // Welcome
        "↑/↓",
        "←/→",
        "+/-",
        "Space",
        "Enter",
        "F",
        "Esc",
        // Resume prompt
        "Y",
        "N",
        // Main screen
        "Tab",
        "BackTab",
        "Ctrl+Space",
        "C",
        "A",
        "E",
        "D",
        "R",
        "Ctrl+R",
        "Ctrl+Shift+R",
        // Add/Edit dialog
        "Mode",
        // Config screen
        // (Tab/BackTab/↑/↓/A/E/D/Esc shared with above tokens)
        // Modifier editor
        // (Tab/Enter/Esc shared with above tokens)
        // Raw config editor
        "↑/↓/←/→",
        "Home/End",
        "Backspace",
        "Ctrl+S",
    ]
}
