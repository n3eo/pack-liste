---
description: "Task list for 013-no-compat-code"
---

# Tasks: Remove Version-Compatibility Code

**Input**: Design documents from `/specs/013-no-compat-code/`

**Prerequisites**: plan.md (required), spec.md (required for user stories)

**Tests**: Included — the strict-load contract and the rejected-file safety need tests.

**Organization**: Sequenced AFTER 010 and 012 (see plan — this spec re-baselines the formats).

## Format: `[ID] [P?] [Story] Description`

---

## Phase 1: Strict Formats (US1)

**Goal**: Both data files load only the current format; every field required; unknown fields rejected.

**Independent Test**: A current-format file loads; a file missing a field or carrying an unknown field fails with a clear error naming the file (US1/AC2).

- [X] T001 [US1] Make `AppConfig` strict in `src/ui/config_io.rs`: remove all five `#[serde(default...)]` attributes (plan D1), add `#[serde(deny_unknown_fields)]` (EC-002)
- [X] T002 [US1] Remove `SavedItem`'s hand-written `Deserialize` impl (the legacy single-`category` lifting, plan D1) in `src/ui/config_io.rs`; the derive handles the strict format (`categories` required; `quantity` stays optional-in-format — automatic items legitimately omit it)
- [X] T003 [US1] Make `TripFile`/`TripItem` strict in `src/config/trip_file.rs`: remove the `#[serde(default)]` attributes; add `deny_unknown_fields`; post-012, `quantity` is required on `TripItem`
- [X] T004 [US1] Verify the load failure surfaces: a config failure at startup exits with the error (EC-003); a trip file passed as the argument that fails to load exits with the error rather than silently starting a new trip that would overwrite the file on exit (US2/AC3) — check `App::new` and `apply_trip_file_path` in `src/ui/input.rs`

**Checkpoint**: One format, one code path.

---

## Phase 2: Test Re-baseline (US1, EC-004)

**Goal**: The compat tests are gone; strict-load tests exist.

**Independent Test**: No test constructs a file format the current program cannot produce (EC-004); the strict contract is tested.

- [X] T005 [US1] Remove the compat tests in `src/ui/tests/config_io.rs`: `old_config_without_trip_params_loads`, `legacy_files_with_activity_fields_load_and_save_clean`, and any legacy-category-lifting test paths (EC-004)
- [X] T006 [US1] Add strict-load tests in `src/ui/tests/config_io.rs`: a current-format config and trip file load; a config missing a required field fails; a file with an unknown field fails; the error names the file
- [X] T007 [US2] Add the rejected-file safety test in `src/ui/tests/config_io.rs`: an unloadable file is byte-identical after the failed load attempt (US2/AC2) — the load path in `src/ui/input.rs` never writes

**Checkpoint**: The tests pin the current format only.

---

## Phase 3: Docs and Spec Supersession (US3, US4)

**Goal**: The documentation states the current format only; the affected specs record the reversal.

**Independent Test**: The docs describe exactly the current format (US3/AC2); no active spec demands old-file loading (US4/AC1).

- [X] T008 [US3] Update the format documentation (`docs/quantity-calculation.md` storage section and any format description): the current format only, no version history, with the regenerate-or-hand-edit path for old files (US3)
- [X] T009 [US4] Add supersession notes to `specs/004-packing-list-io/spec.md`, `specs/010-item-quantity-config/spec.md` (FR-008), and `specs/012-trip-quantities-persisted/spec.md` (EC-001) — each notes "Superseded by spec 013 for format compatibility; the program loads only the current format"
- [X] T010 Final gate: `cargo check`, `cargo clippy`, `cargo clippy --tests`, `cargo test`, `cargo fmt --check` all exit 0; the quickstart's strict-load and rejected-file checks pass; first-run creation still works (EC-001)

---

## Dependencies & Execution Order

- T001–T003 parallelizable (different structs); T004 after them; T005–T007 after T001–T003; T008–T009 parallelizable.
- **Cross-spec (critical)**: lands AFTER 010 and 012 — both add fields whose pre-feature loading relies on the defaults removed here. After this spec, the formats are final.

## Notes

- First-run creation (no config file exists → create one) is NOT compatibility and stays (EC-001).
- The 42 existing tests minus the removed compat tests remain the contract; the strict tests replace them.
