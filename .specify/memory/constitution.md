# Pack-Liste Constitution

## Core Principles

### I. No Unused Code

All code in the repository MUST be reachable from the application's behavior
or its tests. Dead code — unused functions, methods, structs, enums, modules,
or dependencies — MUST be removed rather than annotated away. Lint
suppressions (`#[allow(...)]`) MUST NOT be used to silence unused-code
warnings; the only exception is a suppression scoped to the smallest possible
item with a comment stating why it is justified. Code kept "for future
features" is still unused code: it lives in version control history and is
re-added when the feature is actually built.

Rationale: unused code rots silently, misleads readers, dilutes the warning
signal, and hides real defects behind noise.

### II. Zero Warnings and Errors

`cargo check` and `cargo clippy` MUST complete with zero warnings and zero
errors on the project's own source, in the normal build and the test build.
The project's lint configuration MUST treat warnings as errors in the
standard verification flow, so a warning fails the gate instead of
accumulating. Warnings originating in third-party dependencies are out of
scope; the project's own source is what must be clean. Linter suggestions
MUST be fixed at the cause where reasonably possible; suppression is the
documented exception, never the default.

Rationale: a warning-free build keeps the signal meaningful — when the tools
are silent, new findings are real findings, and reviewers spend attention on
defects instead of noise.

### III. Documented and Tested

Every feature MUST be properly documented and properly tested.

Documentation: public modules, types, and functions MUST carry doc comments
stating purpose and behavior; user-facing behavior MUST be described in the
project docs (README, TUI_USAGE, specs) and updated in the same change that
alters the behavior.

Testing: every feature MUST ship with tests covering its core behavior; the
full test suite MUST pass before any task is considered complete. Bug fixes
MUST come with a test that fails before the fix and passes after it.

Rationale: undocumented, untested code cannot be safely changed by anyone —
including its author, later.

## Quality Standards

The project's verification commands and their required outcomes:

| Command | Required outcome |
|---------|-------------------|
| `cargo check` | Exit 0, zero warnings |
| `cargo clippy` | Exit 0, zero warnings |
| `cargo clippy --tests` | Exit 0, zero warnings |
| `cargo test` | Exit 0, all tests pass |
| `cargo fmt --check` | Exit 0 |

Scope: the project's own source (`src/`, `tests/`, `Cargo.toml`).
Dependency warnings are out of scope. No `unsafe` code without a comment
proving its soundness. No new dependencies without checking the lockfile and
existing conventions first. `Cargo.lock` is never hand-edited.

## Development Workflow

Quality gates, in order:

1. **Static analysis**: `cargo check` and `cargo clippy` pass with zero
   warnings (Principle II) before a task is marked complete.
2. **Tests**: the full suite passes; new behavior arrives with new tests
   (Principle III).
3. **Dead-code sweep**: any code the compiler or linter reports as unused is
   removed in the same change that orphaned it (Principle I).
4. **Docs**: behavior changes ship with matching documentation updates
   (Principle III).
5. **Review**: changes are reviewed against this constitution; complexity
   beyond the simplest sufficient solution must be justified in the change
   description.

Commits follow Conventional Commits (`feat:`, `fix:`, `chore:`, `docs:`,
`refactor:`); PRs stay focused on a single concern.

## Governance

This constitution supersedes conflicting ad-hoc practices in the repository.
Amendments require: a documented change proposal, a semantic-version bump of
this file (MAJOR for principle removal/redefinition, MINOR for additions or
material expansion, PATCH for clarifications), a Sync Impact Report for human
review, and a compliance check of active work against the amended text.
Every review and every planning workflow (`/speckit-plan` Constitution Check)
verifies compliance with these principles; violations block progression until
fixed or explicitly justified in the plan's Complexity Tracking table.

**Version**: 1.0.0 | **Ratified**: 2026-09-20 | **Last Amended**: 2026-09-20
