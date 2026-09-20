---
description: "Task list for 010-item-quantity-config"
---

# Tasks: Per-Item Quantity Configuration

**Input**: Design documents from `/specs/010-item-quantity-config/`

**Prerequisites**: plan.md (required), spec.md (required for user stories)

**Tests**: The spec's SC-001..SC-007 demand test coverage; tests are included per story.

**Organization**: Tasks grouped by user story, in dependency order. Sequenced before 012 and 013 (see plan).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: parallelizable (different files, no dependencies)
- **[Story]**: US1–US4 per spec.md

---

## Phase 1: Data Model and Calculation (US1)

**Goal**: The per-item rate and buffer exist on the model and in the calculation, with defaults reproducing current behavior exactly.

**Independent Test**: `cargo test` — the existing calculation tests pass unchanged (defaults identical, SC-005) plus the new per-item rule tests.

- [X] T001 [US1] Add `QuantityBuffer { None, Reserve(u32), Multiplier(f32) }` (derive Debug, Clone, Copy, PartialEq, Serialize, Deserialize; None default) and `rate: f32` (default 1.0) to `Item` in `src/models/item.rs`; `Item::new` sets rate 1.0 / buffer None
- [X] T002 [US1] Change `ItemQuantitySpec::Auto` to carry `{ rate: f32, buffer: QuantityBuffer }` in `src/quantity/model.rs`; `Fixed(u32)` unchanged
- [X] T003 [US1] Implement the per-item rule in `src/quantity/calculation.rs`: rate amount = `round(min(weekdays,20) × style × weather × rate)`, clamped to minimum 1 (EC-002); then `+ reserve` or `× multiplier` (EC-006: trip multipliers touch only the rate-based part); final clamp 1–99 (EC-003, FR-007)
- [X] T004 [US1] Add calculation tests in `src/quantity/calculation.rs`: rate 1.0/None equals the old rule for identical inputs (SC-002/SC-005); underwear 5 days rate 1.0 + reserve 2 → 7; trousers rate 0.15 → 1 (5 days) and → 2 (14 days); socks 5 days × 1.25 → per the rule; reserve invariant under style/weather changes (SC-006); clamp cases (rate 10 × 30 days → 99; tiny rate → 1); fixed items unchanged

**Checkpoint**: The calculation is per-item; defaults are byte-identical to before.

---

## Phase 2: Persistence (US1, US4)

**Goal**: The modifiers persist with the item and pre-feature files load with defaults.

**Independent Test**: Round-trip tests pass; a pre-feature config loads and calculates identically.

- [X] T005 [US1] Extend `SavedItem` in `src/ui/config_io.rs` with `rate: Option<f32>` and the buffer fields (`reserve: Option<u32>`, `buffer_multiplier: Option<f32>`), serde-defaulted for pre-feature files (FR-008); save writes exactly one buffer kind (SC-007); load maps to `Item`'s rate/buffer
- [X] T006 [US1] Add round-trip tests in `src/ui/tests/config_io.rs`: rate+reserve round-trips; rate+multiplier round-trips; a file with neither loads as 1.0/None; no persisted item ever carries both buffer kinds (SC-007)

**Checkpoint**: Modifiers survive save/load; old files load.

---

## Phase 3: Dialog Editing (US2)

**Goal**: The add/edit dialog edits the rate and buffer with exclusivity enforcement.

**Independent Test**: Edit an item's rate/buffer, save, verify the quantity and the persisted values; set a reserve then a multiplier — the reserve clears (US2/AC4).

- [X] T007 [US2] Extend `InputMode::AddingItem`/`EditingItem` in `src/ui/state.rs` with `rate: String` and `buffer` dialog state (kind + value); `apply_item_edit` writes them to the item; setting one buffer kind clears the other (FR-011)
- [X] T008 [US2] Add the Rate and Buffer fields to the shared `draw_item_dialog` in `src/ui/draw.rs` (after Mode; the dialog's existing field-rendering pattern); extend `InputField` with Rate/Buffer and the field cycle (`next`/`prev`) in `src/ui/state.rs`
- [X] T009 [US2] Wire the dialog handler in `src/ui/handlers/welcome.rs` + `dialogs.rs`: typing/Backspace edit the focused field; Left/Right step the value (rate ±0.05, reserve ±1, multiplier ±0.05); Space on Buffer cycles None → Reserve → Multiplier; validation rejects rate outside 0.05–10.0, reserve outside 0–10, multiplier outside 1.0–3.0 (FR-006, keeping the previous valid value)
- [X] T010 [US2] Update hints (`src/ui/hints.rs` AddEditPopup), help (`src/ui/help.rs`), and the extraction tests with the new dialog bindings; entries must fit the minimum width (hint tests)
- [X] T011 [US2] Add state tests in `src/ui/tests/`: exclusivity (set reserve → set multiplier → reserve cleared, and the reverse); validation bounds; the dialog's rate/buffer reach the item on save

**Checkpoint**: The user can configure per-item modifiers from the dialog.

---

## Phase 4: Fixed Items and Documentation (US3)

**Goal**: Fixed items ignore the modifiers; the rule is documented.

**Independent Test**: A fixed item's quantity never changes; `docs/quantity-calculation.md` describes the per-item rule.

- [X] T012 [US3] Verify (and test, if not already covered) that fixed items never recalculate: the calculation passes them as `Fixed` regardless of their rate/buffer (FR-005); switching fixed→automatic reactivates the modifiers (EC-005)
- [X] T013 [US3] Update `docs/quantity-calculation.md` with the per-item rule (rate, buffer kinds, exclusivity, clamps, defaults) replacing the global-multiplier-only description
- [X] T014 Final gate: `cargo check`, `cargo clippy`, `cargo clippy --tests`, `cargo test`, `cargo fmt --check` all exit 0 with zero warnings; the quickstart's example-based checks pass

---

## Dependencies & Execution Order

- T001–T004 (model + calculation) before T005–T006 (persistence) before T007–T011 (dialog) — the dialog edits what the model stores.
- T012–T013 parallelizable after T004.
- **Cross-spec**: land before 012 (trip quantities) and 013 (remove compat) — see the plan's sequencing note. Under the amended 018 model, no recalculation follows a modifier edit; the user presses a recalculate key (the plan's R7).

## Notes

- The 42 existing tests are the behavioral contract: the default-modifier equivalence (T004) is what keeps them passing unchanged.
- The buffer enum makes exclusivity a type guarantee (SC-007) — no runtime both-check needed beyond the dialog's kind-switching.
