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
        entry("Ctrl+E", "Open the raw config file editor"),
        quit_entry("q", "Quit (saves config and trip file)"),
        entry("?", "Show this help (Main and Welcome screens)"),
        entry("O", "Open the configuration screen (modifiers)"),
        blank(),
        section("WELCOME SCREEN:", Color::Yellow),
        entry(
            "↑/↓",
            "Move focus between fields (file, weekdays, style, weather)",
        ),
        entry(
            "←/→",
            "Adjust the focused value (weekdays, cycle style/weather)",
        ),
        entry("+/-", "Adjust weekdays"),
        entry(
            "Enter",
            "Edit file name (on file field) / Continue to main screen",
        ),
        entry("F", "Edit the trip file path"),
        quit_entry("Esc", "Quit"),
        blank(),
        section("RESUME PROMPT:", Color::Yellow),
        entry("Y / Enter", "Resume the previous trip file"),
        entry("N / Esc", "Start a fresh list"),
        blank(),
        section("MAIN SCREEN (normal mode):", Color::Green),
        entry("↑/↓", "Move the cursor through the focused panel's rows"),
        entry("Tab / BackTab", "Switch focus between panels"),
        entry("Space", "Toggle selection (left) / toggle packed (right)"),
        entry("Ctrl+Space", "Toggle the category's selection (left panel)"),
        entry(
            "C",
            "Toggle collapse/expand of the category under the cursor",
        ),
        entry("A", "Add a new item"),
        entry("E", "Edit the item under the cursor"),
        entry("D", "Delete the item under the cursor (with confirmation)"),
        entry("R", "Recalculate quantities on the current list"),
        quit_entry("Esc", "Quit"),
        blank(),
        section("ADD/EDIT ITEM DIALOG:", Color::Yellow),
        entry(
            "Tab",
            "Cycle fields: Name -> Category -> Quantity -> Mode -> Name",
        ),
        entry(
            "↑/↓",
            "Move the category dropdown highlight (category field)",
        ),
        entry("←/→", "Cycle the category selection (category field)"),
        entry("Space", "Toggle the quantity mode (Mode field)"),
        entry("Enter", "Save"),
        quit_entry("Esc", "Cancel"),
        blank(),
        section("CONFIG SCREEN (modifiers):", Color::Yellow),
        entry("Tab / BackTab", "Switch section (travel styles / weather)"),
        entry("↑/↓", "Move the modifier cursor"),
        entry("A", "Add a modifier"),
        entry("E", "Edit the modifier under the cursor"),
        entry("D", "Delete the modifier under the cursor"),
        quit_entry(
            "Esc",
            "Close the modifier editor if open; otherwise leave the config screen",
        ),
        blank(),
        section("MODIFIER EDITOR POPUP:", Color::Yellow),
        entry("Tab", "Switch between name and multiplier field"),
        entry("Enter", "Save the modifier"),
        quit_entry("Esc", "Cancel the editor"),
        blank(),
        section("RAW CONFIG EDITOR (Ctrl+E):", Color::Yellow),
        entry("↑/↓/←/→", "Move the text cursor"),
        entry("Home/End", "Cursor to line start / end"),
        entry("Enter", "Insert a new line"),
        entry("Backspace", "Delete backward"),
        entry("Ctrl+S", "Save and apply (validates YAML first)"),
        quit_entry("Esc", "Discard edits and return"),
    ]
}
