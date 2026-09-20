---
description: "Task list for 006-responsive-small-terminals"
---

# Tasks: Responsive Small Terminal UI

**Input**: Design documents from `/specs/006-responsive-small-terminals/`

**Prerequisites**: plan.md (required), spec.md (required for user stories)

**Tests**: Included per story (threshold and truncation decisions are pure functions — testable).

**Organization**: The largest UI feature; lands after 007 (composes with its panel reuse).

## Format: `[ID] [P?] [Story] Description`

---

## Phase 1: Layout Mode (US1, US4)

**Goal**: The main screen adapts: side-by-side, stacked, or the too-small notice.

**Independent Test**: 80x24 renders completely side-by-side; narrower switches to stacked with both lists full-width and navigable (US1/AC1-2); very small shows the notice (US4/AC2-3).

- [X] T001 [US1] Add `LayoutMode { SideBySide, Stacked, TooSmall }` and the pure `layout_mode(width, height)` decision in `src/ui/screens/main.rs` (plan D2: the threshold is 80 columns — two 39-col minimum panels + margins; the too-small floor per FR-008's framing)
- [X] T002 [US1] Branch `draw_main_screen`: stacked splits vertically instead of horizontally — both panel renderers at full width (plan D1's reuse); `TooSmall` renders only the readable notice (plan D4)
- [X] T003 [US1] Add threshold tests in `src/ui/tests/state.rs`: the mode at 80/79 columns (the boundary, stable — the spec's flicker edge case), the too-small floor, and the pure-function property (same inputs → same mode)

**Checkpoint**: The layout serves every size.

---

## Phase 2: Long Text and Popups (US1, US2, US3)

**Goal**: Nothing truncates silently; every popup fits.

**Independent Test**: A long name at narrow width shows a visible truncation indicator (US1/AC3); each popup fits and stays answerable at 80x24 and below (US3/AC1-3).

- [X] T004 [US1] Add width-aware truncation with a visible `…` indicator to the row renderers in `src/ui/screens/main.rs` (names, categories) and the welcome file path in `src/ui/screens/welcome.rs` (plan D6); add truncation tests
- [X] T005 [US3] Audit every popup's sizing in `src/ui/draw.rs` and `src/ui/screens/welcome.rs` (add/edit dialog, delete/edit confirmations, modifier editor, resume prompt): `min()`-clamp width and height against the terminal at every use (FR-003); the PTY check at 80x24 and below
- [X] T006 [US2] Verify every screen at 80x24 (FR-001) — `src/ui/screens/welcome.rs`, `src/ui/screens/help.rs`, `src/ui/screens/config.rs`, `src/ui/screens/editor.rs` — via the PTY size sweep; fix any overflow found

**Checkpoint**: Small terminals are first-class.

---

## Phase 3: Hint Collapse (US5)

**Goal**: The navigation explanation collapses to a short `?` pointer when space is tight.

**Independent Test**: Large → the full hints; tight → the single `? Help` line with the lists keeping their space; `?` opens help at every size (US5/AC1-4).

- [X] T007 [US5] Add the collapse rule to the hint system in `src/ui/hints.rs` (plan D3): when the packed rows would squeeze the lists (the hints area's height vs the list minimum), render the single short `? Help` line instead; the decision is a pure function of (width, height, entries) — stable at the boundary
- [X] T008 [US5] Add collapse tests: full hints when they fit, the short line when they do not, the switch stable at the threshold, `?` binding unchanged (FR-011); update the hint-width tests if the entries change

**Checkpoint**: The explanation is always reachable, never in the way.

---

## Phase 4: Resize Robustness (US4) and Verification

- [X] T009 [US4] Verify live resizing (FR-004, plan D5): the poll loop redraws per cycle; scroll offsets clamp per frame (the existing `min()` chains); the PTY resize sweep — shrink while deep-scrolled, grow from the notice — no crash, no data loss, the previous screen returns intact (US4/AC1-3)
- [X] T010 Update `TUI_USAGE.md` with the supported sizes, the stacked mode, and the hint collapse
- [X] T011 Final gate: `cargo check`, `cargo clippy`, `cargo clippy --tests`, `cargo test`, `cargo fmt --check` all exit 0; the quickstart's full size-sweep passes

---

## Dependencies & Execution Order

- T001 → T002 → T003 (the mode before the layout before the tests); T004–T006 parallelizable after T002; T007–T008 independent of the layout work (the hints module); T009 last.
- **Cross-spec**: after 007 (the fullscreen toggle composes — one panel either way); the hint work extends spec 016's `hint_rows` (plan D3).

## Notes

- The 42 existing tests are the contract; the hint tests (`hint_rows_fit_width_and_keep_every_entry`) must keep passing — the collapse is an additional rule above the packing, not a replacement.
- FR-007: the same key bindings at every size — no size-conditional bindings.
