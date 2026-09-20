# your-project

> Rust project.

Instructions for AI coding agents working in this repository.

## Setup

- Install dependencies: `cargo build`
- Build: `cargo build`
- Run tests: `cargo test`
- Lint / format: `cargo clippy`

## Quality gate (warning-free requirement)

The codebase MUST stay warning-free. The `[lints]` table in `Cargo.toml`
denies warnings, so a plain run fails on any warning — no extra flags
needed. Before marking any task complete, all of these must pass:

| Command | Required outcome |
|---------|-------------------|
| `cargo check` | Exit 0, zero warnings |
| `cargo clippy` | Exit 0, zero warnings |
| `cargo clippy --tests` | Exit 0, zero warnings |
| `cargo test` | Exit 0, all tests pass |
| `cargo fmt --check` | Exit 0 |

Scope: the project's own source (`src/`, `tests/`, `Cargo.toml`).
Dependency warnings are out of scope. Dead code is removed, never
annotated away; `#[allow(...)]` suppressions are the documented exception,
not the default.

## Code style

- Enforce strict typing across the codebase.
- Run `cargo fmt`; `cargo clippy` must pass with no warnings.
- Prefer `Result` over panics in library code.
- No `unwrap()` in non-test code without a justifying comment.

## Project structure

- `src/` modules; integration tests in `tests/`.

## Guardrails

Things agents get wrong here. Follow these strictly:

- Don't edit `Cargo.lock` by hand.
- Don't introduce `unsafe` without a comment proving its soundness.
- Run the full test suite and fix any failures before marking a task complete.
- Don't add new dependencies without checking the lockfile and existing conventions first.
- Never commit secrets, API keys, or `.env` files.
- Match existing patterns. Don't introduce a new library for something the codebase already solves.

## Commits & pull requests

- Use Conventional Commits: `feat:`, `fix:`, `chore:`, `docs:`, `refactor:`.
- Keep PRs focused on a single concern; describe what changed and why.
