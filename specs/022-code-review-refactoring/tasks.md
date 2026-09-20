---
description: "Task list for 022-code-review-refactoring"
---

# Tasks: Code Review Remediation

**Input**: Design documents from `/specs/022-code-review-refactoring/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: The spec requires behavior preservation (FR-011) — the 39 existing tests are the contract and MUST pass unmodified after every task. New tests are required only where the spec demands them (US5 error surfacing, per constitution Principle III "bug fixes come with a test"; the status line is new behavior and ships with tests).

**Organization**: Tasks are grouped by user story, in the spec's priority order. Story order is also the implementation order (research R10): US1 split → US2 dedup → US3 dead code → US4 renames → US5 errors → US6 docs.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1–US6)
- Include exact file paths in descriptions

## Path Conventions

Single project: `src/` at repository root. All UI work happens under `src/ui/` (target tree in [contracts/module-structure.md](contracts/module-structure.md)).

---

## Phase 1: Setup

**Purpose**: Baseline capture before any refactor step.

- [X] T001 Capture pre-refactor baseline: run `cargo test` and record the 39 passing test names; save a reference `config.yaml` and trip file produced by the current binary (e.g., under `/tmp/pack-liste-verify/`) for the format round-trip check in quickstart.md §2
- [X] T002 Create the target module skeleton: empty `src/ui/state.rs`, `src/ui/input.rs`, `src/ui/handlers/mod.rs`, `src/ui/draw.rs`, `src/ui/screens/mod.rs`, `src/ui/hints.rs`, `src/ui/help.rs`, `src/ui/config_io.rs`, `src/ui/tests/mod.rs` with `mod` declarations wired in `src/ui/mod.rs` (contents move in US1; skeleton compiles empty)

**Checkpoint**: Gate green (`cargo check`, `cargo clippy`, `cargo clippy --tests`, `cargo test`, `cargo fmt --check`), baseline recorded.

---

## Phase 2: User Story 1 — Split the monolith into cohesive modules (Priority: P1) 🎯 MVP

**Goal**: `src/ui/app.rs` (4,917 lines) is dissolved into the concern-separated tree from [contracts/module-structure.md](contracts/module-structure.md); behavior identical.

**Independent Test**: Full gate green; all 39 tests pass with assertions unmodified (test location may move); manual smoke pass (quickstart.md §6) shows identical behavior on every screen.

### Implementation for User Story 1

- [X] T003 [US1] Move the state layer to `src/ui/state.rs`: `AppDrawState`, `Screen`, `Focus`, `FocusSide`, `InputField`, `InputMode`, `AppState`, `PopupFields`, `TripParams`, `ConfigSection`, `ModifierField`/editor types, and the state-operation methods (`active_rows`, `apply_item_edit`, `move_cursor_up/down`, `sync_left_view_to_right_cursor`, selection/collapse ops) — from `src/ui/app.rs`
- [X] T004 [US1] Move config I/O to `src/ui/config_io.rs`: `AppConfig`, `SavedItem` (with its custom `Deserialize` impl), `default_travel_style_name`, `default_weather_name`, `load_config`, `create_default_config`, and the free functions they depend on — from `src/ui/app.rs`
- [X] T005 [US1] Move hint/help content to `src/ui/hints.rs` and `src/ui/help.rs`: `HintSurface`, `hint_entries`, `help_binding_tokens`, `help_screen_text`, hint rendering helpers — from `src/ui/app.rs`
- [X] T006 [US1] Move per-screen drawing to `src/ui/screens/`: `welcome.rs` (`draw_welcome_screen`), `main.rs` (`draw_main_screen` and panel rendering), `help.rs` (`draw_help_screen`), `config.rs` (`draw_config_screen`, modifier editor rendering), `editor.rs` (`draw_config_editor_screen`), plus `screens/mod.rs` re-exports — from `src/ui/app.rs`
- [X] T007 [US1] Move the remaining draw dispatch and shared popup rendering to `src/ui/draw.rs`: `App::draw` (keeping the per-frame `draw_state.clone()` and scroll write-back, research R8), `draw_confirm_delete_popup`, and the shared draw helpers — from `src/ui/app.rs`
- [X] T008 [US1] Move input handling to `src/ui/input.rs` and `src/ui/handlers/`: `handle_input` and its per-screen branches to `handlers/welcome.rs`, `handlers/main.rs`, `handlers/dialogs.rs` (add/edit/delete popups), `handlers/config.rs` (config screen + modifier editor), `handlers/editor.rs` (raw config editor); `App::run` and `App::new` stay with `App` — from `src/ui/app.rs`
- [X] T009 [US1] Move the 33 tests from the `mod tests` block (currently lines 233–1196, before the production code) into `src/ui/tests/` split by subject: `state.rs`, `input.rs`, `hints.rs`, `config_io.rs` — assertions unchanged, only `use` paths updated
- [X] T010 [US1] Delete `src/ui/app.rs`; make `src/ui/mod.rs` re-export only `App` (the sole type `src/app.rs` consumes); update `src/app.rs` import path if needed
- [X] T011 [US1] Verify US1: full gate green, all 39 tests pass unmodified, no file exceeds 1,500 lines (`find src -name "*.rs" | xargs wc -l`), manual smoke pass per quickstart.md §6

**Checkpoint**: The monolith is gone; every screen behaves identically. MVP delivered — the codebase is navigable even if later stories stop here.

---

## Phase 3: User Story 2 — Decompose input handling and remove duplication (Priority: P1)

**Goal**: The ~473-line `handle_input` becomes a two-level dispatcher (research R4); the four duplication sites (research R5) each have one shared implementation.

**Independent Test**: Full gate green; help/hint contract tests pass unmodified (the binding set is unchanged); grep shows the duplicated names gone (quickstart.md §3).

### Implementation for User Story 2

- [X] T012 [US2] Introduce `HandlerOutcome { Handled, Exit }` in `src/ui/input.rs` and convert `handle_input` to a two-level dispatcher: global keys first (Ctrl+C, Ctrl+E, q, ?, O), then `match screen` routing to `handlers/<screen>.rs`, each handler matching on `input_mode`/focus; no handler exceeds 150 lines
- [X] T013 [US2] Consolidate `get_unique_categories` + `get_all_categories` (identical bodies) into one method on `AppDrawState` in `src/ui/state.rs`; update all call sites
- [X] T014 [US2] Unify `draw_add_item_popup` + `draw_edit_item_popup` into one `draw_item_dialog` parameterized by the existing `PopupFields` (title and Mode field via `auto_quantity: Option<bool>`); place it in `src/ui/screens/` (dialog rendering); update `App::draw` dispatch in `src/ui/draw.rs`
- [X] T015 [US2] Extract the 4× category-grouping construction (`BTreeMap<String, Vec<...>>` at former lines 1298, 1324, 4150, 4347) into one `group_items_by_category` helper in `src/ui/state.rs`; update the four call sites in `screens/main.rs` and `state.rs`
- [X] T016 [US2] Unify `cycle_travel_style` + `cycle_weather` into one `cycle_value(list, current, direction)` helper in `src/ui/state.rs`; update the welcome-screen handler call sites
- [X] T017 [US2] Verify US2: full gate green, all 39 tests pass unmodified, `grep -rn "get_all_categories\|get_unique_categories\|draw_add_item_popup\|draw_edit_item_popup\|cycle_travel_style\|cycle_weather" src/` returns nothing, manual smoke pass on main + dialogs + welcome

**Checkpoint**: Input handling is navigable per screen; duplication is gone; bindings unchanged.

---

## Phase 4: User Story 3 — Remove dead fields and dependencies (Priority: P2)

**Goal**: The five never-read model fields and the `chrono` dependency are gone (research R1); persisted formats unaffected.

**Independent Test**: Full gate green; format round-trip (quickstart.md §2) loads the pre-refactor files identically; `grep -rn "chrono" Cargo.toml` empty.

### Implementation for User Story 3

- [X] T018 [US3] Remove `Item.id` and `Item.base_quantity` from `src/models/item.rs`; change `Item::new` to `Item::new(name, categories, quantity)`; update all call sites (`save_new_item` in `src/ui/handlers/dialogs.rs`, `load_config`/`create_default_config` in `src/ui/config_io.rs`, test helpers in `src/ui/tests/`)
- [X] T019 [US3] Remove `PackingList.id`, `PackingList.name`, `PackingList.file_path` from `src/models/packing_list.rs`; make `PackingList::new` parameterless (or replace with `Default`); update call sites in `src/ui/config_io.rs` and `src/ui/tests/`
- [X] T020 [US3] Remove `chrono` from `Cargo.toml` `[dependencies]`; run `cargo build` so `Cargo.lock` updates via cargo (never hand-edit); remove `mockall` from `[dev-dependencies]` (zero usage, research R9)
- [X] T021 [US3] Verify US3: full gate green, all 39 tests pass, format round-trip per quickstart.md §2 (pre-refactor `config.yaml` + trip file load identically), `grep -rn "chrono\|mockall" Cargo.toml` empty, `grep -rn "base_quantity\|\.id\b" src/models/` clean

**Checkpoint**: The model contains only used fields; the dependency footprint shrank; files load identically.

---

## Phase 5: User Story 4 — Names say what the code means (Priority: P2)

**Goal**: `Item.editable` → `Item.selected` (research R2); visibility tightened to actual use.

**Independent Test**: Full gate green; `grep -rn "editable" src/` empty; selection default (all items pre-selected on a new list, data-model V2) still holds via existing tests.

### Implementation for User Story 4

- [X] T022 [US4] Rename `Item.editable` → `Item.selected` in `src/models/item.rs` and update all references (28 sites, now spread across `src/ui/state.rs`, `src/ui/handlers/`, `src/ui/screens/main.rs`, `src/ui/tests/`); the field means "selected for the trip"
- [X] T023 [US4] Tighten visibility: in `src/ui/state.rs`, `src/ui/input.rs`, `src/ui/handlers/`, `src/ui/screens/`, `src/ui/config_io.rs`, `src/ui/hints.rs`, `src/ui/help.rs` reduce `pub` to `pub(crate)` or private everywhere except the `App` re-export in `src/ui/mod.rs`; use `#[cfg(test)]`/`pub(crate)` for test access instead of blanket `pub`
- [X] T024 [US4] Verify US4: full gate green, all 39 tests pass unmodified, `grep -rn "editable" src/` empty, `grep -c "pub " src/ui/*.rs src/ui/**/*.rs` shows only justified public items, manual smoke pass (selection toggling on both panels)

**Checkpoint**: Names are honest; visibility reflects real use; behavior unchanged.

---

## Phase 6: User Story 5 — Errors are handled, not swallowed (Priority: P2)

**Goal**: The six `.ok()` sites surface failures via a status line (research R6); the production `unwrap()` is gone; `main.rs` reports errors once; unused tracing removed (research R7).

**Independent Test**: Full gate green; forced save failure (quickstart.md §5) shows a visible error and no crash; new tests assert the status line; `grep -rn "\.ok();" src/ui/` empty on save/recalculate paths.

### Implementation for User Story 5

- [X] T025 [US5] Add `StatusMessage { message: String, kind: StatusKind }` with `StatusKind { Error, Info }` and a `status: Option<StatusMessage>` field to `AppDrawState` in `src/ui/state.rs`, with a `set_status` helper; clear on mode/screen transition (data-model V3)
- [X] T026 [US5] Render the status line in the footer of the main and config screens (`src/ui/screens/main.rs`, `src/ui/screens/config.rs`), styled by `StatusKind` (Error = visible/red, Info = neutral), without breaking the hint-line layout (hint rows fit width, per the existing hint tests)
- [X] T027 [US5] Replace the six `.ok()` sites with surfaced errors: `save_config`/`update_quantities` after add-item and edit-item (`src/ui/handlers/dialogs.rs`), after modifier-editor save and config-screen changes (`src/ui/handlers/config.rs`); on failure set the status line instead of discarding; propagate with `?` only where the caller already returns `Result` and exiting is acceptable
- [X] T028 [US5] Replace the production `unwrap()` on `modifier_editor.take()` in `src/ui/handlers/config.rs` (former line 2766) with explicit `if let Some(editor) = ... take()` handling
- [X] T029 [US5] Unify error reporting in `src/main.rs`: report the application error exactly once (drop the `eprintln!` + `bail!` double report — return/propagate the error so `main`'s `Result` prints it once)
- [X] T030 [US5] Remove the unused tracing setup: `tracing` and `tracing-subscriber` from `Cargo.toml`, the init block and the `--debug` flag from `src/main.rs` (its only effect was the log filter; zero log statements exist, research R7); update `TUI_USAGE.md` if it documents `--debug`
- [X] T031 [US5] Add tests for the status line in `src/ui/tests/`: a forced save failure sets `status` to an Error message; a successful operation clears it; the message does not survive a mode transition (data-model V3)
- [X] T032 [US5] Verify US5: full gate green, all tests pass (39 + new status tests), forced-failure check per quickstart.md §5 (unwritable path → visible error, no crash), `grep -rn "tracing\|debug" src/main.rs` clean, `grep -n "\.ok();" src/ui/handlers/*.rs` empty

**Checkpoint**: No silent failures; the user sees save/recalculate errors; the entry point reports once.

---

## Phase 7: User Story 6 — Documentation matches the code (Priority: P3)

**Goal**: `docs/quantity-calculation.md` describes the actual post-refactor code; tests sit adjacent to their subjects (already done in T009; verified here).

**Independent Test**: Every structural claim in the docs verifiable against the tree (quickstart.md §7); full gate green.

### Implementation for User Story 6

- [X] T033 [US6] Rewrite `docs/quantity-calculation.md` to match reality: actual module tree (`src/quantity/calculation.rs`, `src/quantity/model.rs`, `src/config/modifiers.rs`, `src/ui/` — no `src/services/`, no `error.rs`), YAML storage (not JSON), actual error handling (anyhow `Result`, the status line), current modifier semantics (buffer/reserve per spec 010, no pre-017 activity multipliers)
- [X] T034 [US6] Update `TUI_USAGE.md` if it references removed CLI flags (`--debug`) or stale behavior; verify `AGENTS.md` commands still match
- [X] T035 [US6] Verify test adjacency: every test module in `src/ui/tests/` imports from the module it tests; no test file references a module it does not exercise
- [X] T036 [US6] Final verification: run the complete quickstart.md (all 7 sections) end to end; confirm SC-001 through SC-007 from spec.md

**Checkpoint**: Documentation is truthful; the full validation suite passes.

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Final sweep across all stories.

- [X] T037 Run `cargo fmt` over the tree; confirm `cargo fmt --check` exits 0
- [X] T038 Confirm the per-frame clone trade-off is documented where it happens: a short comment at the `draw_state.clone()` site in `src/ui/draw.rs` stating why it is kept (research R8, FR-012)
- [X] T039 Final gate: `cargo check`, `cargo clippy`, `cargo clippy --tests`, `cargo test`, `cargo fmt --check` all exit 0 with zero warnings; commit per Conventional Commits (`refactor:` for the structural work)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately.
- **US1 (Phase 2)**: Depends on Setup. Blocks all other stories (they modify the split modules).
- **US2 (Phase 3)**: Depends on US1 (handlers/screens exist as modules).
- **US3 (Phase 4)**: Depends on US2 (call sites consolidated before the model shrinks).
- **US4 (Phase 5)**: Depends on US3 (renames touch the smaller, honest model).
- **US5 (Phase 6)**: Depends on US4 (error sites are in final-named modules).
- **US6 (Phase 7)**: Depends on US5 (docs describe the final structure).
- **Polish (Phase 8)**: Depends on all stories.

### User Story Dependencies

The stories are strictly sequential by design (research R10): each lands on the previous one's output, and every story ends gate-green so partial completion is still a viable codebase. Unlike feature specs, these stories are NOT parallelizable across a team — they touch the same files in sequence.

### Within Each User Story

- State/model moves before handler moves before draw moves (US1).
- Dispatcher before deduplication (US2): the dispatcher reveals the call sites.
- Field removal before dependency removal (US3): the dependency dies with its last user.
- Rename before visibility tightening (US4): the rename churns references; visibility is a separate, mechanical pass.
- Status mechanism before call-site conversion before entry-point cleanup (US5).
- Docs last (US6).

### Parallel Opportunities

Limited by design — the refactor is a sequence of behavior-preserving steps over the same files. Within stories, only these are safely parallel:

- T003–T007 (US1 moves) touch disjoint target files once the skeleton (T002) exists — but each requires the source `src/ui/app.rs`, so run them sequentially in practice, one commit per move.
- T018/T019 (US3 field removals) touch different model files — parallelizable.
- T033/T034 (US6 doc updates) touch different files — parallelizable.

---

## Parallel Example: User Story 3

```bash
# Different model files, no shared state:
Task: "Remove Item.id and Item.base_quantity in src/models/item.rs"
Task: "Remove PackingList.id/name/file_path in src/models/packing_list.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (baseline + skeleton).
2. Complete Phase 2: US1 — the monolith is split.
3. **STOP and VALIDATE**: gate green, 39 tests unmodified, smoke pass identical.
4. The codebase is already navigable — US1 alone delivers the core review value.

### Incremental Delivery

1. Setup → US1 (split) → validate.
2. US2 (dispatcher + dedup) → validate.
3. US3 (dead code) → validate with format round-trip.
4. US4 (renames) → validate.
5. US5 (errors) → validate with forced-failure test.
6. US6 (docs) → final quickstart run.
7. Polish → commit.

Each story ends with the gate green and all tests passing, so stopping after any story leaves a better codebase than before it.

---

## Notes

- [P] tasks = different files, no dependencies (rare here — see Parallel Opportunities).
- [Story] labels map tasks to spec.md user stories for traceability.
- The 39 existing tests are the behavioral contract (FR-011): assertions never change; only `use` paths do.
- The help/hint contract tests (spec 016) guard the binding set through every step — do not weaken them.
- Commit after each task or logical group; Conventional Commits (`refactor:` for moves/splits, `chore:` for dependency removal, `docs:` for US6).
- Avoid: drive-by redesign beyond the spec's findings, weakening tests to make them pass, hand-editing `Cargo.lock`.
