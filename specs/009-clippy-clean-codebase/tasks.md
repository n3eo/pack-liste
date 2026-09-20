---

description: "Task list for Warning-Free Codebase via Static Analysis feature implementation"

---

# Tasks: Warning-Free Codebase via Static Analysis

**Input**: Design documents from `/specs/009-clippy-clean-codebase/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/verification.md, quickstart.md

**Tests**: No new unit tests are requested. The feature is behavior-preserving; the regression net is the existing 11-test suite plus the verification scenarios in quickstart.md (the quality gate itself is this feature's "test").

**Organization**: Tasks are grouped by user story. US1 carries the implementation weight (reaching zero warnings); US2 verifies the fixes are genuine; US3 installs the guard. The constitution (v1.0.0, Principles I–III) governs this work.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete tasks)
- **[Story]**: Which user story this task belongs to (e.g. US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/` at repository root

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Baseline evidence before any change

- [X] T001 Capture the baseline warning inventory: run `cargo check`, `cargo clippy`, and `cargo clippy --tests`; save full outputs to `specs/009-clippy-clean-codebase/baseline-check.txt` and `specs/009-clippy-clean-codebase/baseline-clippy.txt` (expected: 12 and 37 warnings per research.md; this is the "before" evidence for SC-003)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: None for this feature

This feature has no blocking infrastructure. Note the opposite ordering constraint: the lint gate (US3) MUST be installed only AFTER the cleanup (US1/US2) completes — denying warnings before the fixes exist would fail every build. Proceed directly to User Story 1.

---

## Phase 3: User Story 1 - Clean static analysis with no warnings (Priority: P1) 🎯 MVP

**Goal**: `cargo check`, `cargo clippy`, and `cargo clippy --tests` all complete with zero warnings on the project's own source.

**Independent Test**: Run the three commands; all exit 0 with no warnings (quickstart.md Scenario 1).

### Implementation for User Story 1

*Dead code first — removing doomed files before fixing lints inside them avoids wasted work (research.md: removal decision).*

- [X] T002 [US1] Remove the dead services module: delete `src/services/packing_service.rs`, `src/services/item_service.rs`, `src/services/mod.rs`, and the `mod services;` declaration in `src/main.rs` (dead: `PackingService`, `ItemService` never constructed)
- [X] T003 [P] [US1] Remove the dead config model: delete `src/config/model.rs` (`PackingListConfig` never constructed) and the `pub mod model;` line in `src/config/mod.rs`
- [X] T004 [P] [US1] Remove the dead error enums: delete `src/config/error.rs` (`ConfigError`), `src/quantity/error.rs` (`QuantityError`), `src/ui/error.rs` (`UiError`), and their `pub mod error;` lines in `src/config/mod.rs`, `src/quantity/mod.rs`, `src/ui/mod.rs`
- [X] T005 [P] [US1] Remove the dead model methods: `Item::toggle_packed` and `Item::set_quantity` in `src/models/item.rs`; `PackingList::{remove_item, get_item_mut, total_items, packed_items, unpacked_items}` in `src/models/packing_list.rs` (keep `new` and `add_item`)
- [X] T006 [US1] Remove now-unused dependencies from `Cargo.toml`: `serde_json` (expected — `PackingService` was the only user); check whether `chrono` still has a user after T005 (`PackingList::new` timestamp id) and remove it if not; verify with `cargo build`; never hand-edit `Cargo.lock` (repo guardrail)

*Mechanical/idiomatic fixes (all locations listed in research.md).*

- [X] T007 [US1] Fix the 17 `collapsible_if` warnings by collapsing to `if a && b`: 16 in `src/ui/app.rs` (lines ~866, 867, 906, 937, 1181, 1203, 1212, 1231, 1239, 1271, 1279, 1280, 1325, 1362, 1808, 2082) and 1 in `src/config/trip_file.rs` (~line 50)
- [X] T008 [US1] Fix the 3 `manual_clamp` warnings by using the clamp idiom clippy suggests at `src/ui/app.rs` (~lines 3174, 3328, 3482 — the popup width/height computations)
- [X] T009 [US1] Fix the `for_kv_map` warning in `right_item_order` (`src/ui/app.rs` ~line 429): iterate `categories_map.values()` instead of `&categories_map`
- [X] T010 [US1] Fix the `while_let_loop` warning in `App::run` (`src/ui/app.rs` ~line 799): rewrite the `loop { match self.state {...} }` as a `while let` on `AppState::Running`, keeping the post-loop `save_config()`/`save_trip_file()` calls intact
- [X] T011 [US1] Fix the `unused_mut` warning at `src/ui/app.rs` ~line 484 (drop `mut`) and the unnecessary-parentheses warning at `src/ui/app.rs` ~line 3165
- [X] T012 [US1] Fix the 2 `too_many_arguments` warnings structurally: introduce a `PopupFields` struct in `src/ui/app.rs` (fields per data-model.md: `field: InputField`, `name: String`, `category: String`, `quantity: String`, `category_list: Vec<String>`, `category_index: usize`); refactor `draw_add_item_popup` and `draw_edit_item_popup` to take `(f, size, &PopupFields)` and update their call sites in `App::draw`; do NOT use `#[allow]` (FR-005)
- [X] T013 [US1] Re-run fix waves until clean: after T002–T012, run `cargo check && cargo clippy && cargo clippy --tests`; fix any cascading warnings (e.g., newly unused imports exposed by removals) and repeat until all three commands emit zero warnings (research.md: iterative-wave decision)

**Checkpoint**: `cargo check`, `cargo clippy`, `cargo clippy --tests` all pass with zero warnings; `cargo test` still passes (11 tests)

---

## Phase 4: User Story 2 - Fixes improve the code, not silence the tools (Priority: P2)

**Goal**: The warning-free state was reached by genuine fixes: dead code removed (not annotated), idiomatic rewrites applied, zero unjustified suppressions, behavior unchanged.

**Independent Test**: The grep checks return clean, the test suite passes identically, and the diff review confirms removal-not-annotation (quickstart.md Scenarios 2 and 5).

### Implementation for User Story 2

- [X] T014 [US2] Verify zero suppressions: `grep -rn "#\[allow(" src/` returns no matches (target: none at all; any exception must be scoped + carry a justifying comment per FR-005 and constitution Principle I)
- [X] T015 [US2] Verify the dead code is gone, not annotated: `grep -rn "PackingService\|ItemService\|PackingListConfig\|UiError\|QuantityError\|ConfigError" src/` returns no matches
- [X] T016 [US2] Verify behavior is unchanged: `cargo test` passes with the same 11 tests; `cargo build` succeeds; `./target/debug/pack-liste --help` output is identical to before the cleanup (same flags, same version)
- [X] T017 [US2] Review the full diff for fix quality: collapsible ifs are collapsed (not commented out), clamps use the clamp idiom, `PopupFields` is used at every popup call site, the `while let` rewrite preserved the exit-path saves; no drive-by redesign beyond what the warnings required (FR-006)

**Checkpoint**: The warning-free state is achieved by genuine fixes with behavior preserved

---

## Phase 5: User Story 3 - The clean state is guarded going forward (Priority: P3)

**Goal**: A structural lint gate fails on any newly introduced warning, and the verification commands are documented.

**Independent Test**: Introduce a deliberate warning, the gate fails; remove it, the gate passes (quickstart.md Scenario 3).

### Implementation for User Story 3

- [X] T018 [US3] Add the `[lints]` table to `Cargo.toml` denying warnings by default (e.g., `unsafe_code = "deny"`, `unused = "deny"`, clippy `all` denied — finalize the exact keys against Rust 1.98 support per contracts/verification.md; the contract requirement is the outcome: plain `cargo clippy` fails on warnings)
- [X] T019 [US3] Prove the gate bidirectionally (SC-005): append `fn _unused_probe() {}` to `src/main.rs`, run `cargo clippy` and confirm it FAILS; remove the probe, run again and confirm exit 0 with zero warnings
- [X] T020 [P] [US3] Document the gate in `AGENTS.md`: state the verification commands (`cargo check`, `cargo clippy`, `cargo clippy --tests`, `cargo test`, `cargo fmt --check`) and the warning-free requirement (FR-008)

**Checkpoint**: The gate is structural and proven; the docs state the requirement

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final validation across everything

- [X] T021 Run `cargo fmt` and confirm `cargo fmt --check` exits 0
- [X] T022 Run all six quickstart.md validation scenarios and record the results (Scenario 6 is a manual TUI smoke test: start with `--force-tui`, quit with q, confirm config/trip save)
- [X] T023 Final gate run and baseline comparison: `cargo check`, `cargo clippy`, `cargo clippy --tests`, `cargo test` all clean; diff the outputs against the T001 baseline and confirm 100% of the original warnings are resolved (SC-003)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Empty by design (see note); the lint gate deliberately comes last
- **US1 (Phase 3)**: Depends on Phase 1 (baseline). Within US1: T002–T005 (removals) MUST precede T007–T011 (mechanical fixes in surviving files); T006 depends on T002–T005; T013 is the closing wave
- **US2 (Phase 4)**: Depends on US1 completion (it verifies US1's work)
- **US3 (Phase 5)**: Depends on US1+US2 (the gate can only be installed on a clean codebase)
- **Polish (Phase 6)**: Depends on all user stories

### User Story Dependencies

- **User Story 1 (P1)**: The MVP — delivers the warning-free codebase on its own
- **User Story 2 (P2)**: Verification pass over US1's changes; no new code
- **User Story 3 (P3)**: Enforcement; requires US1+US2 done

### Parallel Opportunities

- T003, T004, T005 touch disjoint files — can run in parallel after T002
- T007–T012 all touch `src/ui/app.rs` (except T007's `trip_file.rs` part) — run sequentially to avoid conflicts
- T020 (AGENTS.md) can run in parallel with T018/T019

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: baseline capture
2. Complete Phase 3: User Story 1 (all fixes, zero warnings)
3. **STOP and VALIDATE**: the three tool commands are clean; tests pass
4. The codebase is warning-free — the user's core request is delivered

### Incremental Delivery

1. Baseline → US1 (warning-free) → validate
2. US2 (quality verification) → validate
3. US3 (structural guard) → validate
4. Polish (fmt, quickstart, final gate) → done

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- The constitution (v1.0.0) Principles I and II are satisfied by US1+US2; the `[lints]` table makes Principle II self-enforcing
- Removing dead code may cascade (unused imports, unused deps) — T013 exists for exactly that; do not stop at the first clean-looking run
- Never hand-edit `Cargo.lock`; use `cargo remove` or drop the Cargo.toml entry and rebuild
- Commit after each task or logical group (Conventional Commits per AGENTS.md)
