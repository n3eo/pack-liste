---
description: "Task list for 023-harmonized-navigation-text"
---

# Tasks: Harmonized Navigation Text

**Input**: Design documents from `/specs/023-harmonized-navigation-text/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Included per story — the spec's success criteria are property-based and ship as tests (Constitution III).

**Organization**: Tasks are grouped by user story. US1 (canonical wording) and US2 (de-duplication) are both P1; US1 lands first because US2's de-duplication restructures the same lines.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

Single project: `src/` at repository root. All work in `src/ui/hints.rs`, `src/ui/help.rs`, `src/ui/screens/main.rs`, and `src/ui/tests/hints.rs`.

---

## Phase 1: Foundational — The Canonical Table

**Purpose**: The single-source binding table every surface and the help screen derive from (FR-005, research R3).

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [X] T001 Create the canonical binding table in `src/ui/hints.rs`: a `const` array of `(key, action)` entries holding every canonical pair from [contracts/canonical-bindings.md](contracts/canonical-bindings.md) (per-meaning entries for the 5 drifted keys per research R2: `Enter` Confirm/Save/Continue/New line, `Esc` Cancel/Back, `F` Fullscreen/Edit path, `Space` Select/Packed/Toggle, `Tab` Switch panel/Switch section/Next field); each entry as a named `const` tuple so surfaces reference it by name
- [X] T002 Rewrite `hint_entries(surface)` in `src/ui/hints.rs` to compose each of the nine surfaces from the canonical `const` entries (references, no re-typed strings); the compositions match the contract's surface lists exactly (FR-002); remove the per-surface string literals

**Checkpoint**: The gate is green; every action string exists exactly once in the source; the existing contract test `hint_entries_match_the_contract_binding_sets` passes against the new compositions (update its expected sets to the canonical wording where it drifted: `Enter→Continue` on Welcome, `Space→Toggle` on AddEditPopup).

---

## Phase 2: User Story 1 — One binding, one wording, everywhere (Priority: P1) 🎯 MVP

**Goal**: Every key shows the same action text on every surface where it appears (SC-001); the help screen agrees (FR-004).

**Independent Test**: `cargo test canonical_action_text_is_unique_per_key` and `cargo test help_tokens_match_canonical_table` pass; the quickstart §1/§3/§5 checks show identical wording per key across surfaces.

### Implementation for User Story 1

- [X] T003 [US1] Add the test `canonical_action_text_is_unique_per_key` in `src/ui/tests/hints.rs`: for every key, collect its action texts across all nine surfaces — the multiset must equal the canonical table's per-meaning entries for that key (no surface shows a text that is not the canonical one for its meaning)
- [X] T004 [US1] Derive the help screen's key tokens from the canonical table in `src/ui/help.rs`: `help_content()` uses the canonical key strings (single source); keep the verbose descriptions, adjusting any that contradict the canonical action word (e.g. the `↑/↓` descriptions contain "Navigate", `Tab` descriptions contain "Switch panel"/"Switch section"/"Next field" per surface)
- [X] T005 [US1] Add the test `help_tokens_match_canonical_table` in `src/ui/tests/hints.rs`: every key token the help screen lists exists in the canonical table, and each help description contains its key's canonical action word
- [X] T006 [US1] Update `TUI_USAGE.md` where it restates hint wording (the key tables per screen) to the canonical wording

**Checkpoint**: US1 done — same key, same words, everywhere; the help screen agrees.

---

## Phase 3: User Story 2 — Each surface states its bindings once (Priority: P1)

**Goal**: No (key, action) pair is visible twice on one screen (SC-002) — the two main panels stop restating their 7 shared pairs.

**Independent Test**: `cargo test no_pair_repeats_on_one_screen` passes; the quickstart §5 main-screen check shows the left footer without the shared pairs and the right footer carrying them.

### Implementation for User Story 2

- [X] T007 [US2] De-duplicate the main screen's hint lines in `src/ui/hints.rs` + `src/ui/screens/main.rs` per research R4: the LeftPanel composition keeps only the left-specific entries (`Space Select`, `Ctrl+Space Category`, `A Add`, `E Edit`, `D Delete`, `C Collapse`); the 7 shared pairs (`↑/↓ Navigate`, `Tab Switch panel`, `F Fullscreen`, `R`/`Ctrl+R`/`Ctrl+Shift+R Recalc…`, `? Help`) move to the RightPanel composition only; both panels' renderers keep the existing adaptive layout (`hint_rows` packing, `collapsed_hint_line`) unchanged (FR-007)
- [X] T008 [US2] Add the test `no_pair_repeats_on_one_screen` in `src/ui/tests/hints.rs`: the LeftPanel and RightPanel compositions are pair-disjoint; no composition contains a duplicate pair; the Config + ModifierEditor compositions are pair-disjoint (the popup lists only popup-specific bindings)
- [X] T009 [US2] Update the contract test `hint_entries_match_the_contract_binding_sets` in `src/ui/tests/hints.rs` to the de-duplicated LeftPanel/RightPanel sets; update `TUI_USAGE.md`'s main-screen key table to match

**Checkpoint**: US2 done — nothing restated on one screen; both panels' own bindings still visible.

---

## Phase 4: User Story 3 — The wording stays harmonized (Priority: P2)

**Goal**: The canonical table is the single source; drift is impossible without a test failing (SC-004, FR-005).

**Independent Test**: The quickstart §4 grep shows each canonical action string exactly once in `src/ui/hints.rs`; the three property tests fail on any introduced drift.

### Implementation for User Story 3

- [X] T010 [US3] Add the test `canonical_entries_are_all_used` in `src/ui/tests/hints.rs`: every canonical table entry is referenced by at least one surface composition or the help screen (no dead entries — Constitution I)
- [X] T011 [US3] Verify the single-source property in `src/ui/tests/hints.rs`: assert the canonical table contains no duplicate (key, action) pairs and that `hint_entries` returns only references into the table (e.g. by asserting each surface pair is `eq` to some canonical entry — pointer-identity where the language allows, value-equality otherwise)

**Checkpoint**: US3 done — one source, pinned by tests.

---

## Phase 5: Polish & Cross-Cutting Concerns

- [X] T012 Run the quickstart validation end to end ([quickstart.md](quickstart.md) §1–§6): the three property tests, the single-source greps, the visual check on every screen, and the behavior-unchanged pass (FR-006)
- [X] T013 Final gate: `cargo check`, `cargo clippy`, `cargo clippy --tests`, `cargo test`, `cargo fmt --check` all exit 0 with zero warnings

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Foundational)**: blocks everything — the table must exist before any composition references it.
- **Phase 2 (US1)**: depends on Phase 1.
- **Phase 3 (US2)**: depends on Phase 2 (the de-duplication restructures the compositions US1 just canonicalized).
- **Phase 4 (US3)**: depends on Phase 3 (the single-source property is asserted against the final compositions).
- **Phase 5**: depends on all stories.

### Within Each User Story

- Tests before/with the implementation they pin (T003 with the table; T005/T007/T008 with their changes).
- The contract test (`hint_entries_match_the_contract_binding_sets`) is updated in the same task as the composition change it pins.

### Parallel Opportunities

- T006 (TUI_USAGE.md) is parallel-safe with T004/T005 (different files).
- T010/T011 are parallel-safe with each other (same file, disjoint assertions — sequence in practice).

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1 (the canonical table).
2. Complete Phase 2 (US1 — the wording is harmonized everywhere).
3. **STOP and VALIDATE**: the uniqueness and help tests pass; every screen shows the canonical wording.
4. The core complaint (drift) is resolved; US2's de-duplication can follow independently.

### Incremental Delivery

1. Phase 1 → the single source exists.
2. Phase 2 (US1) → same key, same words, everywhere.
3. Phase 3 (US2) → nothing restated on one screen.
4. Phase 4 (US3) → drift is pinned by tests.
5. Phase 5 → quickstart + gate.

---

## Notes

- The existing hint tests are the contract: `hint_keys_are_help_tokens`, `hint_entries_match_the_contract_binding_sets`, `hint_rows_fit_width_and_keep_every_entry` must keep passing (updated in the same change as the wording they pin, per the plan's constraints).
- No key behavior changes (FR-006) — if a test failure implies a behavior change, the test or the change is wrong.
- The `+/- Weekdays` and `Home/End Line start/end` entries are single-surface; they join the canonical table like the rest (single source) but have no cross-surface harmonization to do.
- Commit after each phase; Conventional Commits (`refactor:` for the table/composition work, `docs:` for TUI_USAGE.md).
