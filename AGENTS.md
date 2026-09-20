# your-project

> Rust project.

Instructions for AI coding agents working in this repository.

## Setup

- Install dependencies: `cargo build`
- Build: `cargo build`
- Run tests: `cargo test`
- Lint / format: `cargo clippy -- -D warnings`

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
