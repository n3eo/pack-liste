# Implementation Plan: Responsive Small Terminal UI

**Branch**: `006-responsive-small-terminals` | **Date**: 2026-09-20 | **Spec**: [spec.md](spec.md)

## Summary

Make every screen work at small terminal sizes. The main screen
switches from side-by-side to a vertical stacked arrangement below a
width threshold (both lists full-width, navigable); every screen and
popup fits 80x24; long names truncate with a visible indicator; the
navigation hints collapse to a short `?` pointer when space is tight
(US5, the refinement); below the minimum usable size a readable
"terminal too small" notice replaces broken content; live resizing
never crashes, corrupts, or loses data.

**Sequencing note**: The largest of the UI features. Lands after 007
(fullscreen — the stacked layout and the fullscreen toggle compose:
fullscreen is one panel either way) and independently of the
data-model features. The hint-collapse work (US5) extends the
existing `hint_rows` packing (spec 016) rather than replacing it.

## Technical Context

**Language/Version**: Rust 1.98 (2024 edition)

**Primary Dependencies**: ratatui 0.30, crossterm 0.29 (unchanged —
crossterm's Resize event drives FR-004)

**Storage**: None — layout is derived per frame from the viewport.

**Testing**: `cargo test` — 42 existing tests; new tests: the
stacked-layout threshold decision (pure function of width), the
hint-collapse decision (US5's threshold, stable at the boundary),
truncation with indicator. PTY-based size checks in the quickstart
(80x24, stacked width, below-minimum).

**Target Platform**: Terminal TUI (Linux)

**Project Type**: CLI/TUI application (single binary)

**Performance Goals**: Re-render on resize within one poll cycle
(100 ms); no layout computation beyond per-frame O(rows).

**Constraints**: Warning-free gate; full usability down to 80x24
(FR-008); below that only the notice and data safety apply (FR-005);
the same key bindings at every size (FR-007); the hint switch is
stable at the threshold (no flicker, the spec's edge case).

**Scale/Scope**: A layout-mode decision in `draw_main_screen`
(side-by-side vs stacked vs notice), popup `min()` sizing audits,
long-text truncation in the row renderers, the hint-collapse rule,
and the Resize event path.

## Constitution Check

- **Principle I**: satisfied — all layout code is reachable at the
  sizes it serves.
- **Principle II**: satisfied — gate clean per task.
- **Principle III**: satisfied — the stacked layout and hint
  collapse are documented (TUI_USAGE) and tested.
- No violations.

## Project Structure

```text
src/
└── ui/
    ├── screens/
    │   ├── main.rs       # layout mode: side-by-side / stacked / notice;
    │   │                 #   row truncation with "…" indicator
    │   ├── welcome.rs    # long-path truncation (already inner-clamped;
    │   │                 #   verify at 80 cols)
    │   └── (popups in draw.rs)  # min() sizing audit per popup
    ├── hints.rs          # the collapse rule: full rows when they fit
    │                     #   without squeezing the lists, else the short
    │                     #   "? Help" line (US5)
    ├── input.rs          # the Resize event re-renders (the poll loop
    │                     #   already redraws each cycle)
    └── tests/
        └── state.rs      # threshold + truncation + collapse tests
```

**Structure Decision**: The layout mode is a pure function of the
viewport (`fn layout_mode(width, height) -> LayoutMode`), decided in
the renderer — no persisted state, no resize bookkeeping. The
stacked arrangement reuses both panel renderers with different
areas (the same reuse as 007's fullscreen).

## Phase 0 Research

- **D1 — Current layout (verified)**: `draw_main_screen` splits
  horizontally 50/50 unconditionally (`screens/main.rs:21`); below
  ~80 columns the two panels are too narrow for the row content
  (selection symbol + name + quantity). The stacked mode splits
  vertically instead — each panel full-width.
- **D2 — Threshold**: side-by-side needs ~2× the minimum panel
  width. The hint system's minimum panel width is 39 columns (the
  hint tests); the threshold is 80 columns (two 39-col panels +
  margins) — at or below 80 the stacked mode engages. The exact
  value is a contract decision; 80 aligns with FR-001/FR-008.
- **D3 — Hint collapse (US5, the refinement)**: `hint_rows` already
  packs entries into rows that fit the width (spec 016). The
  refinement adds a vertical dimension: when the packed rows would
  squeeze the lists (the hints area's height vs the list minimum),
  the hints collapse to a single `("? Help", …)` line. The switch
  is a pure function of (width, height, entries) — stable at the
  boundary because it is recomputed per frame from the same inputs.
- **D4 — Minimum-size notice (US4)**: below the minimum usable size
  (below 80x24 per FR-008's framing — the notice applies below the
  smallest size the layouts can serve), the renderer draws only the
  notice; all state is untouched (the draw path never mutates
  beyond the existing scroll write-backs).
- **D5 — Resize handling (FR-004)**: the run loop redraws every
  poll cycle (100 ms); a Resize event needs no special handling
  beyond the next draw — the viewport is read per frame. Scroll
  offsets are clamped per frame already (the `min()` chains in the
  panel renderers).
- **D6 — Long text (FR-006)**: the row renderers draw name +
  quantity without width awareness; truncation with a visible `…`
  indicator at the panel width, applied to names, categories, and
  the welcome file path (the path field already clamps its inner
  area; verify at 80 cols).

## Phase 1 Design

### Data model

```text
LayoutMode { SideBySide, Stacked, TooSmall }
fn layout_mode(width: u16, height: u16) -> LayoutMode   // pure
```

### Contracts

- 80x24: every screen renders completely (FR-001); the main screen
  is side-by-side at ≥ the threshold, stacked below it (FR-002).
- Hints: full rows when they fit without squeezing; the short
  `? Help` line otherwise (FR-009/FR-010); `?` opens help at every
  size (FR-011, existing binding).
- Popups: fit and stay answerable at 80x24 and below (FR-003).
- Below minimum: the notice only; no data loss (FR-005).

### Quickstart

- 80x24: all screens complete; main side-by-side.
- Narrower: stacked, both lists full-width and navigable.
- Very small: the notice; grow back → the previous screen intact.
- Deep-scrolled list + shrink: no crash, cursor clamped into view.
- Hints: full at large, the short `?` line when tight, `?` works
  at every size.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None. | | |
