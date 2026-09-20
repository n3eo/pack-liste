# Quickstart Validation Guide: Harmonized Navigation, UI, and UX

**Feature**: Harmonized Navigation, UI, and UX Across Views
**Spec**: [spec.md](spec.md) | **Contract**: [contracts/hints-and-bindings.md](contracts/hints-and-bindings.md)

Runnable scenarios proving the harmonization end-to-end. Prerequisites:
Rust 1.98+, the repository, a terminal; the quality gate clean before
starting.

## Scenario 1: Uniform arrows (US1, SC-001)

```bash
cargo run -- --force-tui <trip-file>
```

1. On the main screen, press ←/→ on a category header.
2. Press `C` on the same header.
3. Open the edit dialog (`E`), Tab to the Category field, press ←/→.

**Expected**: ←/→ do nothing on the main screen; `C` collapses/expands
with the cursor-to-header behavior; in the dialog, ←/→ cycle the category
selection within bounds.

## Scenario 2: Hints on both list views at every size (US2, SC-002)

1. Open the main screen at a large size; read both panels' bottom lines.
2. Resize the terminal smaller (down to 80x24); read again.
3. Fill a panel with a deep list (many items).

**Expected**: Both panels show their hint line at every size — left:
navigate/select/category/add/edit/delete/collapse/switch/help; right:
navigate/packed/switch/help. The hints never overlap list content; the
list viewport shrinks instead.

## Scenario 3: One hint style everywhere (US3, SC-003)

1. Visit every screen: welcome, main, config, raw editor, and each popup
   (add, edit, delete confirm, modifier editor).
2. Compare each hint line's separator, glyphs, and colors.

**Expected**: All hint lines use " | " separators, ↑/↓/←/→ glyphs,
yellow-bold keys, plain actions. The add/edit popup's category hint reads
"Select category" (no "fields." artifact).

## Scenario 4: Focus indication (US4)

1. On the main screen, press Tab to switch focus; watch both titles.
2. Move the cursor on both panels.

**Expected**: The focused panel's title is highlighted per one
convention; the cursor row highlight is identical on both panels.

## Scenario 5: Help and hints agree (US5, SC-004)

```bash
cargo test help
```

**Expected**: The extraction test passes, now covering both the help
content and the hint entries. Manually: press `?` and verify no
Left/Right collapse entry remains and the dialog's ←/→ cycling is
documented.

## Final gate

```bash
cargo check && cargo clippy && cargo clippy --tests && cargo test && cargo fmt --check
```

**Expected**: All clean, zero warnings; all tests pass including the
extended extraction test and the new dialog-cycling tests.
