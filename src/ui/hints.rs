use ratatui::style::{Color, Stylize};
use ratatui::text::{Line, Span};

/// The surfaces that carry a navigation hint line.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum HintSurface {
    LeftPanel,
    RightPanel,
    Welcome,
    Config,
    AddEditPopup,
    DeletePopup,
    ConfirmEditPopup,
    ModifierEditor,
    RawEditor,
}

/// The hint entries for a surface: (key, action) pairs rendered by
/// `hint_line`. Single source for rendering and the extraction test.
pub(crate) fn hint_entries(surface: HintSurface) -> &'static [(&'static str, &'static str)] {
    match surface {
        HintSurface::LeftPanel => &[
            ("↑/↓", "Navigate"),
            ("Space", "Select"),
            ("Ctrl+Space", "Category"),
            ("A", "Add"),
            ("E", "Edit"),
            ("D", "Delete"),
            ("C", "Collapse"),
            ("R", "Recalc item"),
            ("Ctrl+R", "Recalc category"),
            ("Ctrl+Shift+R", "Recalc all"),
            ("Tab", "Switch"),
            ("F", "Fullscreen"),
            ("?", "Help"),
        ],
        HintSurface::RightPanel => &[
            ("↑/↓", "Navigate"),
            ("Space", "Packed"),
            ("R", "Recalc item"),
            ("Ctrl+R", "Recalc category"),
            ("Ctrl+Shift+R", "Recalc all"),
            ("Tab", "Switch"),
            ("F", "Fullscreen"),
            ("?", "Help"),
        ],
        HintSurface::Welcome => &[
            ("↑/↓", "Navigate fields"),
            ("←/→", "Adjust values"),
            ("+/-", "Weekdays"),
            ("Enter", "Edit file / Continue"),
            ("F", "Edit path"),
            ("O", "Config"),
            ("Ctrl+E", "Raw editor"),
            ("q", "Quit"),
        ],
        HintSurface::Config => &[
            ("Tab", "Section"),
            ("↑/↓", "Move"),
            ("A", "Add"),
            ("E", "Edit"),
            ("D", "Delete"),
            ("Ctrl+E", "Raw editor"),
            ("Esc", "Back"),
        ],
        HintSurface::AddEditPopup => &[
            ("Tab", "Cycle fields"),
            ("↑/↓", "Fields / dropdown"),
            ("←/→", "Adjust values"),
            ("Space", "Mode / Buffer / Category"),
            ("Enter", "Save"),
            ("Esc", "Cancel"),
        ],
        HintSurface::DeletePopup => &[("Enter", "Confirm"), ("Esc", "Cancel")],
        HintSurface::ConfirmEditPopup => &[("Enter", "Confirm"), ("Esc", "Cancel")],
        HintSurface::ModifierEditor => &[
            ("Tab", "Switch field"),
            ("Enter", "Save"),
            ("Esc", "Cancel"),
        ],
        HintSurface::RawEditor => &[
            ("↑/↓/←/→", "Cursor"),
            ("Home/End", "Line start/end"),
            ("Enter", "New line"),
            ("Backspace", "Delete"),
            ("Ctrl+S", "Save & apply"),
            ("Esc", "Cancel"),
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
        "O",
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
