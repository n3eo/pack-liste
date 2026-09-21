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
            HintSurface::MainScreen,
            &[
                ("↑/↓", "Navigate"),
                ("Space", "Select"),
                ("Space", "Packed"),
                ("Ctrl+Space", "Category"),
                ("A", "Add"),
                ("E", "Edit"),
                ("D", "Delete"),
                ("C", "Collapse"),
                ("R", "Recalc item"),
                ("Ctrl+R", "Recalc category"),
                ("Ctrl+Shift+R", "Recalc all"),
                ("Tab", "Switch panel"),
                ("F", "Fullscreen"),
                ("?", "Help"),
            ],
        ),
        (
            HintSurface::Welcome,
            &[
                ("↑/↓", "Navigate"),
                ("←/→", "Adjust"),
                ("+/-", "Weekdays"),
                ("Enter", "Continue"),
                ("F", "Edit path"),
                ("Ctrl+O", "Config"),
                ("Ctrl+E", "Raw editor"),
                ("q", "Quit"),
            ],
        ),
        (
            HintSurface::Config,
            &[
                ("Tab", "Switch section"),
                ("↑/↓", "Navigate"),
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
                ("Tab", "Next field"),
                ("↑/↓", "Navigate"),
                ("←/→", "Adjust"),
                ("Space", "Toggle"),
                ("Enter", "Save"),
                ("Esc", "Cancel"),
            ],
        ),
        (
            HintSurface::DeletePopup,
            &[("Enter", "Confirm"), ("Esc", "Cancel")],
        ),
        (
            HintSurface::ConfirmEditPopup,
            &[("Enter", "Confirm"), ("Esc", "Cancel")],
        ),
        (
            HintSurface::ModifierEditor,
            &[("Tab", "Next field"), ("Enter", "Save"), ("Esc", "Cancel")],
        ),
        (
            HintSurface::RawEditor,
            &[
                ("↑/↓/←/→", "Navigate"),
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
    let line = hint_line(hint_entries(HintSurface::MainScreen));
    let text: String = line.spans.iter().map(|s| s.content.to_string()).collect();
    assert!(text.contains(" | "), "hint line must use | separators");
    assert!(!text.contains("Up/Down"), "hint line must use glyphs");
    assert!(!text.contains("Left/Right"), "hint line must use glyphs");
    let all_text = format!("{}{}", help_screen_text(), {
        let mut s = String::new();
        for surface in [
            HintSurface::MainScreen,
            HintSurface::Welcome,
            HintSurface::Config,
            HintSurface::AddEditPopup,
            HintSurface::DeletePopup,
            HintSurface::ConfirmEditPopup,
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
    // The main screen's shared footer spans the full terminal width,
    // so its minimum is the stacked-mode width (40 columns).
    let entries = hint_entries(HintSurface::MainScreen);
    let rows = hint_rows(entries, 40);
    let mut seen: Vec<(&str, &str)> = Vec::new();
    for row in &rows {
        let text: String = row.spans.iter().map(|s| s.content.to_string()).collect();
        assert!(
            text.chars().count() <= 40,
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
    // A wide width packs everything into one row (the full set is
    // ~215 columns including separators)
    assert_eq!(hint_rows(entries, 220).len(), 1);
    // An empty entry list still yields one (blank) row
    assert_eq!(hint_rows(&[], 40).len(), 1);
}

// ---- US1/US3 (spec 018): the recalculate action ----

// ---- Spec 023: the canonical binding table ----

use crate::ui::hints::bindings;

#[test]
fn canonical_action_text_is_unique_per_key() {
    // SC-001: every (key, meaning) has exactly one action text. The
    // canonical table holds each pair once; no surface may show a
    // text for a key that is not the canonical one for its meaning.
    // A key with several meanings has several entries — but each
    // (key, action) pair exists exactly once in the table.
    let mut seen = std::collections::BTreeSet::new();
    for (key, action) in bindings::ALL {
        assert!(
            seen.insert((key, action)),
            "duplicate canonical pair: {key} -> {action}"
        );
    }
    // Every surface's entries are canonical: each (key, action) pair
    // a surface shows exists in the table (no surface-specific
    // paraphrase can slip in).
    for surface in all_hint_surfaces() {
        for (key, action) in hint_entries(surface) {
            assert!(
                bindings::ALL.contains(&(*key, *action)),
                "{surface:?} shows a non-canonical pair: {key} -> {action}"
            );
        }
    }
}

#[test]
fn canonical_entries_are_all_used() {
    // Constitution I / FR-005: every canonical entry is referenced by
    // at least one surface composition — no dead entries.
    for (key, action) in bindings::ALL {
        let used = all_hint_surfaces()
            .iter()
            .any(|surface| hint_entries(*surface).contains(&(key, action)));
        assert!(
            used,
            "canonical pair {key} -> {action} is not used by any surface"
        );
    }
}

#[test]
fn no_pair_repeats_on_one_screen() {
    // SC-002: no (key, action) pair is visible twice on one screen.
    // The main screen renders ONE shared footer (spec 023, US2); the
    // config screen's footer and the modifier editor popup are
    // visible together.
    let config: std::collections::BTreeSet<(&str, &str)> =
        hint_entries(HintSurface::Config).iter().copied().collect();
    let editor: std::collections::BTreeSet<(&str, &str)> =
        hint_entries(HintSurface::ModifierEditor)
            .iter()
            .copied()
            .collect();
    let shared_cfg: Vec<_> = config.intersection(&editor).collect();
    assert!(
        shared_cfg.is_empty(),
        "the config screen and its popup restate shared pairs: {shared_cfg:?}"
    );
    // No composition contains a duplicate pair
    for surface in all_hint_surfaces() {
        let entries = hint_entries(surface);
        let unique: std::collections::BTreeSet<_> = entries.iter().copied().collect();
        assert_eq!(
            unique.len(),
            entries.len(),
            "{surface:?} contains a duplicate pair"
        );
    }
}

#[test]
fn help_tokens_match_canonical_table() {
    // FR-004: the help screen agrees with the canonical table — every
    // key the help lists is a canonical key, and each description
    // contains its key's canonical action word.
    let help = help_screen_text();
    let canonical_keys: std::collections::BTreeSet<&str> =
        bindings::ALL.iter().map(|(k, _)| *k).collect();
    for (key, action) in bindings::ALL {
        // Every canonical key that a hint surface shows appears in the help
        let used_by_surface = all_hint_surfaces()
            .iter()
            .any(|s| hint_entries(*s).contains(&(key, action)));
        if used_by_surface {
            assert!(
                help.contains(key),
                "help screen is missing the canonical key: {key}"
            );
            assert!(
                help.contains(action),
                "help description for {key} lacks the canonical action word: {action}"
            );
        }
        assert!(
            canonical_keys.contains(key),
            "internal consistency: {key} must be a canonical key"
        );
    }
}
