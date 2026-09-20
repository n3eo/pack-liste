# Verification Contract: Warning-Free Codebase

**Feature**: Warning-Free Codebase via Static Analysis
**Date**: 2026-09-20
**Spec**: [spec.md](../spec.md)

This is the contract this feature establishes: the project's quality gate.
It defines the commands, their required outcomes, and the structural
enforcement that makes the gate automatic.

## The gate

| Command | Required outcome |
|---------|-------------------|
| `cargo check` | Exit 0, zero warnings |
| `cargo clippy` | Exit 0, zero warnings |
| `cargo clippy --tests` | Exit 0, zero warnings |
| `cargo test` | Exit 0, all tests pass (11 tests) |
| `cargo fmt --check` | Exit 0 (formatting clean) |

Scope: the project's own source (`src/`, `tests/`, `Cargo.toml`).
Warnings emitted by third-party dependencies are out of scope.

## Structural enforcement

### Lint table (Cargo.toml)

The project's `Cargo.toml` carries a `[lints]` table that denies warnings by
default, so a plain `cargo check`/`cargo clippy` run fails on any warning
without extra flags:

```toml
[lints.rust]
unsafe_code = "deny"
unused = "deny"
missing_docs = "allow"   # not a project convention; kept at default

[lints.clippy]
all = "warn"             # promoted to deny via the rust `warnings` entry
```

The exact table is finalized during implementation against the toolchain's
supported keys (Rust 1.98); the contract requirement is the outcome — a
plain `cargo clippy` fails on warnings — not the specific table shape. If a
key is unsupported, the equivalent supported configuration is used and
documented here.

### Suppression policy

- Default: fix or remove. Suppression is the exception.
- Any `#[allow(...)]` must be scoped to the smallest item and carry a
  comment stating why it is justified (spec FR-005).
- Target for this feature: zero suppressions.

## Regression proof (SC-005)

After the cleanup, the gate is proven bidirectionally:

1. Introduce a deliberate warning (e.g., an unused variable in `main`).
2. Run `cargo clippy` — it MUST fail (non-zero exit).
3. Remove the warning.
4. Run `cargo clippy` — it MUST pass (exit 0, zero warnings).

## Documentation hook

AGENTS.md's verification section states the gate commands and the
warning-free requirement, so the standard agent/developer workflow runs
them (spec FR-008).
