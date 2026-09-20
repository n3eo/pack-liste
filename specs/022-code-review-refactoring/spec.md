# Feature Specification: Code Review Remediation — Understandability, Readability, Separation of Logic

**Feature Branch**: `022-code-review-refactoring`

**Created**: 2026-09-20

**Status**: Draft

**Input**: User description: "you are a very critical senior developer that conducts a code review, focus on understandability, readability, separation of loics, perform a review and suggest new user stories and tasks"

## Review Context

A critical senior-developer code review was performed on the current codebase, focused on understandability, readability, and separation of logic. The review found the application functionally sound (all quality gates green, 39 tests passing) but structurally degraded: the entire application lives in one 4,917-line file, input handling is a single ~470-line function with ~63 match arms, identical logic is duplicated in up to four places, dead model fields and an unused dependency exist, and the project's own documentation describes a code structure that no longer exists.

This specification turns the review findings into remediation work. The stakeholder is the developer/maintainer, not the end user: every story is behavior-preserving internal-quality work, in the same spirit as spec 009 (warning-free codebase). No user-visible feature changes are in scope.

### Verified findings (evidence base)

| # | Finding | Evidence (verified in code) |
|---|---------|------------------------------|
| F1 | Monolithic UI module | `src/ui/app.rs` is 4,917 lines; state, input handling, drawing, config I/O, and a 33-test module (~964 lines, starting at line 233) all live in one file, with tests placed before the code they test |
| F2 | Oversized input handler | `handle_input` spans ~473 lines (line 2078) with ~63 match arms and repeated screen/mode guard conditions; `handle_config_screen_input` ~165 lines (line 2751); `handle_add_edit_input` ~119 lines (line 2916) |
| F3 | Duplicated logic | `get_unique_categories` (line 1877) and `get_all_categories` (line 1936) have identical bodies; `draw_add_item_popup` (line 4528) and `draw_edit_item_popup` (line 4677) differ only in title and one field; category-grouping map construction appears 4× (lines 1298, 1324, 4150, 4347); `cycle_travel_style`/`cycle_weather` (lines 1950, 1966) repeat the same cycle logic |
| F4 | Per-frame full-state clone | `App::draw` clones the entire draw state (including the full packing list) every frame (line 2002); ~50 `clone()` calls in the file |
| F5 | Dead model fields and dependency | `Item.id`, `Item.base_quantity`, `PackingList.id`, `PackingList.name`, `PackingList.file_path` are never read outside their definitions; `PackingList::new` pulls in the `chrono` dependency solely to timestamp the unused id — `chrono` is used nowhere else |
| F6 | Misleading naming | `Item.editable` actually means "selected for the trip" (28 references); `Screen`, `Focus`, `InputField`, `InputMode` are `pub` with zero external users |
| F7 | Swallowed errors | `.ok()` discards save/recalculate errors at 6 call sites (lines 2785, 2874, 3111, 3114, 3135, 3139); one production `unwrap()` (line 2766); `main.rs` reports errors twice (`eprintln!` then `bail!`) |
| F8 | Stale documentation | `docs/quantity-calculation.md` documents a nonexistent `src/services/` directory, nonexistent error types (`QuantityError`, `ConfigError`, `UiError`), and JSON storage (actual: YAML) |
| F9 | Unused logging setup | `tracing`/`tracing-subscriber` are initialized in `main.rs` but zero log statements exist in the source |

## User Scenarios & Testing *(mandatory)*

### User Story 1 - The application is split into cohesive modules (Priority: P1)

A developer opens the UI source and finds a module tree where each concern has a home: application state, per-screen input handling, per-screen drawing, configuration I/O, and the test suite each live in their own file, and no single file holds the whole application. The 4,917-line monolith is gone. A developer looking for "how the welcome screen handles keys" or "where the final list is drawn" can navigate to the right file by name, without scrolling through unrelated code. Behavior is unchanged: every existing test passes without modification, and the application runs exactly as before.

**Why this priority**: The monolith (F1) is the root cause amplifying every other finding — duplicated logic survives because nobody sees both copies, oversized handlers grow because there is no natural place to split, and tests sit before the code they test because there is nowhere else to put them. Splitting first makes every later story cheaper and safer.

**Independent Test**: The full quality gate (`cargo check`, `cargo clippy`, `cargo clippy --tests`, `cargo test`, `cargo fmt --check`) passes after the split, with all 39 existing tests unmodified; the application starts and navigates identically.

**Acceptance Scenarios**:

1. **Given** the single-file UI module, **When** the split is complete, **Then** each concern (state, input, drawing, config I/O, tests) lives in its own module and no source file exceeds a maintainable size.
2. **Given** the split codebase, **When** a developer searches for a screen's input handling, **Then** it is found in a module dedicated to input handling for that screen, not inside a monolithic dispatcher.
3. **Given** the split codebase, **When** the full test suite runs, **Then** all existing tests pass unmodified.
4. **Given** the split codebase, **When** the application is run manually, **Then** all screens, navigation, and interactions behave exactly as before the split.

---

### User Story 2 - Input handling is decomposed and duplication is removed (Priority: P1)

A developer reads the input handling and finds a small dispatcher that routes each key event to a focused handler for the current screen and mode, instead of a ~473-line function with ~63 match arms and repeated guard conditions. The duplicated logic identified in the review is gone: the two identical category-listing functions are one, the add/edit popups share their rendering, the category-grouping construction exists once, and the travel-style/weather cycling uses one shared mechanism. Reading any single handler, a developer can understand it without holding the entire application in their head.

**Why this priority**: The oversized handler (F2) and the duplication (F3) are the two most direct hits on the review's stated focus — understandability and readability. They are also the highest-churn code in the file, so every future feature pays the cost of their current shape.

**Independent Test**: All quality gates pass; a key-event trace (manual or scripted) through every screen and mode produces identical behavior before and after; the duplicated functions no longer exist as separate copies.

**Acceptance Scenarios**:

1. **Given** the monolithic input handler, **When** key handling is decomposed, **Then** a dispatcher routes events to per-screen/per-mode handlers, and no single handler carries the combined logic of multiple screens.
2. **Given** the two identical category functions, **When** duplication is removed, **Then** exactly one implementation remains and all call sites use it.
3. **Given** the near-identical add/edit popup renderers, **When** they are unified, **Then** a single shared renderer serves both, differing only in the title and the mode field.
4. **Given** the four copies of category-grouping construction, **When** they are extracted, **Then** one shared helper builds the grouping and all four call sites use it.
5. **Given** the parallel travel-style/weather cycling functions, **When** they are unified, **Then** one cycle mechanism handles both, parameterized by the list being cycled.
6. **Given** the decomposed input handling, **When** the full test suite runs, **Then** all existing tests pass and manual navigation through every screen behaves identically.

---

### User Story 3 - Dead code, dead fields, and dead dependencies are removed (Priority: P2)

A developer reviews the data model and finds it contains only fields that are actually used. The five never-read fields identified in the review (`Item.id`, `Item.base_quantity`, `PackingList.id`, `PackingList.name`, `PackingList.file_path`) are deleted along with their construction code, and the `chrono` dependency — which exists solely to timestamp one of those dead fields — is removed from the project. The dependency footprint shrinks and the model reads as an honest description of the data the application actually manages.

**Why this priority**: Dead fields and a dead dependency are direct violations of the project constitution's no-unused-code principle (Principle I). They are cheap to remove, carry zero behavioral risk, and shrink the model before the renames of User Story 4 touch it.

**Independent Test**: All quality gates pass; a search for the removed field names returns nothing; the dependency list no longer contains `chrono`; the serialized output format is unchanged for all fields that remain.

**Acceptance Scenarios**:

1. **Given** the five never-read model fields, **When** they are removed, **Then** no field exists in the data model that is not read by the application, and the quality gates stay green.
2. **Given** the `chrono` dependency used only by dead code, **When** the dead field is removed, **Then** the dependency is dropped from the manifest and the build still succeeds.
3. **Given** the removed fields, **When** a packing list is saved and reloaded, **Then** the persisted data is complete and correct for all remaining fields.
4. **Given** the removal is complete, **When** the full test suite runs, **Then** all existing tests pass.

---

### User Story 4 - Names say what the code means (Priority: P2)

A developer reads the model and UI types and finds that identifiers describe their actual role: the field that tracks whether an item is selected for the trip is named for selection, not for editability; types are visible only where other modules genuinely use them, not blanket-public. The misleading names identified in the review are corrected, and a reader can trust that a name means what it says.

**Why this priority**: Misleading naming (F6) is a slow, compounding cost: every future reader pays a re-translation tax, and the wrong name invites wrong usage. It is scheduled after the dead-field removal (User Story 3) so renames touch a smaller, honest model.

**Independent Test**: All quality gates pass; a search for the old misleading names returns nothing; behavior is unchanged.

**Acceptance Scenarios**:

1. **Given** the field that means "selected for the trip" but is named for editability, **When** it is renamed, **Then** the new name describes selection and all references are updated consistently.
2. **Given** types that are public but have no external users, **When** visibility is tightened, **Then** only types genuinely used across modules remain public, and the build stays green.
3. **Given** the renames, **When** the full test suite runs, **Then** all existing tests pass and the application behaves identically.

---

### User Story 5 - Errors are handled, not swallowed (Priority: P2)

A developer traces what happens when saving the configuration or recalculating quantities fails and finds the failure is surfaced — propagated to the caller, reported to the user, or logged — instead of being silently discarded by `.ok()`. The one production `unwrap()` is replaced with explicit handling, and the application's entry point reports an error exactly once, not twice. The unused logging setup identified in the review is either removed or put to real use as part of this error-reporting work.

**Why this priority**: Swallowed errors (F7) are a correctness-adjacent readability problem: the code looks like it handles persistence, but failures vanish. Fixing them after the module split means each call site lands in a focused module where the right handling is obvious.

**Independent Test**: All quality gates pass; a forced save failure (e.g., unwritable path) produces a visible error for the user or a log entry, not silence; no production `unwrap()` remains on the affected paths.

**Acceptance Scenarios**:

1. **Given** the six call sites that discard save/recalculate errors, **When** error handling is fixed, **Then** each failure is propagated, reported, or logged — none is silently discarded.
2. **Given** the production `unwrap()` on the modifier editor, **When** it is replaced, **Then** the code handles the absent-editor case explicitly without panicking.
3. **Given** the entry point that reports errors twice, **When** reporting is unified, **Then** an application error is reported exactly once to the user.
4. **Given** the initialized-but-unused logging setup, **When** this story completes, **Then** either the logging setup is removed or real log statements exist at the error sites.
5. **Given** the fixed error handling, **When** the full test suite runs, **Then** all existing tests pass.

---

### User Story 6 - Documentation matches the code (Priority: P3)

A developer reads the project's quantity-calculation documentation and finds it describes the code as it actually is: the real module tree, the real storage format (YAML), the real error handling, and the current modifier semantics. The stale sections identified in the review are rewritten or removed, and the tests sit adjacent to the code they test, so documentation and code tell the same story.

**Why this priority**: Stale documentation (F8) misleads every new reader and erodes trust in all documentation. It is scheduled last because it must describe the final post-refactor structure — writing it earlier would mean writing it twice.

**Independent Test**: Every structural claim in the documentation (module paths, storage format, error types) is verifiably true against the codebase; the full quality gate passes.

**Acceptance Scenarios**:

1. **Given** documentation describing a nonexistent directory structure, error types, and storage format, **When** it is updated, **Then** every path, type, and format it mentions exists in the codebase.
2. **Given** the test module separated from the code under test, **When** tests are relocated, **Then** each test lives adjacent to the module it exercises.
3. **Given** the updated documentation, **When** a developer follows any instruction in it, **Then** the instruction works as written.

---

### Edge Cases

- What happens if splitting the monolith changes behavior in a way the existing tests do not catch?
  **Answer**: Each split step is followed by the full quality gate and a manual navigation pass over every screen; the split is done in small moves, one concern at a time, so any regression is attributable to a single step.
- What happens if the add/edit popup unification cannot capture a genuine behavioral difference between the two popups?
  **Answer**: The shared renderer is parameterized by the differences (title, mode field); if a difference proves structural rather than parametric, the two renderers remain separate and the attempt is documented as the justified exception.
- What happens if removing a "dead" field turns out to be load-bearing (e.g., serialized output consumers rely on it)?
  **Answer**: The removal is verified against the persisted file format: a list saved before the change must load correctly after it. The persisted format is the packing list the user edits by hand, so any field a human reader could rely on is treated as part of the contract and kept or migrated deliberately.
- What happens if the per-frame state clone (F4) cannot be removed without a redesign?
  **Answer**: The clone is not blindly removed. The refactor measures or justifies the cost; if the clone is measurably negligible at realistic list sizes, it is documented as an accepted trade-off rather than restructured speculatively.
- What happens if tightening visibility breaks the test module's access?
  **Answer**: Tests are part of the crate and use the appropriate test-only visibility (e.g., `#[cfg(test)]` modules or `pub(crate)`), not blanket `pub`; the gates stay green.
- What if two findings conflict (e.g., a rename makes a split harder)?
  **Answer**: The stories are ordered deliberately — split first, then dead-code removal, then renames, then error handling, then documentation — and each story lands with the gate green, so conflicts are resolved by doing them in sequence, not interleaved.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The UI source MUST be organized into modules separated by concern (state, input handling, drawing, configuration I/O, tests), with no single module holding the entire application.
- **FR-002**: Key event handling MUST be decomposed into a dispatcher routing to per-screen/per-mode handlers; the monolithic input function MUST NOT remain.
- **FR-003**: Duplicated logic MUST be consolidated: the identical category functions, the add/edit popup rendering, the category-grouping construction, and the value-cycling functions MUST each exist as a single shared implementation.
- **FR-004**: All never-read model fields (`Item.id`, `Item.base_quantity`, `PackingList.id`, `PackingList.name`, `PackingList.file_path`) MUST be removed, together with the `chrono` dependency.
- **FR-005**: Identifiers MUST describe their actual meaning; the item selection field MUST NOT be named for editability, and type visibility MUST be restricted to genuinely cross-module use.
- **FR-006**: Save and recalculate failures MUST be surfaced (propagated, reported, or logged) at every call site; no production path MAY silently discard them via `.ok()`, and no production `unwrap()` MAY remain on the affected paths.
- **FR-007**: The application entry point MUST report an error exactly once.
- **FR-008**: The logging setup MUST either be removed or carry real log statements; it MUST NOT remain initialized and unused.
- **FR-009**: The quantity-calculation documentation MUST describe the actual module tree, storage format, and error handling; it MUST NOT reference directories, types, or formats that do not exist.
- **FR-010**: Tests MUST be located adjacent to the modules they test.
- **FR-011**: Every refactor in this feature MUST preserve behavior: all existing tests MUST pass unmodified, and the persisted packing-list format MUST remain loadable.
- **FR-012**: The per-frame full-state clone MUST be either eliminated or explicitly documented as an accepted trade-off with justification.

### Key Entities *(include if feature involves data)*

- **Module boundary**: The separation line between concerns (state, input, drawing, persistence); each review finding is remediated inside a module whose name states its concern.
- **Dispatcher**: The small routing layer that maps a key event to the handler for the current screen and mode; replaces the monolithic match.
- **Dead field**: A model field that is written but never read; removed rather than retained, per the project constitution.
- **Swallowed error**: A fallible operation whose result is discarded; every such site must surface the failure instead.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: No source file in the project exceeds 1,500 lines; the UI concern split is complete and each module is navigable by name.
- **SC-002**: The monolithic input function is gone; no single input handler exceeds 150 lines, and the dispatcher routes by screen and mode.
- **SC-003**: Zero duplicated implementations remain for the four duplication findings (category listing, popup rendering, category grouping, value cycling).
- **SC-004**: The data model contains zero never-read fields, and the dependency manifest no longer lists `chrono`.
- **SC-005**: Zero `.ok()` calls on save/recalculate paths and zero production `unwrap()` calls on the affected paths remain.
- **SC-006**: All 39 existing tests pass unmodified after the full remediation, and the full quality gate (`cargo check`, `cargo clippy`, `cargo clippy --tests`, `cargo test`, `cargo fmt --check`) is green.
- **SC-007**: Every structural claim in the project documentation is verifiable against the codebase.

## Assumptions

- The review findings listed above are complete for the current state of the code; the remediation targets exactly these findings, not a general rewrite. New findings discovered during the work are recorded but do not expand this spec's scope without user agreement.
- All refactoring is behavior-preserving; "better structure" never justifies a user-visible change. The 39 existing tests are the behavioral contract, supplemented by manual navigation passes over every screen.
- The persisted packing-list format is a human-editable contract (established in earlier specs); field removal is safe only for fields a human reader cannot rely on. The five dead fields are written by the application but never read, and are assumed safe to remove after verifying round-trip loading.
- The per-frame state clone is a performance question, not a correctness one; it is measured or justified, not speculatively redesigned. At realistic list sizes (hundreds of items), the clone cost is assumed negligible pending measurement.
- The module split follows the codebase's existing directory conventions (`src/ui/`, `src/models/`, `src/config/`, `src/quantity/`); no new top-level architecture is introduced.
- Removing `chrono` touches the dependency manifest; the lockfile is updated only through the toolchain, never by hand, per the project guardrails.
- The `tracing` setup is removed rather than extended, unless error-handling work (User Story 5) makes log statements the natural reporting mechanism at specific sites.
- Story ordering (split → duplication → dead code → renames → errors → documentation) is the planned sequence; each story lands independently with the gate green, so partial completion still yields a viable, better codebase.
