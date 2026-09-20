# Quickstart: Code Review Remediation

**Feature**: 022-code-review-refactoring | **Date**: 2026-09-20 | **Spec**: [spec.md](spec.md)

This feature is behavior-preserving refactoring: the application must
look and behave exactly as before. Validation therefore means proving
*nothing changed* while the structure improved. Run the full sequence
after each user story, and the smoke pass after US1, US2, and US5.

## Prerequisites

- Rust toolchain (1.98+, edition 2024)
- A terminal (the TUI requires a TTY)
- A scratch directory for test files (e.g., `/tmp/pack-liste-verify`)

## 1. Quality gate (every story)

```bash
cargo check
cargo clippy
cargo clippy --tests
cargo test
cargo fmt --check
```

**Expected**: all five exit 0; `cargo test` reports 39 passing tests
(33 UI + 6 quantity) — the same 39 as before the refactor, with
assertions unmodified. Zero warnings anywhere (constitution
Principle II).

## 2. Format round-trip (US3, US4)

Proves the persisted formats are untouched (data-model V1).

```bash
mkdir -p /tmp/pack-liste-verify && cd /tmp/pack-liste-verify
# Before the refactor: run the app, add an item, save, quit.
# Keep the produced config.yaml and trip file.
```

Then, after the refactor:

```bash
cargo run -- /tmp/pack-liste-verify/<trip-file>
```

**Expected**: the trip loads, all previously selected items appear
selected, packed markers are preserved, quantities recalculate to the
same values. A hand-edit of the trip file (e.g., flipping one
`packed: false` to `true`) is honored on the next load.

## 3. Structure verification (US1, US2)

```bash
find src -name "*.rs" | xargs wc -l | sort -n
grep -rn "get_all_categories\|get_unique_categories" src/
grep -rn "draw_add_item_popup\|draw_edit_item_popup" src/
grep -rn "cycle_travel_style\|cycle_weather" src/
```

**Expected**:
- No file exceeds 1,500 lines; `src/ui/app.rs` no longer exists (its
  contents live in the `src/ui/` tree per
  [contracts/module-structure.md](contracts/module-structure.md)).
- The duplicated names above are gone, replaced by single shared
  implementations (SC-003).

## 4. Dead code verification (US3)

```bash
grep -rn "chrono\|tracing\|mockall" Cargo.toml
grep -rn "base_quantity\|\.editable" src/
```

**Expected**: no matches. `Cargo.toml` lists neither `chrono`,
`tracing`, `tracing-subscriber`, nor `mockall`; no model field
`base_quantity` or `editable` remains (`selected` replaced it).

## 5. Error surfacing (US5)

```bash
# Make the config path unwritable, then trigger a save from the TUI
mkdir -p /tmp/pack-liste-verify/ro && chmod 555 /tmp/pack-liste-verify/ro
cargo run -- /tmp/pack-liste-verify/ro/trip.yaml
# In the TUI: toggle an item (triggers save), press Ctrl+S in the raw editor
```

**Expected**: the failure is visible — a status line reports the save
error (or the existing editor error line does) — and the application
does not crash or silently continue. Restore permissions afterwards:
`chmod 755 /tmp/pack-liste-verify/ro`.

**Convergence additions (Phase 9)**:

- Trigger a save failure from the **config screen** (add or edit a
  modifier with the config dir unwritable): the status line must be
  visible on the config screen's footer, not only the main screen.
- After a status message is set, navigate to another screen (Esc) or
  close a dialog: the message must clear (data-model V3) — it does not
  persist across the transition.

## 6. Manual smoke pass (US1, US2, US5)

Launch: `cargo run`

Walk every screen and interaction; compare against pre-refactor
behavior:

1. **Welcome**: up/down moves focus (file field first, required),
   left/right adjusts weekdays / cycles travel style and weather,
   Space toggles activities, Enter proceeds, F edits the path, Esc
   quits.
2. **Main**: up/down navigates both panels, Tab switches sides, Space
   toggles selection (left) / packed (right), Ctrl+Space toggles a
   category, C collapses/expands, A adds, E edits, D deletes (with
   confirmation), R recalculates, ? opens help, O opens config.
3. **Dialogs**: Tab cycles Name → Category → Quantity → Mode, up/down
   moves the category dropdown, left/right cycles the category, Esc
   cancels, Enter saves.
4. **Config screen**: up/down navigates sections and entries,
   left/right adjusts, A adds, E edits, D deletes, Esc returns.
5. **Modifier editor**: up/down cycles fields, Esc returns to the
   config screen (not stuck).
6. **Raw editor (Ctrl+E)**: type, edit, Ctrl+S saves; a YAML error
   shows in the footer; Esc returns.
7. **Quit (q / Ctrl+C / Esc)**: config and trip file save; relaunching
   with the same file resumes the trip.

**Expected**: identical behavior to the pre-refactor binary at every
step; the help screen (?) lists the same bindings as before.

## 7. Documentation check (US6)

```bash
# Every path the docs mention must exist
grep -n "src/" docs/quantity-calculation.md
```

**Expected**: every referenced module path exists in the tree; the
document describes YAML storage (not JSON) and the actual module
layout; no mention of `QuantityError`/`ConfigError`/`UiError` or a
`services/` directory.
