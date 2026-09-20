# Quickstart Validation Guide: Warning-Free Codebase

**Feature**: Warning-Free Codebase via Static Analysis
**Spec**: [spec.md](spec.md) | **Contract**: [contracts/verification.md](contracts/verification.md)

Runnable scenarios that prove the feature end-to-end: a warning-free
codebase and a gate that keeps it that way.

## Prerequisites

- Rust 1.98+ (2024 edition) with clippy and rustfmt components
- The pack-liste repository checked out

## Scenario 1: Clean static analysis (SC-001, SC-002)

```bash
cd pack-liste
cargo check
cargo clippy
cargo clippy --tests
```

**Expected**: All three commands exit 0 and print no warnings for the
project's own code. (First run compiles dependencies; their warnings, if
any, are out of scope.)

## Scenario 2: Behavior unchanged (SC-004)

```bash
cargo test
cargo build
./target/debug/pack-liste --help
```

**Expected**: All 11 tests pass. The binary starts and shows the same CLI
help as before the cleanup (same flags, same version).

## Scenario 3: The gate fails on a new warning (SC-005)

```bash
# Introduce a deliberate warning
echo 'fn _unused_probe() {}' >> src/main.rs
cargo clippy; echo "exit: $?"
```

**Expected**: `cargo clippy` fails (non-zero exit) with a dead-code warning
for `_unused_probe`.

```bash
# Remove it and confirm the gate passes again
# (delete the appended line)
cargo clippy; echo "exit: $?"
```

**Expected**: Exit 0, zero warnings.

## Scenario 4: Formatting clean

```bash
cargo fmt --check
```

**Expected**: Exit 0. (Run `cargo fmt` first if the cleanup touched
formatting.)

## Scenario 5: Dead code is gone, not annotated

```bash
grep -rn "PackingService\|ItemService\|PackingListConfig\|UiError\|QuantityError\|ConfigError" src/
grep -rn "#\[allow(" src/
```

**Expected**: No matches for the removed items in `src/`. The `#[allow(`
search returns no unjustified suppressions (target: none at all).

## Scenario 6: Application still works end to end

```bash
./target/debug/pack-liste --force-tui
```

**Expected**: The TUI starts, shows the welcome screen, and quits with q —
saving config and trip file as before. (Manual smoke test; the automated
equivalent is the test suite plus the pty-driven checks used during
development.)
