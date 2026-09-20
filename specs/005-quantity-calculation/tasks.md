---


description: "Task list for Quantity Calculation feature implementation"

---

# Tasks: Quantity Calculation

**Input**: Design documents from `/specs/005-quantity-calculation/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: The examples below include test tasks. Tests are OPTIONAL - only include them if explicitly requested in the feature specification.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

> **Alignment note (2026-09-20)**: Task statuses were reconciled with the
> shipped application. User Story 5 (packing-list management: naming, JSON
> export/import, `PackingService`) was superseded by the trip-file flow
> (spec 004); its tasks are marked cancelled. The optional contract/
> integration test tasks (T007–T029, T034–T043) were not carried out; the
> quantity rule is covered by unit tests in `src/quantity/calculation.rs`
> and config/trip-file tests in `src/ui/app.rs`.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/`, `tests/` at repository root
- Paths shown below assume single project - adjust based on plan.md structure

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [x] T001 Create project structure per implementation plan
- [x] T002 Initialize Rust project with Cargo.toml dependencies (ratatui, thiserror, serde, anyhow)
- [x] T003 [P] Configure linting (clippy) and formatting (rustfmt) tools

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T004 [P] Create `.config/pack-liste/` directory for configuration and data storage
- [x] T005 [P] Implement global error handling and reporting using `thiserror` and `anyhow`
- [x] T006 [P] Setup logging infrastructure (tracing or log crate)

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Calculate Quantities Based on Weekdays (Priority: P1) 🎯 MVP

**Goal**: Implement quantity calculation based on the number of weekdays for a trip.

**Independent Test**: Verify that quantities update correctly when weekdays are changed.

### Tests for User Story 1 (OPTIONAL - only if tests requested) ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T007 [P] [US1] Contract test for quantity calculation in tests/contract/test_quantity_calculation.rs
- [ ] T008 [P] [US1] Integration test for weekday-based quantity calculation in tests/integration/test_weekdays.rs

### Implementation for User Story 1

- [x] T009 [P] [US1] Create `src/models/weekdays.rs` with Weekdays struct and validation
- [x] T010 [P] [US1] Create `src/models/item.rs` with Item struct, quantity, and packed status
- [x] T011 [US1] Implement `src/quantity/model.rs` with QuantityCalculationInput and QuantityCalculationOutput
- [x] T012 [US1] Implement `src/quantity/calculation.rs` with base quantity calculation logic based on weekdays
- [x] T013 [US1] Implement `src/services/quantity_service.rs` with calculate_weekdays function
- [x] T014 [US1] Add validation and error handling for invalid weekdays
- [x] T015 [US1] Add logging for quantity calculation operations

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently

---

## Phase 4: User Story 2 - Weigh Quantities Based on Travel Style (Priority: P1)

**Goal**: Adjust quantity calculations based on the user's travel style (minimalist, luxury, backpacker).

**Independent Test**: Verify that quantities update correctly when travel style is changed.

### Tests for User Story 2 (OPTIONAL - only if tests requested) ⚠️

- [ ] T016 [P] [US2] Contract test for travel style adjustment in tests/contract/test_travel_style.rs
- [ ] T017 [P] [US2] Integration test for travel style-based quantity calculation in tests/integration/test_travel_style.rs

### Implementation for User Story 2

- [x] T018 [P] [US2] Create `src/models/travel_style.rs` with TravelStyle enum and validation
- [x] T019 [US2] Implement `src/quantity/calculation.rs` with travel style adjustment logic
- [x] T020 [US2] Update `src/services/quantity_service.rs` with calculate_travel_style function
- [x] T021 [US2] Add validation and error handling for invalid travel style

**Checkpoint**: User Story 2 should work independently and integrate with User Story 1

---

## Phase 5: User Story 3 - Weigh Quantities Based on Weather Input (Priority: P1)

**Goal**: Adjust quantity calculations based on weather conditions (hot, cold, rainy).

**Independent Test**: Verify that quantities update correctly when weather input is changed.

### Tests for User Story 3 (OPTIONAL - only if tests requested) ⚠️

- [ ] T022 [P] [US3] Contract test for weather adjustment in tests/contract/test_weather.rs
- [ ] T023 [P] [US3] Integration test for weather-based quantity calculation in tests/integration/test_weather.rs

### Implementation for User Story 3

- [x] T024 [P] [US3] Create `src/models/weather.rs` with WeatherInput enum and validation
- [x] T025 [US3] Implement `src/quantity/calculation.rs` with weather adjustment logic
- [x] T026 [US3] Update `src/services/quantity_service.rs` with calculate_weather function
- [x] T027 [US3] Add validation and error handling for invalid weather input

**Checkpoint**: User Story 3 should work independently and integrate with User Stories 1 and 2

---

## Phase 6: User Story 4 - Weigh Quantities Based on Activities (Priority: P1)

**Goal**: Adjust quantity calculations based on planned activities (hiking, beach, city tour).

**Independent Test**: Verify that quantities update correctly when activities are changed.

### Tests for User Story 4 (OPTIONAL - only if tests requested) ⚠️

- [ ] T028 [P] [US4] Contract test for activity adjustment in tests/contract/test_activities.rs
- [ ] T029 [P] [US4] Integration test for activity-based quantity calculation in tests/integration/test_activities.rs

### Implementation for User Story 4

- [x] T030 [P] [US4] Create `src/models/activity.rs` with Activity enum and validation
- [x] T031 [US4] Implement `src/quantity/calculation.rs` with activity adjustment logic
- [x] T032 [US4] Update `src/services/quantity_service.rs` with calculate_activities function
- [x] T033 [US4] Add validation and error handling for invalid activities

**Checkpoint**: User Story 4 should work independently and integrate with User Stories 1, 2, and 3

---

## Phase 5: User Story 5 - Packing List Management (Priority: P2)

**Goal**: Implement packing list management including creation, selection, renaming, export, and import functionality.

**Independent Test**: Verify that packing lists can be created, renamed, exported, imported, and selected correctly.

### Tests for User Story 5 (OPTIONAL - only if tests requested) ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T034 [P] [US5] Contract test for packing list management in tests/contract/test_packing_list.rs
- [ ] T035 [P] [US5] Integration test for packing list export/import in tests/integration/test_packing_io.rs

### Implementation for User Story 5

> Superseded by the trip-file flow (spec 004). Cancelled 2026-09-20.

- [x] T036 [P] [US5] Create `src/models/packing_list.rs` with PackingList struct, name, and items (kept as the in-memory model)
- [x] T037 [US5] Implement `src/services/packing_service.rs` with create, rename, export, and import functions (superseded; code currently unused)
- [x] T038 [US5] Add validation for packing list names (superseded; trips are identified by file path)
- [x] T039 [US5] Implement packing list persistence in `~/.config/pack-liste/` (superseded; trips persist to user-chosen files)
- [x] T040 [US5] Add error handling for file I/O operations (fulfilled by the trip-file module)
- [x] T041 [US5] Add logging for packing list operations (superseded)

**Checkpoint**: User Story 5 should work independently and integrate with User Stories 1-4

---

## Phase 6: User Story 6 - Editable Item Quantities and Packed Status (Priority: P2)

**Goal**: Implement editable item quantities and packed status toggles.

**Independent Test**: Verify that users can edit quantities and toggle packed status.

### Tests for User Story 6 (OPTIONAL - only if tests requested) ⚠️

- [ ] T042 [P] [US6] Contract test for editable quantities in tests/contract/test_editable_quantities.rs
- [ ] T043 [P] [US6] Integration test for packed status toggles in tests/integration/test_packed_status.rs

### Implementation for User Story 6

- [x] T044 [US6] Update `src/models/item.rs` with editable quantity and packed status fields
- [x] T045 [US6] Implement `src/services/item_service.rs`
- [x] T046 [US6] Add validation for quantity values
- [x] T047 [US6] Add UI components for editing quantities and toggling packed status

**Checkpoint**: User Story 6 should work independently and integrate with User Stories 1-5

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [x] T048 [P] Documentation updates in `docs/quantity-calculation.md`
- [ ] T049 Code cleanup and refactoring across all modules
- [ ] T050 [P] Additional unit tests in tests/unit/
- [ ] T051 Run quickstart.md validation scenarios
- [ ] T052 Final validation and cleanup

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3+)**: All depend on Foundational phase completion
  - User stories can then proceed in parallel (if staffed)
  - Or sequentially in priority order (P1 → P2 → P3)
- **Polish (Final Phase)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational (Phase 2) - Integrates with US1 but should be independently testable
- **User Story 3 (P3)**: Can start after Foundational (Phase 2) - Integrates with US1/US2 but should be independently testable
- **User Story 4 (P4)**: Can start after Foundational (Phase 2) - Integrates with US1/US2/US3 but should be independently testable
- **User Story 5 (P5)**: Can start after Foundational (Phase 2) - Integrates with US1-4 but should be independently testable
- **User Story 6 (P6)**: Can start after Foundational (Phase 2) - Integrates with US1-5 but should be independently testable

### Within Each User Story

- Tests (if included) MUST be written and FAIL before implementation
- Models before services
- Services before endpoints
- Core implementation before integration
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel (within Phase 2)
- Once Foundational phase completes, all user stories can start in parallel (if team capacity allows)
- All tests for a user story marked [P] can run in parallel
- Models within a story marked [P] can run in parallel
- Different user stories can be worked on in parallel by different team members
- User Stories 5 and 6 can run in parallel with each other and with US1-4

---

## Parallel Example: User Story 1

```bash
# Launch all tests for User Story 1 together (if tests requested):
Task: "Contract test for quantity calculation in tests/contract/test_quantity_calculation.rs"
Task: "Integration test for weekday-based quantity calculation in tests/integration/test_weekdays.rs"

# Launch all models for User Story 1 together:
Task: "Create Weekdays struct in src/models/weekdays.rs"
Task: "Create Item struct in src/models/item.rs"
Task: "Implement QuantityCalculationInput in src/quantity/model.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: Test User Story 1 independently
5. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Deploy/Demo (MVP!)
3. Add User Story 2 → Test independently → Deploy/Demo
4. Add User Story 3 → Test independently → Deploy/Demo
5. Add User Story 4 → Test independently → Deploy/Demo
6. Add User Story 5 → Test independently → Deploy/Demo
7. Add User Story 6 → Test independently → Deploy/Demo
8. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1
   - Developer B: User Story 2
   - Developer C: User Story 3
   - Developer D: User Story 4
   - Developer E: User Story 5
   - Developer F: User Story 6
3. Stories complete and integrate independently

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence
