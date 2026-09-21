---
description: "Task list for 024-two-layer-category-nesting"
---

# Tasks: Two-Layer Category Nesting

**Input**: Design documents from `/specs/024-two-layer-category-nesting/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Included per story — the spec's success criteria are testable properties (Constitution III); the row model and layer interactions are state-testable, the rendering is PTY-verified.

**Organization**: Tasks are grouped by user story. US1 (the tree) and US2 (the interactions) are both P1; US1 lands first because US2's collapse/selection operate on the tree US1 builds. US3 (format freeze) is mostly verification — it lands last but constrains every task.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

Single project: `src/` at repository root. Model work in `src/models/item.rs`; state/row work in `src/ui/state.rs`; rendering in `src/ui/screens/main.rs`; tests in `src/ui/tests/state.rs`.

---

## Phase 1: Foundational — The Split and the Row Model

**Purpose**: The pure derivation and the two-layer row list every story builds on (FR-001, research R1/R2).

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [X] T001 Add the pure split helper in `src/models/item.rs`: `category_layers(category: &str) -> (&str, Option<&str>)` returning the top layer (before the FIRST ": ", or the whole string) and the sub-category (everything after the first ": ", verbatim, or `None`); the function never mutates or rewrites the input (FR-001, FR-003)
- [X] T002 Add the split tests in `src/ui/tests/state.rs` (`category_split_*`): "Ausland: Karten" → ("Ausland", Some("Karten")); "A: B: C" → ("A", Some("B: C")); "OhneTrenner" → ("OhneTrenner", None); "Kulturbeutel: Kulturbeutel" → ("Kulturbeutel", Some("Kulturbeutel")); the input string is unchanged
- [X] T003 Extend `PanelRow` in `src/ui/state.rs` with the `Group(String)` variant (the top-layer name); `Category(String)` keeps carrying the FULL category string as its identity (research R2)
- [X] T004 Rebuild `left_rows` in `src/ui/state.rs` as the two-layer tree: rows in group order — `Group(top)`, then per sub-category `Category(full)` followed by its items (unless the sub-category is collapsed); a collapsed group contributes only its `Group` row; separator-free categories contribute `Group(name)` + items directly (no `Category` row, FR-003); items with multiple categories appear under each sub-category they belong to (existing behavior)
- [X] T005 Rebuild the right panel's row building in `src/ui/state.rs` (`right_item_order` / the drawn-rows construction used by `draw_selected_items_panel`) as the same two-layer tree over the selected items, grouped by primary category then split (FR-002, research R7)
- [X] T006 Add the row-model tests in `src/ui/tests/state.rs` (`two_layer_rows_*`): categories sharing a top layer produce one `Group` row followed by their `Category` rows in order; the user's example ("Ausland: Karten" + "Ausland: Handgepäck" with items) produces Group "Ausland", Category "Ausland: Karten", its items, Category "Ausland: Handgepäck", its items; the right panel builds the same tree over selected items

**Checkpoint**: The gate is green; both panels' row lists are the two-layer tree; the existing navigation tests pass (the cursor now also walks `Group` rows — update the row-position expectations in the affected tests).

---

## Phase 2: User Story 1 — The tree renders (Priority: P1) 🎯 MVP

**Goal**: Both panels display the two-layer tree: group headers, indented sub-category headers, items beneath (SC-001, SC-002).

**Independent Test**: The quickstart §5 visual check — the left panel shows `▼ Ausland` with indented `▼ Karten` / `▼ Handgepäck` beneath it and items beneath those; the right panel shows the same tree.

### Implementation for User Story 1

- [X] T007 [US1] Render the `Group` rows in `draw_all_items_panel` (`src/ui/screens/main.rs`): the group header shows the top layer with the existing collapse symbol (▶/▼ driven by the group's collapsed state, Phase 3) and the existing truncation; `Category` rows render the part after ": " indented two spaces under the group; `Item` rows indent one more level
- [X] T008 [US1] Render the `Group` rows in `draw_selected_items_panel` (`src/ui/screens/main.rs`): the group header carries the compiled list's packing indicator (all-packed ✓ / partial ○, computed over every selected item under the group); sub-category headers keep their per-category indicator, indented
- [X] T009 [US1] Extend `sticky_rows` in `src/ui/state.rs` per research R5: when the viewport starts mid-tree, pin the OUTERMOST scrolled-off header — a `Group` row when the window starts inside a group whose header is off-screen (even if a `Category` row is the immediate predecessor), a `Category` row when only the sub-category's header is missing; one pinned row maximum, as today (FR-009)
- [X] T010 [US1] Add the sticky tests in `src/ui/tests/state.rs`: a window starting on an item pins its sub-category header; a window starting on a sub-category header whose group header is off-screen pins the group header; the pinned row never exceeds the viewport
- [X] T011 [US1] PTY-verify the rendered tree: run the app against the reference trip, capture the main screen — the groups appear as top-level headers with their sub-categories indented beneath, items beneath those; scroll deep and confirm the pinned header (quickstart §5)

**Checkpoint**: US1 done — the tree is visible and scroll-safe on both panels.

---

## Phase 3: User Story 2 — Navigation, collapse, selection on both layers (Priority: P1)

**Goal**: The cursor walks the tree; collapse and selection work per layer (SC-003, SC-004).

**Independent Test**: The quickstart §3 tests and §5 checks — collapse a group (the sub-tree hides), collapse a sub-category (only its items hide), `Ctrl+Space` a group (everything beneath flips), navigation never lands on a hidden row.

### Implementation for User Story 2

- [X] T012 [US2] Add `collapsed_groups: Vec<String>` to `AppDrawState` in `src/ui/state.rs` (session state, not persisted; initialize in `src/ui/input.rs` and `src/ui/tests/mod.rs`); extend `set_category_collapsed`/`toggle_category_collapse` to resolve by cursor row: a `Group` row toggles `collapsed_groups`, a `Category` row toggles `collapsed_categories` (unchanged semantics); the cursor clamps to the header it collapsed (FR-005)
- [X] T013 [US2] Extend `toggle_category_selection` in `src/ui/state.rs`: on a `Group` row, flip every item whose ANY category's top layer matches the group (FR-006); on a `Category` row, keep today's per-full-string behavior
- [X] T014 [US2] Extend the recalculate scope resolution: add `RecalculateScope::Group(String)` in `src/ui/state.rs` (recalculates every item whose category's top layer matches — research R6); resolve plain `R` and `Ctrl+R` on a `Group` header to the group scope in `src/ui/handlers/welcome.rs` / `src/ui/input.rs`; `Category` keeps its full-string meaning; `All` unchanged (FR-010)
- [X] T015 [US2] Extend `sync_left_view_to_right_cursor` in `src/ui/input.rs`: expand the cursor item's sub-category AND its group on the left, collapsing others (research R7)
- [X] T016 [US2] Add the interaction tests in `src/ui/tests/state.rs` (`group_collapse_*`, `group_selection_*`, `group_recalc_*`): collapsing a group hides its sub-tree and keeps the cursor on the group header; collapsing a sub-category hides only its items; group selection flips every item under the group (across sub-categories); the group recalc scope covers every item under the group and nothing else; navigation over a collapsed group visits no hidden row
- [X] T017 [US2] PTY-verify the interactions: collapse a group and a sub-category, select a group with `Ctrl+Space`, recalculate from a group header — each behaves per the contract's interaction table (quickstart §5)

**Checkpoint**: US2 done — the tree is fully operable at both layers.

---

## Phase 4: User Story 3 — The stored format is unchanged (Priority: P2)

**Goal**: The category strings persist verbatim; the nesting is display-only (SC-005).

**Independent Test**: The quickstart §4 round trip — the config's category strings are byte-identical before and after a save-load cycle.

### Implementation for User Story 3

- [X] T018 [US3] Verify the persistence paths touch no layer data: `save_config`/`save_trip_file`/`load_config` in `src/ui/input.rs` + `src/ui/config_io.rs` serialize the category strings verbatim; the split helper is used only by the display path (grep: `category_layers` appears only in `src/models/item.rs`, `src/ui/state.rs`, `src/ui/screens/main.rs`, and the tests)
- [X] T019 [US3] Verify the add/edit dialog's category picker in `src/ui/handlers/dialogs.rs` + `src/ui/draw.rs` still lists and assigns the full category strings verbatim (FR-008); optionally dim the group prefix in the displayed entry — the assigned value stays the full string
- [X] T020 [US3] Run the format round trip per quickstart §4: copy the config, run the app (navigate, quit — saves), diff the category strings — byte-identical

**Checkpoint**: US3 done — the frozen contract holds.

---

## Phase 5: Polish & Cross-Cutting Concerns

- [X] T021 Update `TUI_USAGE.md`: the two-layer display (the split rule, the indented tree, collapse/selection at both layers) and the group-level recalculate
- [X] T022 Run the quickstart validation end to end ([quickstart.md](quickstart.md) §1–§6): the split/row/collapse/selection tests, the round trip, the visual checks, the full regression
- [X] T023 Final gate: `cargo check`, `cargo clippy`, `cargo clippy --tests`, `cargo test`, `cargo fmt --check` all exit 0 with zero warnings

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Foundational)**: blocks everything — the split and row model are the substrate.
- **Phase 2 (US1)**: depends on Phase 1 (renders its rows).
- **Phase 3 (US2)**: depends on Phase 2 (collapses/selection operate on the rendered tree; the group header's collapse symbol needs the group state).
- **Phase 4 (US3)**: verification-only; depends on Phase 3 (the full feature is in place to verify against).
- **Phase 5**: depends on all stories.

### Within Each User Story

- Tests with the implementation they pin (T002 with T001; T006 with T004/T005; T010 with T009; T016 with T012–T015).
- The row model (T004/T005) before the rendering (T007/T008); the state before the handlers (T012–T014 before T015).

### Parallel Opportunities

- T001/T002 (model + its tests) are parallel-safe with T003 (different concerns in different areas — sequence in practice, same crate).
- T007/T008 (the two panel renderers) are parallel-safe once T004/T005 exist (different functions in the same file — sequence in practice).
- T018/T019 (verification of disjoint paths) are parallel-safe.
- T021 (docs) is parallel-safe with T018–T020.

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1 (the split + the row model).
2. Complete Phase 2 (US1 — the tree renders on both panels).
3. **STOP and VALIDATE**: the visual check shows the user's example tree; the sticky headers hold.
4. The user's core request (the visible nesting) is delivered; the interactions follow.

### Incremental Delivery

1. Phase 1 → the derivation and row model exist, pinned by tests.
2. Phase 2 (US1) → the tree is visible.
3. Phase 3 (US2) → the tree is operable at both layers.
4. Phase 4 (US3) → the format freeze is verified.
5. Phase 5 → docs + quickstart + gate.

---

## Notes

- The existing state tests are the contract: the navigation/collapse/selection/sticky tests must keep passing, with their row-position expectations updated for the `Group` rows in the same task that changes the row model (never weakened).
- The stored category strings are frozen (FR-007): any task that would rewrite, migrate, or re-serialize them is wrong.
- No new key bindings: `C`, `Ctrl+Space`, `R`, `Ctrl+R` extend to the group level; nothing new is introduced.
- The `collapsed_groups` state is session-only — never persisted (it is not part of the frozen format).
- Commit after each phase; Conventional Commits (`feat:` for the tree and interactions, `test:` where a task is purely tests, `docs:` for TUI_USAGE.md).
