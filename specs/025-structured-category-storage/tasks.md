---
description: "Task list for 025-structured-category-storage (revision 3 — labels, singletons, chips)"
---

# Tasks: Label-Based Item Storage

**Input**: Design documents from `/specs/025-structured-category-storage/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Included per story — the schema, dedup, singleton, and chip contracts are testable (Constitution III); the behavior contract (collapse, scroll, sticky) keeps its tests plus the PTY pass.

**Organization**: Tasks are grouped by user story. Phase 1 is ATOMIC (the model change breaks compilation until every consumer is updated); US1's schema tests and US2's dedup follow; US3's flat UI and US5's singleton/chips build on the model; US4's cutover lands last.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1–US5)
- Include exact file paths in descriptions

## Path Conventions

Single project: `src/` at repository root. Model in `src/models/item.rs`; schemas in `src/ui/config_io.rs` + `src/config/trip_file.rs`; state in `src/ui/state.rs`; rendering in `src/ui/screens/main.rs`; dialogs in `src/ui/handlers/dialogs.rs`; tests in `src/ui/tests/`.

---

## Phase 1: Foundational — The Label Model (ATOMIC)

**Purpose**: `Item` carries `labels: Vec<String>` + `singleton: bool`; both schemas mirror them; every consumer compiles. This phase lands as ONE unit — the type change breaks the build until all consumers are updated.

**⚠️ CRITICAL**: No user story verification can happen until this phase is complete and green.

- [X] T001 [US1] Change `Item` in `src/models/item.rs`: replace `categories: Vec<String>` with `labels: Vec<String>` (one or more, order preserved) + `singleton: bool` (default false); `Item::new(name, labels, quantity)`; delete `primary_category`, `in_category`, and `category_layers` (research R1)
- [X] T002 [US1] Change the configuration schema in `src/ui/config_io.rs`: `SavedItem` gains `labels: Vec<String>` (required, non-empty) + `singleton: bool` (`#[serde(default)]`, `skip_serializing_if` when false) and loses `categories`; `AppConfig` loses its top-level `categories` (FR-002); `deny_unknown_fields` stays (FR-004)
- [X] T003 [US1] Change the trip schema in `src/config/trip_file.rs`: `TripItem` gains `labels: Vec<String>` + `singleton: bool` (same optionality) and loses `category` (FR-003); `reconcile` matches by name and updates labels + singleton from the config item (research R8)
- [X] T004 [US1] Update the save/load paths in `src/ui/input.rs`: `save_config`/`save_trip_file` write the fields; the load builds `Item`s from them; `get_all_categories` becomes `all_labels` (the distinct labels, the picker's data source)
- [X] T005 [US3] Update the row model in `src/ui/state.rs`: `PanelRow` becomes `Label(String)` + `Item(usize)` (research R2); `left_rows`/`right_rows` build one header per distinct label (sorted) with the carrying items beneath — an item appears under EVERY label it carries (FR-007); `collapsed_labels: Vec<String>` replaces `collapsed_categories` + `collapsed_groups`; group-selection and group-recalc resolve by `labels.contains` (FR-008); delete every `category_layers` call
- [X] T006 [US3] Update the renderers in `src/ui/screens/main.rs`: the flat label grouping (one header level; the Group/Category arms die); the packing indicator on a label header computed over the items carrying it; the existing scroll/sticky/highlight machinery unchanged
- [X] T007 [US5] Add the occurrence state in `src/ui/state.rs`: `occurrence_overrides: BTreeMap<(usize, String), bool>` (research R3); the effective occurrence state = the override if present, else the item's own; `toggle_item_selection`/`toggle_item_packed` resolve by cursor row: a singleton flips the item's own state and clears its overrides (FR-011); a non-singleton writes the override (FR-012); initialize the map in `src/ui/input.rs` + `src/ui/tests/mod.rs`
- [X] T008 [US5] Add the chip rendering in `src/ui/screens/main.rs`: a shared helper builds the chip spans for an item row — per label, ` [Label]` between the name and the quantity, `Color::DarkGray` when the occurrence's effective state matches the item's own, `Color::Gray` when it differs (FR-013, research R5); the row truncation budget: the name truncates first, the chips follow, the quantity never leaves the row (SC-009); both panels use the helper
- [X] T009 [US1] Update the add/edit dialog in `src/ui/handlers/dialogs.rs` + `src/ui/state.rs`: the category field becomes a labels multi-select (the picker lists `all_labels`, Left/Right cycles, Space toggles membership, free text adds a new label; at least one required to save — FR-009); a Singleton field (Space toggles) joins the cycle; the save assigns `labels` + `singleton`
- [X] T010 [US1] Update the tests for the model change in `src/ui/tests/`: the `test_item` helper takes (name, labels); the category-era tests (two-layer tree, group/sub-category collapse, `category_split_*`, `multi_category_*`) are deleted with the model (research R10); the behavior tests (collapse, selection, recalc, sticky, scroll) construct label items, same assertions

**Checkpoint**: The gate is green; the app runs on the label model; the flat grouping renders (the updated state tests verify the row model).

---

## Phase 2: User Story 1 — The label format round-trips (Priority: P1) 🎯 MVP

**Goal**: A labeled config loads, renders, and saves back the same structure (SC-001, SC-006).

**Independent Test**: `cargo test config_io` passes; the quickstart §5 round trip preserves every value.

### Implementation for User Story 1

- [X] T011 [US1] Rewrite the schema tests in `src/ui/tests/config_io.rs`: a labeled config loads (labels, optional singleton); an item without a singleton serializes WITHOUT the key; a load→save round trip preserves every label/singleton verbatim; an old-format file (with `categories:`) is rejected with an unknown-field error naming it (FR-004)
- [X] T012 [US1] Add the no-separator test in `src/ui/tests/config_io.rs`: a saved config's serialized yaml contains no ": " inside any label value and no top-level `categories` key (SC-001)

**Checkpoint**: US1 done — the label format is the single accepted form, round-tripping verbatim.

---

## Phase 3: User Story 2 — The deduplicated, individual item list (Priority: P1)

**Goal**: The list is deduplicated on load; every item is one entity across its labels (SC-002, SC-003).

**Independent Test**: `cargo test config_io` + `cargo test state` pass; the quickstart §1/§3 dedup and individual-treatment checks hold.

### Implementation for User Story 2

- [X] T013 [US2] Implement the dedup on load in `src/ui/config_io.rs`: the key is (name, labels) compared as the ordered list exactly as written; the first occurrence wins; later exact matches drop (FR-005, research R6)
- [X] T014 [US2] Add the dedup tests in `src/ui/tests/config_io.rs`: two identical entries load as one; same name with different labels remains two items; the real-data shape (a "Ladekabel"-style 5× duplicate) merges to one
- [X] T015 [US2] Add the individual-treatment tests in `src/ui/tests/state.rs`: a multi-label item appears under each label group in `left_rows`/`right_rows` but is one item — editing its quantity or name changes the one item; the compiled list counts it once (FR-006, SC-003)

**Checkpoint**: US2 done — the list is the deduplicated truth; items are individuals.

---

## Phase 4: User Story 3 — The flat label UI (Priority: P1)

**Goal**: The panels group by label with collapse/select/recalc per label and the full scroll contract (SC-004, SC-005).

**Independent Test**: The quickstart §3/§4 pass — the flat grouping, the per-label actions, and scrolling at both extremes.

### Implementation for User Story 4

- [X] T016 [US3] Add the flat-grouping tests in `src/ui/tests/state.rs`: one `Label` header per distinct label (sorted), the carrying items beneath; an item with labels [A, B] appears under both; a collapsed label hides its items with the cursor clamped to the header (FR-007, FR-008)
- [X] T017 [US3] Add the per-label action tests in `src/ui/tests/state.rs`: group-selection flips every item carrying the label; group-recalc covers exactly those items; the scope resolves from a `Label` header row (FR-008)
- [X] T018 [US3] PTY-verify the flat UI: run the app on a migrated config — one header per label, multi-label items under each of their labels, `C` collapses a label, `Ctrl+Space` selects a label's items, `R`/`Ctrl+R` recalculate a label's items (quickstart §3)
- [X] T019 [US3] PTY-verify scrolling at both extremes in both panels (FR-008): fully down (the last item visible, highlighted) and fully up (the first label header visible, highlighted), with a label collapsed; the sticky header never displaces the cursor (quickstart §4)

**Checkpoint**: US3 done — the flat UI is complete and fully navigable.

---

## Phase 5: User Story 5 — Singletons and chips (Priority: P1)

**Goal**: Singleton toggles synchronize; non-singletons toggle per occurrence; the chips show match/differ (SC-008, SC-009).

**Independent Test**: `cargo test state` passes; the quickstart §3 singleton/chip checks hold.

### Implementation for User Story 5

- [X] T020 [US5] Add the singleton toggle tests in `src/ui/tests/state.rs`: a singleton with labels [A, B] — toggling under A flips the item's state and both occurrences follow (FR-011); a non-singleton — toggling under A writes the override, the B occurrence keeps the item's state (FR-012); the persisted save stores the item's own state only
- [X] T021 [US5] Add the chip tests in `src/ui/tests/state.rs` (or the rendering helper's unit home): the chip model — dark when the effective occurrence state matches the item's own, light when it differs; a singleton's chips are uniformly dark (FR-013, SC-009)
- [X] T022 [US5] PTY-verify the singletons and chips: a migrated singleton (e.g. an "Ausland: Karten" item) — toggle under one label, both occurrences flip, the chips stay uniformly dark grey; a non-singleton — toggle under one label, only that occurrence changes, the other label's chip turns light grey; the quantity never leaves the row (quickstart §3)
- [X] T023 [US5] Verify the dialog: assign multiple labels (Space toggles, free text adds), toggle Singleton, save — the item carries the labels and the flag; at least one label is required (FR-009)

**Checkpoint**: US5 done — the singleton semantics and the chips are verified end to end.

---

## Phase 6: User Story 4 — The old format is gone, cleanly (Priority: P2)

**Goal**: The cutover is complete: one format, no compat code, the real config migrated (SC-006).

**Independent Test**: The quickstart §0/§2 — the real config is labeled; an old-format file is rejected with a clear error.

### Implementation for User Story 4

- [X] T024 [US4] Migrate the real configuration (one-time, external — research R6): back up `~/.config/pack-liste/config.yaml`, convert it (each "X: Y" → labels [X, Y] + `singleton: true`; "X" → labels [X]; drop the top-level `categories`; merge the exact duplicates, first wins, with a report of every merge), and convert any existing trip files the same way
- [X] T025 [US4] Verify the rejection: place an old-format file (with `categories:`) at the config path; run the app — it exits with the unknown-field error naming `categories`, the file untouched (quickstart §2)
- [X] T026 [US4] Verify no compat code remains: `grep -rn "category_layers\|primary_category\|in_category\|selected_categories\|collapsed_categories\|collapsed_groups" src/` returns nothing; `grep -rn '": "' src/` finds no separator splitting in any load/save path (SC-005's label analog)

**Checkpoint**: US4 done — the label format is the only format, in the code and in the data.

---

## Phase 7: Polish & Cross-Cutting Concerns

- [X] T027 Update `TUI_USAGE.md`: the label yaml examples (`labels` + optional `singleton`), the flat label grouping, the singleton semantics, the chips, the dialog's label assignment
- [X] T028 Run the quickstart validation end to end ([quickstart.md](quickstart.md) §0–§6): the migration, the schema/dedup tests, the rejection, the flat-UI/singleton/chip pass, the scroll extremes, the round trip, the full regression
- [X] T029 Final gate: `cargo check`, `cargo clippy`, `cargo clippy --tests`, `cargo test`, `cargo fmt --check` all exit 0 with zero warnings

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (the model)**: ATOMIC — T001–T010 land together; blocks everything.
- **Phase 2 (US1)**: depends on Phase 1 (the schema tests need the schemas).
- **Phase 3 (US2)**: depends on Phase 2 (the dedup tests the schema); T015 depends on the row model (Phase 1's T005).
- **Phase 4 (US3)**: depends on Phase 1; the PTY checks (T018/T019) need the migrated data (Phase 6's T024) — run them against a converted copy if T024 has not landed.
- **Phase 5 (US5)**: depends on Phase 1 (T007/T008) and Phase 3 (the individual-item semantics).
- **Phase 6 (US4)**: depends on Phases 2–5 (the app must work before the real data is cut over).
- **Phase 7**: depends on all stories.

### Within Each User Story

- The model (T001) before the schemas (T002/T003) before the paths (T004–T009) — in one landing.
- The tests with the implementation they pin (T010 with the model; T011/T012 after the schemas; T014/T015 after the dedup/row model).

### Parallel Opportunities

- T002 and T003 (the two schemas) are parallel-safe (different files).
- T011/T012 and T013/T014 are parallel-safe within their phases.
- T016/T017 (the state tests) are parallel-safe with each other.
- T027 (docs) is parallel-safe with T024–T026.

---

## Implementation Strategy

### MVP First (Phase 1 + 2)

1. Land Phase 1 atomically — the label model, green gate.
2. Land Phase 2 — the schema tests pin the format.
3. **STOP and VALIDATE**: the app runs on the label model; the round trip holds.
4. The storage change (the essence) is delivered; the dedup, the flat UI, the singletons, and the cutover follow.

### Incremental Delivery

1. Phase 1 → the model and schemas are labeled.
2. Phase 2 (US1) → the format round-trips, pinned by tests.
3. Phase 3 (US2) → the list is deduplicated; items are individuals.
4. Phase 4 (US3) → the flat UI verified end to end.
5. Phase 5 (US5) → the singletons and chips verified.
6. Phase 6 (US4) → the real data migrated; the old format rejected.
7. Phase 7 → docs + quickstart + gate.

---

## Notes

- The existing behavior tests are the contract: the collapse/selection/recalc/sticky/scroll tests must keep passing with only their constructors updated (T010) — never weakened.
- The category-era tests are DELETED with the model (research R10) — dead tests for dead behavior violate Principle I.
- The migration (T024) is external and one-time: no migration code ships in the app (FR-010, research R6).
- `deny_unknown_fields` is the rejection mechanism (FR-004): serde's unknown-field error names the field; no hand-rolled validation is added.
- The occurrence-override map (T007) is session state — never persisted; the load normalizes a disagreeing singleton to the item's own state (FR-014).
- Commit per phase; Conventional Commits (`refactor:` for the model/schema work, `feat:` for the singletons/chips, `test:` where a task is purely tests, `docs:` for TUI_USAGE.md).
