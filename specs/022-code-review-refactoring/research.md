# Research: Code Review Remediation

**Feature**: 022-code-review-refactoring | **Date**: 2026-09-20 | **Spec**: [spec.md](spec.md)

Phase 0 research. Every unknown was resolved by direct inspection of the
current codebase; no external research was required. Findings below are
the decisions the plan and tasks build on.

## R1. Are the five "dead" model fields really safe to remove?

**Decision**: Yes — all five (`Item.id`, `Item.base_quantity`,
`PackingList.id`, `PackingList.name`, `PackingList.file_path`) are
written but never read, and none reaches either persisted format.

**Rationale**: Verified by grep across `src/`:
- `Item.id` / `Item.base_quantity`: constructed in `Item::new` and
  `load_config`/`create_default_config` (`src/ui/app.rs:3087, 3175,
  3228`), never read. `Item` derives `Serialize`, but the only
  serialization path is `AppConfig`/`SavedItem` (`src/ui/app.rs:134`),
  which has its own `name`/`categories`/`quantity`/`auto_quantity`
  fields — `Item` itself is never serialized.
- `PackingList.id` / `.name` / `.file_path`: set in `PackingList::new`
  (`src/models/packing_list.rs:8-11`), never read anywhere. The trip
  file (`src/config/trip_file.rs:13`) stores `weekdays`, `travel_style`,
  `weather`, `items: Vec<TripItem>` — no list id/name/path. The config
  file (`AppConfig`) stores no list identity either.
- `chrono` exists solely to timestamp the dead `PackingList.id`
  (`src/models/packing_list.rs:19`); removing the field removes the
  dependency.

**Alternatives considered**: Keep the fields "for future features" —
rejected: the constitution (Principle I) explicitly forbids keeping code
for future use. Migrate the persisted format — unnecessary: the fields
never reach a file.

## R2. Does the `editable` rename touch the persisted format?

**Decision**: No. Rename `Item.editable` → `Item.selected` freely; the
persisted formats (`SavedItem` in the config, `TripItem` in the trip
file) do not contain an `editable`/`selected` field.

**Rationale**: `SavedItem` (`src/ui/app.rs:139`) serializes
`name/categories/quantity/auto_quantity`; `TripItem`
(`src/config/trip_file.rs:25`) serializes `name/category/packed`. The
selection state is derived at load: `load_config` sets `editable: true`
via `Item::new` defaults, and reconcile logic
(`TripFile::reconcile`) matches by name/category. A rename is purely
internal.

**Alternatives considered**: Rename to `is_selected` — viable but less
idiomatic for a `bool` field; `selected` chosen.

## R3. How should the monolith be split — and what is the target module tree?

**Decision**: Split `src/ui/app.rs` (4,917 lines) into a `src/ui/`
module tree by concern, keeping `src/app.rs` as the thin entry shim it
already is:

```text
src/ui/
├── mod.rs            # re-exports the public surface (App, run entry)
├── state.rs          # AppDrawState, Screen, Focus, FocusSide, InputField,
│                     # InputMode, AppState, cursor/selection state ops
├── input.rs          # the dispatcher: routes key events to handlers
├── handlers/         # per-screen/per-mode handlers
│   ├── mod.rs
│   ├── welcome.rs
│   ├── main.rs
│   ├── dialogs.rs    # add/edit/delete popups
│   ├── config.rs     # config screen + modifier editor
│   └── editor.rs     # raw config editor
├── draw.rs           # App::draw dispatch + shared draw helpers
├── screens/          # per-screen drawing
│   ├── mod.rs
│   ├── welcome.rs
│   ├── main.rs
│   ├── help.rs
│   ├── config.rs
│   └── editor.rs
├── hints.rs          # hint entries, help tokens, hint rendering
├── help.rs           # help screen content
├── config_io.rs      # AppConfig, SavedItem, load/save/default config
└── tests/            # the 33 tests, split alongside the code they test
    ├── mod.rs
    ├── state.rs
    ├── input.rs
    └── ...
```

**Rationale**: The split follows the concern boundaries the file
already exhibits internally (state structs at the top, handlers in the
middle, draw functions at the bottom, tests in between). Each module
has one reason to change. The `handlers/` and `screens/` sub-trees
mirror each other so a developer can navigate by name. Tests move
adjacent to their subject (constitution Principle III, spec FR-010).

**Alternatives considered**:
- Split into fewer, larger files (e.g., one `input.rs` of 1,000 lines) —
  rejected: recreates the monolith at a smaller scale and leaves
  per-screen logic entangled.
- Full `screens/` crate-per-screen architecture — rejected: overkill
  for a single-binary TUI; modules suffice.
- Keep tests in one `tests.rs` — rejected: FR-010 requires adjacency to
  the code under test.

## R4. How should the input dispatcher be designed?

**Decision**: A two-level dispatcher in `src/ui/input.rs`:
1. Match on `screen` → per-screen handler module.
2. Within each screen handler, match on `input_mode` (or focus) →
   focused handler function.

Each handler is a small function taking `&mut AppDrawState` (and the
key), returning `Result<HandlerOutcome>` where `HandlerOutcome` is
`Handled` or `Exit`. Global keys (Ctrl+C, Ctrl+E, q, ?, O) are checked
first in the dispatcher, before screen routing.

**Rationale**: The current `handle_input` (~473 lines, ~63 match arms)
mixes global, screen, and mode logic in one match. The two-level
dispatch makes each handler readable in isolation and gives the
help/hint contract tests a stable structure to verify against. The
`HandlerOutcome` pattern avoids the boolean-return ambiguity of the
current `Result<bool>`.

**Alternatives considered**:
- Table-driven keymap (key → action enum → handler) — rejected for
  now: the bindings are mode-dependent and a full action-enum would
  be a large speculative redesign; the two-level match is the minimal
  faithful decomposition.
- Keep `Result<bool>` — rejected: ambiguous semantics (exit vs.
  handled) that reviewers must trace.

## R4b. What happens to the help/hint contract tests during the split?

**Decision**: They move with the code they test and keep passing
unchanged. The binding tables live in `hints.rs`; the tests asserting
them move to `tests/hints.rs`.

**Rationale**: `help_lists_every_binding_token`,
`help_screen_text_covers_all_handler_bindings`,
`hint_keys_are_help_tokens`, `hint_entries_match_the_contract_binding_sets`,
`hint_lines_use_the_unified_style`, `hint_rows_fit_width_and_keep_every_entry`
(`src/ui/app.rs:631-901`) encode the spec 016 contract
(`specs/016-harmonized-ui-navigation/contracts/hints-and-bindings.md`).
They are the behavioral guard for the split: if the split changes any
binding, these tests fail. They must not be weakened.

**Alternatives considered**: Rewrite the tests against the new module
structure — unnecessary: they test observable content (help text, hint
entries), not module paths.

## R5. How should the duplicated logic be consolidated?

**Decision**: One shared implementation per duplication site:
- `get_unique_categories` + `get_all_categories` → one
  `categories()` method on `AppDrawState` (or free function over the
  item list).
- `draw_add_item_popup` + `draw_edit_item_popup` → one
  `draw_item_dialog` parameterized by `PopupFields` (which already
  exists and carries the differences: title via mode, `auto_quantity`
  presence).
- The 4× category-grouping (`BTreeMap<String, Vec<...>>` at lines
  1298, 1324, 4150, 4347) → one `group_items_by_category` helper.
- `cycle_travel_style` + `cycle_weather` → one `cycle_value(list,
  current, direction)` helper.

**Rationale**: Each pair differs only in data, not logic. The
`PopupFields` struct already bundles the dialog differences — the two
draw functions are the same rendering modulo title and the Mode field.
A single parameterized implementation is the honest structure.

**Alternatives considered**: Keep both popup functions but extract
shared sub-blocks — rejected: the functions are ~150 lines each and
the shared portion is ~90%; a single parameterized function is
clearer than fragmenting rendering across helpers.

## R6. How should the swallowed errors be surfaced?

**Decision**: Introduce a status-line mechanism in `AppDrawState`:
`status: Option<StatusMessage>` where `StatusMessage` carries
`message: String` and `kind: StatusKind` (`Error`/`Info`). Save and
recalculate call sites replace `.ok()` with `.inspect_err(|e|
state.set_status(StatusKind::Error, e))`-style surfacing (propagated
where the caller already returns `Result`, status line otherwise).
The status line renders in the main/config screen footer area.

**Rationale**: The TUI already has a precedent: the raw config editor
surfaces YAML errors via `config_editor_error: Option<String>`
(`src/ui/app.rs:1237, 2706, 2711`) rendered in its footer
(`src/ui/app.rs:4109`). Extending this pattern to save/recalculate
failures is consistent, visible to the user, and testable (assert on
`state.status` after a forced failure). The production `unwrap()` at
line 2766 (`modifier_editor.take().unwrap()`) is replaced by explicit
`if let Some(editor) = ... take()` handling.

**Alternatives considered**:
- Propagate all errors up to `run()` and exit — rejected: a failed
  save should not kill the session; the user loses the TUI.
- `tracing` log statements only — rejected: invisible to the TUI user
  (and the tracing setup is itself under review, R7).
- Panic — rejected: constitution forbids unwarranted panics in
  library code.

## R7. What to do with the unused tracing setup?

**Decision**: Remove `tracing` and `tracing-subscriber` from
`Cargo.toml` and the init code from `main.rs` (including the `--debug`
flag, which only controls the log filter).

**Rationale**: Zero `debug!/info!/warn!/error!` statements exist in
`src/` (verified by grep). The `--debug` flag's only effect is the
env-filter level; with no log statements, it is a no-op flag.
Constitution Principle I: unused code (and dependencies) must be
removed. If logging is wanted later, it can be re-added with actual
log statements when a real need exists.

**Alternatives considered**: Add log statements at the new error
surfacing sites — rejected: the status line (R6) is the user-visible
mechanism; log files are invisible to a TUI user. Keeping the setup
"for future debugging" — rejected: Principle I.

## R8. What about the per-frame state clone (F4)?

**Decision**: Keep the clone, documented as an accepted trade-off.

**Rationale**: The clone exists because `terminal.draw` requires
`&mut` on the closure state while `App` holds the authoritative
state; the clone lets rendering compute scroll offsets and write them
back after the draw call (`src/ui/app.rs:2002, 2069-2073`). At
realistic scale (hundreds of items), cloning a few hundred small
structs per frame is negligible against a 100 ms poll loop. Removing
it would require restructuring `App`/`AppDrawState` ownership — a
speculative redesign with regression risk, exactly what this
behavior-preserving feature must avoid. FR-012 permits "explicitly
documented as an accepted trade-off".

**Alternatives considered**: Restructure to borrow-based rendering —
rejected as speculative redesign; re-measure only if a real
performance problem appears.

## R9. What about `mockall` (dev-dependency) and the empty `tests/` tree?

**Decision**: Remove `mockall` from `[dev-dependencies]` (zero usage,
verified by grep). Leave the empty `tests/contract|integration|unit`
directories in place — they are scaffolding for the repo's test
convention, not code, and removing them is outside this feature's
scope.

**Alternatives considered**: Remove the empty directories too —
deferred: not a code-quality finding; the user may rely on the
convention. (Noted for the user in the completion report.)

## R10. What is the execution order and why?

**Decision**: The spec's story order is the implementation order:
1. US1 module split (mechanical, gate-green after each move)
2. US2 dispatcher + deduplication (now each change lands in a focused
   module)
3. US3 dead code removal (model shrinks before renames)
4. US4 renames + visibility
5. US5 error surfacing (call sites are now in focused modules)
6. US6 documentation + test adjacency (describes the final structure)

**Rationale**: Each step compounds: the split makes deduplication
visible, deduplication shrinks the code the renames touch, and the
docs must describe the end state. Every step ends gate-green with all
39 tests passing, so partial completion is still a viable codebase.

**Alternatives considered**: Dead-code removal first (smallest diff)
— rejected: the split is the highest-value change and is safest while
the code is still in one file (no cross-module visibility questions);
renames first — rejected: renaming across a 4,917-line file is
error-prone; after the split each rename is module-local.

## R11. Does the split change the public API?

**Decision**: No externally visible change. `src/ui/mod.rs` re-exports
`App` (the only type `src/app.rs` uses: `crate::ui::app::App`). All
other types become `pub(crate)` or private. `main.rs`/`app.rs` are
untouched except where US5 unifies error reporting.

**Rationale**: The binary crate has no external consumers; `pub` on
internal types is noise (finding F6). Tightening visibility is part of
US4 but the split must not *widen* anything.

**Alternatives considered**: Keep everything `pub` for test
convenience — rejected: `#[cfg(test)]` modules and `pub(crate)` cover
test access without leaking.

## R12. How is behavior preservation verified?

**Decision**: Three layers:
1. The 39 existing tests (33 in `src/ui/app.rs`, 6 in
   `src/quantity/calculation.rs`) pass unmodified after every step.
2. The help/hint contract tests (R4b) guard the binding set.
3. A manual smoke pass per split step: launch, navigate every screen
   (Welcome → Main → Help → Config → Modifier editor → Raw editor →
   dialogs), toggle selection/packed, add/edit/delete an item, save,
   reload, quit.

**Rationale**: The tests are the behavioral contract (spec FR-011);
the manual pass covers what unit tests cannot (rendering, terminal
interaction).

**Alternatives considered**: Scripted PTY tests — deferred: the
existing test suite plus manual pass is the project's established
verification level; adding a PTY harness is new infrastructure beyond
this feature's scope.
