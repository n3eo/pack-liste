---
description: "Task list for 012-trip-quantities-persisted"
---

# Tasks: Persist Quantities in the Trip File

**Input**: Design documents from `/specs/012-trip-quantities-persisted/`

**Prerequisites**: plan.md (required), spec.md (required for user stories)

**Tests**: Included per story (the spec's acceptance scenarios are testable).

**Organization**: Sequenced after 010, before 013 (see plan).

## Format: `[ID] [P?] [Story] Description`

---

## Phase 1: Storage (US1)

**Goal**: Every selected item's quantity is saved to the trip file.

**Independent Test**: Compile a list, save, read the trip file — each item carries its quantity (US1/AC1-3).

- [X] T001 [US1] Add `quantity: Option<u32>` to `TripItem` in `src/config/trip_file.rs` (serde default None, `skip_serializing_if` when None — pre-feature files load, EC-001)
- [X] T002 [US1] Write the quantity at save time: `save_trip_file` in `src/ui/input.rs` sets `quantity: Some(item.quantity)` for every selected item (fixed items included, US1/AC3); no calculation happens at save (plan D4)

**Checkpoint**: The file is a complete snapshot.

---

## Phase 2: Restore (US2, US3)

**Goal**: A resumed trip shows the saved quantities; hand edits are honored.

**Independent Test**: Save, change a quantity-relevant config value, resume — the saved numbers are shown (US2/AC1); hand-edit a quantity in the file, resume — the edited value appears and survives the next save (US3).

- [X] T003 [US2] Restore on load: `apply_trip_file_path` and the `App::new` resume path in `src/ui/input.rs` write the trip item's quantity into the matching catalog item and skip the recalculation on resume (plan D1 — the current `update_quantities()` call after restore is removed from the resume path; only a fresh list calculates on load)
- [X] T004 [US2] Carry the quantity through `TripFile::reconcile` in `src/config/trip_file.rs`: items that survive reconciliation keep their quantity (EC-002)
- [X] T005 [US3] Add tests in `src/ui/tests/config_io.rs`: round-trip with quantities; resume-restores-saved-quantities (a modifier changed between save and resume — the saved numbers win, US2/AC1); hand-edited quantity loads and survives the next save (US3/AC1-2); a pre-feature trip file (no quantity keys) loads and calculates as before (EC-001)

**Checkpoint**: The file is authoritative for its own trip.

---

## Phase 3: Separation and Docs (US4)

**Goal**: The input config's quantity behavior is untouched; the format is documented.

**Independent Test**: The existing config round-trip tests pass unmodified (US4); the docs describe the quantity key.

- [X] T004b [US4] Verify the input config is unchanged: the existing config tests pass without modification; no calculated quantity is written to `config.yaml` (US4/AC1-2)
- [X] T005b [US4] Update `docs/quantity-calculation.md`'s storage section: the trip file carries quantities; resume restores them; the input config keeps fixed quantities only
- [X] T006 Final gate: `cargo check`, `cargo clippy`, `cargo clippy --tests`, `cargo test`, `cargo fmt --check` all exit 0; the quickstart's save/resume/hand-edit checks pass

---

## Dependencies & Execution Order

- T001 → T002 → T003 → T004 → T005 (storage before restore before tests).
- **Cross-spec**: after 010 (the quantities saved are the per-item-calculated ones); before 013 (the optional-on-load field is compat machinery 013 removes); satisfies the amended 018's FR-008 resume-restore requirement.

## Notes

- The 42 existing tests are the contract; the config tests passing unmodified is the US4 proof.
- Under the amended 018, parameter changes no longer auto-recalculate — the "recalculate on parameter change" in US2/AC2 becomes "the user presses a recalculate key".
