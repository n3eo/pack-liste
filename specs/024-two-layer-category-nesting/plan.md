# Implementation Plan: Two-Layer Category Nesting

**Branch**: `024-two-layer-category-nesting` | **Date**: 2026-09-21 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/024-two-layer-category-nesting/spec.md`

## Summary

Derive a two-layer tree from the existing category strings at display time: split each category on the first ": " into a top-layer group and a sub-category; render both panels as group headers, sub-category headers, and items; extend navigation, collapse, selection, sticky headers, and the recalculate scopes to both layers. The stored category strings are untouched — the nesting is a pure view over the existing data.

## Technical Context

**Language/Version**: Rust 1.98 (2024 edition)

**Primary Dependencies**: ratatui 0.30, crossterm 0.29 (unchanged)

**Storage**: None new — the category strings (with ": ") are persisted verbatim; the layers are derived per frame, never stored.

**Testing**: `cargo test` — new state-layer tests for the split, the row model, collapse/selection at both layers, and the format round trip; PTY checks for the rendered tree. The full gate must stay green.

**Target Platform**: Terminal TUI (Linux)

**Project Type**: CLI/TUI application (single binary)

**Performance Goals**: Unchanged — the derivation is O(categories) per frame, negligible at the existing scale (35 categories, ~200 items).

**Constraints**: Warning-free gate; the persisted formats are frozen (FR-007); no new key bindings; the split is on the FIRST ": " only; identity is always the full category string.

**Scale/Scope**: A pure split helper, a two-layer row model (PanelRow gains a group variant), collapse state for both layers, the two panel renderers, the sticky-header extension, and the recalculate scope resolution. No new modules.

## Constitution Check

- **Principle I** (no unused code): satisfied — the single-layer row model is replaced by the two-layer model; no dead variants remain.
- **Principle II** (zero warnings): satisfied — gate green per task.
- **Principle III** (documented and tested): satisfied — the split rule and layer semantics are documented; tests cover the split, the tree rendering model, both layers' collapse/selection, and the format round trip; `TUI_USAGE.md` documents the two-layer display.
- No violations.

## Project Structure

### Documentation (this feature)

```text
specs/024-two-layer-category-nesting/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output (the layer derivation)
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (the display-tree contract)
└── tasks.md             # /speckit-tasks output
```

### Source Code (repository root)

```text
src/
├── models/
│   └── item.rs          # gains the pure split helper (top layer /
│                        #   sub-category from a category string)
└── ui/
    ├── state.rs         # PanelRow gains the group variant; left_rows /
    │                    #   right rows build the two-layer tree;
    │                    #   collapse + selection at both layers;
    │                    #   sticky headers per enclosing level
    ├── screens/
    │   └── main.rs      # both panel renderers draw the two-layer tree
    │                    #   (indentation for the sub-layer)
    └── tests/
        └── state.rs     # the new state-layer tests
```

**Structure Decision**: The split helper lives on the model (`Item`/category level) because it is a data rule; the row model, collapse, and rendering stay in the UI state layer where the single-layer logic already lives. No new modules — the existing ones gain the second layer.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None. | | |
