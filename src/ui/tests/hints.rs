use super::all_hint_surfaces;
use crate::ui::help::help_screen_text;
use crate::ui::hints::{HintSurface, help_binding_tokens, hint_entries, hint_line, hint_rows};

#[test]
fn help_lists_every_binding_token() {
    // The tokens the handlers actually respond to (the curated binding
    // list from contracts/input-and-help.md). Every token must appear
    // in the help content; the token list must have no duplicates.
    let tokens = help_binding_tokens();

    // No duplicates (SC-004)
    let mut sorted = tokens.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        tokens.len(),
        "help_binding_tokens contains duplicates"
    );

    // Every token is rendered by the help screen: the entries are
    // built from the same token strings, so the help text contains
    // each token verbatim.
    let help_text = help_screen_text();
    for token in &tokens {
        assert!(
            help_text.contains(token),
            "help screen is missing binding: {token}"
        );
    }
}

#[test]
fn help_screen_text_covers_all_handler_bindings() {
    // The curated handler-binding list (from the code review): every
    // binding the key handlers respond to must be a help token.
    let handler_bindings = [
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
        // Main
        "Tab",
        "BackTab",
        "Ctrl+Space",
        "C",
        "A",
        "E",
        "D",
        "R",
        // Dialog (Mode field added by US1)
        "Mode",
        // Raw config editor
        "Home/End",
        "Backspace",
        "Ctrl+S",
    ];
    let tokens = help_binding_tokens();
    for binding in handler_bindings {
        assert!(
            tokens.contains(&binding),
            "handler binding missing from help tokens: {binding}"
        );
    }
}

// ---- US5 (spec 016): hints and help agree ----

#[test]
fn hint_keys_are_help_tokens() {
    // Every key shown in any surface's hint line must be a help
    // token, so hints cannot show a binding the help omits.
    let tokens = help_binding_tokens();
    for surface in all_hint_surfaces() {
        for (key, _) in hint_entries(surface) {
            assert!(
                tokens.contains(key),
                "hint key {key:?} ({surface:?}) missing from help tokens"
            );
        }
    }
}

#[test]
fn hint_entries_match_the_contract_binding_sets() {
    // The drop/stale direction: each surface's entry set must equal
    // the contract's binding set exactly, so removing or corrupting
    // an entry (e.g. deleting ("C", "Collapse") from the left
    // panel) fails this test.
    let expected: [(HintSurface, &[(&str, &str)]); 8] = [
        (
            HintSurface::LeftPanel,
            &[
                ("↑/↓", "Navigate"),
                ("Space", "Select"),
                ("Ctrl+Space", "Category"),
                ("A", "Add"),
                ("E", "Edit"),
                ("D", "Delete"),
                ("C", "Collapse"),
                ("R", "Recalculate"),
                ("Tab", "Switch"),
                ("?", "Help"),
            ],
        ),
        (
            HintSurface::RightPanel,
            &[
                ("↑/↓", "Navigate"),
                ("Space", "Packed"),
                ("R", "Recalculate"),
                ("Tab", "Switch"),
                ("?", "Help"),
            ],
        ),
        (
            HintSurface::Welcome,
            &[
                ("↑/↓", "Navigate fields"),
                ("←/→", "Adjust values"),
                ("+/-", "Weekdays"),
                ("Enter", "Edit file / Continue"),
                ("F", "Edit path"),
                ("O", "Config"),
                ("Ctrl+E", "Raw editor"),
                ("q", "Quit"),
            ],
        ),
        (
            HintSurface::Config,
            &[
                ("Tab", "Section"),
                ("↑/↓", "Move"),
                ("A", "Add"),
                ("E", "Edit"),
                ("D", "Delete"),
                ("Ctrl+E", "Raw editor"),
                ("Esc", "Back"),
            ],
        ),
        (
            HintSurface::AddEditPopup,
            &[
                ("Tab", "Cycle fields"),
                ("↑/↓", "Category dropdown"),
                ("←/→", "Cycle category"),
                ("Space", "Mode / Category"),
                ("Enter", "Save"),
                ("Esc", "Cancel"),
            ],
        ),
        (
            HintSurface::DeletePopup,
            &[("Enter", "Confirm"), ("Esc", "Cancel")],
        ),
        (
            HintSurface::ModifierEditor,
            &[
                ("Tab", "Switch field"),
                ("Enter", "Save"),
                ("Esc", "Cancel"),
            ],
        ),
        (
            HintSurface::RawEditor,
            &[
                ("↑/↓/←/→", "Cursor"),
                ("Home/End", "Line start/end"),
                ("Enter", "New line"),
                ("Backspace", "Delete"),
                ("Ctrl+S", "Save & apply"),
                ("Esc", "Cancel"),
            ],
        ),
    ];
    for (surface, contract) in expected {
        let actual = hint_entries(surface);
        assert_eq!(
            actual, contract,
            "{surface:?} hint entries drifted from the contract"
        );
    }
}

#[test]
fn hint_lines_use_the_unified_style() {
    // The single constructor renders " | " separators and no
    // long-form arrow words; the corrupted text is gone.
    let line = hint_line(hint_entries(HintSurface::LeftPanel));
    let text: String = line.spans.iter().map(|s| s.content.to_string()).collect();
    assert!(text.contains(" | "), "hint line must use | separators");
    assert!(!text.contains("Up/Down"), "hint line must use glyphs");
    assert!(!text.contains("Left/Right"), "hint line must use glyphs");
    let all_text = format!("{}{}", help_screen_text(), {
        let mut s = String::new();
        for surface in [
            HintSurface::LeftPanel,
            HintSurface::RightPanel,
            HintSurface::Welcome,
            HintSurface::Config,
            HintSurface::AddEditPopup,
            HintSurface::DeletePopup,
            HintSurface::ModifierEditor,
            HintSurface::RawEditor,
        ] {
            for (k, a) in hint_entries(surface) {
                s.push_str(k);
                s.push_str(a);
            }
        }
        s
    });
    assert!(
        !all_text.contains("fields.category"),
        "corrupted hint text must be gone"
    );
}

#[test]
fn hint_rows_fit_width_and_keep_every_entry() {
    // At the minimum panel width (39 columns at an 80-column
    // terminal), every entry still appears and no row overflows.
    let entries = hint_entries(HintSurface::LeftPanel);
    let rows = hint_rows(entries, 39);
    let mut seen: Vec<(&str, &str)> = Vec::new();
    for row in &rows {
        let text: String = row.spans.iter().map(|s| s.content.to_string()).collect();
        assert!(
            text.chars().count() <= 39,
            "hint row exceeds the panel width: {text:?}"
        );
        // Count full "KEY Action" occurrences, not substrings
        for (key, action) in entries {
            if text.contains(&format!("{key} {action}")) {
                seen.push((key, action));
            }
        }
    }
    // Every entry appears exactly once across the rows
    assert_eq!(seen.len(), entries.len(), "every entry must appear once");
    // A wide width packs everything into one row
    assert_eq!(hint_rows(entries, 200).len(), 1);
    // An empty entry list still yields one (blank) row
    assert_eq!(hint_rows(&[], 39).len(), 1);
}

// ---- US1/US3 (spec 018): the recalculate action ----
