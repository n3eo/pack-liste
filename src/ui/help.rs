use ratatui::style::{Color, Stylize};
use ratatui::text::{Line, Span};

// The help screen's content as plain text, for the completeness test.
#[cfg(test)]
pub(crate) fn help_screen_text() -> String {
    let help_text = help_content();
    let mut text = String::new();
    for line in &help_text {
        for span in &line.spans {
            text.push_str(&span.content);
        }
        text.push('\n');
    }
    text
}

// The help screen's content, shared by the renderer and the
// completeness test.
pub(crate) fn help_content() -> Vec<Line<'static>> {
    fn key_span(k: &'static str) -> Span<'static> {
        Span::raw(k).bold().fg(Color::Yellow)
    }
    fn quit_key_span(k: &'static str) -> Span<'static> {
        Span::raw(k).bold().fg(Color::Red)
    }
    fn entry(k: &'static str, d: &'static str) -> Line<'static> {
        Line::from(vec![key_span(k), Span::raw(" - "), Span::raw(d)])
    }
    fn quit_entry(k: &'static str, d: &'static str) -> Line<'static> {
        Line::from(vec![quit_key_span(k), Span::raw(" - "), Span::raw(d)])
    }
    fn section(t: &'static str, c: Color) -> Line<'static> {
        Line::from(Span::raw(t).bold().fg(c))
    }
    fn blank() -> Line<'static> {
        Line::from("")
    }

    vec![
        section("GLOBAL (any screen):", Color::Yellow),
        entry("Ctrl+C", "Quit immediately (saves config and trip file)"),
        entry("Ctrl+E", "Raw editor: open the raw config file editor"),
        quit_entry("q", "Quit (saves config and trip file)"),
        entry("?", "Help: show this screen (Main and Welcome)"),
        entry("O", "Config: open the configuration screen (modifiers)"),
        blank(),
        section("WELCOME SCREEN:", Color::Yellow),
        entry(
            "↑/↓",
            "Navigate between fields (file, weekdays, style, weather)",
        ),
        entry(
            "←/→",
            "Adjust the focused value (weekdays, cycle style/weather)",
        ),
        entry("+/-", "Adjust the Weekdays"),
        entry(
            "Enter",
            "Edit file name (on file field) / Continue to main screen",
        ),
        entry("F", "Edit path: change the trip file path"),
        quit_entry("Esc", "Quit"),
        blank(),
        section("RESUME PROMPT:", Color::Yellow),
        entry("Y / Enter", "Resume the previous trip file"),
        entry("N / Esc", "Start a fresh list"),
        blank(),
        section("MAIN SCREEN (normal mode):", Color::Green),
        entry("↑/↓", "Navigate the focused panel's rows"),
        entry("Tab / BackTab", "Switch panel"),
        entry("Space", "Select the item (left) / Packed toggle (right)"),
        entry("Ctrl+Space", "Toggle the category's selection (left panel)"),
        entry("C", "Collapse/expand the category under the cursor"),
        entry("A", "Add a new item"),
        entry("E", "Edit the item under the cursor"),
        entry("D", "Delete the item under the cursor (with confirmation)"),
        entry(
            "R",
            "Recalc item: the current item (on a category header: its category)",
        ),
        entry("Ctrl+R", "Recalc category: the current item's category"),
        entry("Ctrl+Shift+R", "Recalc all: every automatic item"),
        entry(
            "F",
            "Fullscreen: toggle the compiled-list view (Tab is a no-op there)",
        ),
        quit_entry("Esc", "Quit"),
        blank(),
        section("ADD/EDIT ITEM DIALOG:", Color::Yellow),
        entry(
            "Tab",
            "Next field: Name -> Category -> Quantity -> Mode -> Rate -> Buffer -> Name",
        ),
        entry("↑/↓", "Navigate fields"),
        entry(
            "←/→",
            "Adjust the focused value: category selection, quantity ±1",
        ),
        entry("Space", "Toggle the quantity mode (Mode field)"),
        entry(
            "Space",
            "Cycle the buffer kind: none -> reserve -> multiplier (Buffer field)",
        ),
        entry("Enter", "Save"),
        quit_entry("Esc", "Cancel"),
        blank(),
        section("EDIT CONFIRMATION:", Color::Yellow),
        entry("Enter", "Confirm: apply the edit and save"),
        quit_entry("Esc", "Discard the edit (nothing written)"),
        entry("N", "Discard the edit (nothing written)"),
        blank(),
        section("CONFIG SCREEN (modifiers):", Color::Yellow),
        entry("Tab / BackTab", "Switch section (travel styles / weather)"),
        entry("↑/↓", "Navigate the modifier list"),
        entry("A", "Add a modifier"),
        entry("E", "Edit the modifier under the cursor"),
        entry("D", "Delete the modifier under the cursor"),
        quit_entry(
            "Esc",
            "Close the modifier editor if open; otherwise leave the config screen",
        ),
        blank(),
        section("MODIFIER EDITOR POPUP:", Color::Yellow),
        entry("Tab", "Next field (name / multiplier)"),
        entry("Enter", "Save the modifier"),
        quit_entry("Esc", "Cancel the editor"),
        blank(),
        section("RAW CONFIG EDITOR (Ctrl+E):", Color::Yellow),
        entry("↑/↓/←/→", "Navigate the text cursor"),
        entry("Home/End", "Line start/end: jump the cursor"),
        entry("Enter", "New line: insert one"),
        entry("Backspace", "Delete backward"),
        entry("Ctrl+S", "Save & apply (validates YAML first)"),
        quit_entry("Esc", "Discard edits and return"),
    ]
}
