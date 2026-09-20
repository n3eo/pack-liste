# Implementation Plan: Weather Box Single Line

**Branch**: `019-weather-box-single-line` | **Date**: 2026-09-20 | **Spec**: [spec.md](spec.md)

## Summary

Restore the welcome screen's uniform compact field layout: the
weather box gets the same fixed three-line height as the packing
-list-file, weekdays, and travel-style boxes, instead of stretching
to the bottom of the screen. The freed space below the fields stays
empty; the hint footer remains fully visible.

**Sequencing note**: Independent; can land in any order. One
constraint change in one layout call.

## Technical Context

**Language/Version**: Rust 1.98 (2024 edition)

**Primary Dependencies**: ratatui 0.30 (unchanged)

**Storage**: None — layout only.

**Testing**: `cargo test` — 42 existing tests; verification is visual
(the quickstart's multi-size check): the weather box's height equals
the other fields' at every size, no field touches the bottom or the
hint footer.

**Target Platform**: Terminal TUI (Linux)

**Project Type**: CLI/TUI application (single binary)

**Performance Goals**: Unchanged.

**Constraints**: Warning-free gate; the four fields render as equal
compact blocks; the hint footer stays visible at all supported
sizes.

**Scale/Scope**: One constraint in `draw_welcome_screen`'s
`params_layout`.

## Constitution Check

- **Principle I**: satisfied — a constraint change; no code added.
- **Principle II**: satisfied — gate clean.
- **Principle III**: satisfied — the layout behavior is covered by
  the quickstart's size checks.
- No violations.

## Project Structure

```text
src/
└── ui/
    └── screens/
        └── welcome.rs   # params_layout's fourth constraint:
                         #   Constraint::Min(3) → Constraint::Length(3)
```

**Structure Decision**: The four field constraints become four
`Length(3)` values — uniform, fixed, compact; the parent
`content_area` (already `Min(12)`) absorbs the freed space.

## Phase 0 Research

- **D1 — The cause (verified)**: `params_layout` uses
  `[Length(3), Length(3), Length(3), Min(3)]` — the weather field's
  `Min(3)` grows to fill the content area (the activities field's
  old role, left flexible when spec 017 removed activities). The
  fix: `Length(3)`, matching the other three.
- **D2 — No downstream coupling**: nothing reads the weather box's
  height; the resume prompt and hint footer are laid out from the
  outer split, unaffected.

## Phase 1 Design

### Data model

None — layout only.

### Contracts

- The four welcome fields are equal-height compact blocks at every
  terminal size; the space below them is empty; the hint footer is
  fully visible.

### Quickstart

- Open the welcome screen at 80x24, 120x40, and a tall size: the
  weather box matches the other fields' height at every size; no
  field touches the screen bottom or the footer.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None. | | |
