# Implementation Plan: Harmonized Navigation Text

**Branch**: `023-harmonized-navigation-text` | **Date**: 2026-09-21 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/023-harmonized-navigation-text/spec.md`

## Summary

Give every key exactly one canonical action text, defined once and referenced by every surface's hint line and the help screen; remove the same-screen duplication where both main panels restate their shared bindings. The canonical bindings live in one table in the hints module; each surface's hint line is composed from that table; the help screen's key tokens and action words derive from the same table. No key behavior changes — only the displayed text and its single-source definition.

## Technical Context

**Language/Version**: Rust 1.98 (2024 edition)

**Primary Dependencies**: ratatui 0.30, crossterm 0.29 (unchanged)

**Storage**: None — display text only; nothing persisted.

**Testing**: `cargo test` — the existing hint contract tests in `src/ui/tests/hints.rs` are extended: a canonical-uniqueness test (each key's action text is identical everywhere it appears) and a no-duplication test (no pair twice on one screen). The full gate (`cargo check`, `cargo clippy`, `cargo clippy --tests`, `cargo test`, `cargo fmt --check`) must stay green.

**Target Platform**: Terminal TUI (Linux)

**Project Type**: CLI/TUI application (single binary)

**Performance Goals**: Unchanged.

**Constraints**: Warning-free gate; no behavior change (FR-006); the adaptive hint layout (`hint_rows` packing, `collapsed_hint_line`) is untouched (FR-007); the help screen may stay verbose but must not contradict the canonical wording (FR-004).

**Scale/Scope**: One canonical table (~20 entries), nine surface compositions, the help screen's agreement, and the contract tests. No new modules.

## Constitution Check

- **Principle I** (no unused code): satisfied — the canonical table replaces the per-surface literal lists; no dead entries remain (a canonical entry with no referencing surface is removed).
- **Principle II** (zero warnings): satisfied — gate green per task.
- **Principle III** (documented and tested): satisfied — the canonical table is documented; the uniqueness and no-duplication properties are pinned by tests; `TUI_USAGE.md` is updated where it restates hint wording.
- No violations.

## Project Structure

### Documentation (this feature)

```text
specs/023-harmonized-navigation-text/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output (the canonical binding entity)
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (the canonical binding contract)
└── tasks.md             # /speckit-tasks output
```

### Source Code (repository root)

```text
src/
└── ui/
    ├── hints.rs         # the canonical binding table + surface
    │                    #   compositions (one source for everything)
    ├── help.rs          # key tokens and action words derived from the
    │                    #   canonical table (long descriptions stay)
    └── tests/
        └── hints.rs     # the extended contract tests
```

**Structure Decision**: The canonical table lives in `src/ui/hints.rs` next to the surface compositions — one module owns all navigation text. The help screen keeps its own verbose descriptions but references the canonical key tokens and action words; no new files.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None. | | |
