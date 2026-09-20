# Contract: Module Structure and Invariants

**Feature**: Code Review Remediation | **Date**: 2026-09-20 | **Spec**: [spec.md](../spec.md)

This feature has no external interfaces to users or other systems: it is
behavior-preserving internal refactoring of a single-binary TUI. The
contracts that matter are (a) the module structure the refactor must
produce and (b) the invariants that must hold across it. The existing
input/help contract (`specs/016-harmonized-ui-navigation/contracts/hints-and-bindings.md`)
remains in force, unchanged.

## Module structure contract (US1)

After the refactor, `src/ui/` MUST have this shape (research R3):

```text
src/ui/
├── mod.rs          # module declarations; re-exports App
├── state.rs        # AppDrawState, Screen, Focus, FocusSide, InputField,
│                   # InputMode, AppState, StatusMessage, cursor ops
├── input.rs        # key dispatcher: global keys, then screen routing
├── handlers/       # one module per screen/mode family
│   ├── welcome.rs  # welcome screen + resume prompt
│   ├── main.rs     # main screen, normal mode
│   ├── dialogs.rs  # add/edit/delete popups
│   ├── config.rs   # config screen + modifier editor
│   └── editor.rs   # raw config editor
├── draw.rs         # App::draw dispatch + shared draw helpers
├── screens/        # one module per screen's rendering
│   ├── welcome.rs
│   ├── main.rs
│   ├── help.rs
│   ├── config.rs
│   └── editor.rs
├── hints.rs        # hint entries, help tokens, hint rendering
├── help.rs         # help screen content
├── config_io.rs    # AppConfig, SavedItem, load/save/default config
└── tests/          # tests adjacent to the code they test
```

Rules:

- `src/ui/mod.rs` re-exports only what `src/app.rs` consumes (`App`).
  All other types are `pub(crate)` or private (US4).
- No source file exceeds 1,500 lines (SC-001); no input handler
  exceeds 150 lines (SC-002).
- `handlers/` and `screens/` mirror each other by name: the handler
  for a screen lives in `handlers/<screen>.rs`, its rendering in
  `screens/<screen>.rs`.
- Tests live in `src/ui/tests/` (or `#[cfg(test)]` modules within the
  file under test), never in a monolithic test module before the
  production code (FR-010).

## Behavior invariants (all stories)

These MUST hold after every step, verified by the existing suite:

1. **Binding contract**: the spec 016 binding set
   (`hints-and-bindings.md`) is unchanged; the help/hint tests
   (`help_lists_every_binding_token`,
   `help_screen_text_covers_all_handler_bindings`,
   `hint_keys_are_help_tokens`,
   `hint_entries_match_the_contract_binding_sets`,
   `hint_lines_use_the_unified_style`,
   `hint_rows_fit_width_and_keep_every_entry`) pass unmodified.
2. **Test preservation**: all 39 existing tests pass without
   modification to their assertions (test *location* moves; test
   *content* does not).
3. **Format preservation**: a `config.yaml` and trip file saved by
   the pre-refactor binary load identically post-refactor
   (data-model.md V1).
4. **Gate**: `cargo check`, `cargo clippy`, `cargo clippy --tests`,
   `cargo test`, `cargo fmt --check` all exit 0 with zero warnings
   after every step.
5. **Selection default**: on a new/empty list all items are
   pre-selected (V2) — the rename must not alter this.
6. **Error visibility**: after US5, a forced save/recalculate failure
   is visible to the user (status line) or propagated — never
   silently discarded (FR-006).

## Dependency contract (US3/US5)

`Cargo.toml` after the refactor MUST NOT list `chrono`, `tracing`,
`tracing-subscriber`, or `mockall`. `Cargo.lock` is updated only by
`cargo` (never hand-edited). No new dependencies are added by this
feature.
