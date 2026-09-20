# Quickstart Validation Guide: Remove Activity Options

**Feature**: Remove Activity Options
**Spec**: [spec.md](spec.md) | **Contract**: [contracts/bindings-and-formats.md](contracts/bindings-and-formats.md)

Runnable scenarios proving the removal end-to-end. Prerequisites: the
quality gate clean before starting; a terminal for the TUI checks.

## Scenario 1: No activity surface anywhere (US1, FR-001, SC-001)

```bash
cargo run -- --force-tui <trip-file>
```

1. On the welcome screen, press ↑/↓ repeatedly and watch the focus
   cycle: file → weekdays → style → weather → file. No activities
   field exists.
2. Press `O` (config screen); press Tab repeatedly: the section cycle
   is travel styles ↔ weather. No activities section exists.
3. Press `?` (help): no activity binding or description appears.

**Expected**: exactly four welcome fields, two config sections, zero
activity references on any screen, hint, or help entry.

## Scenario 2: The calculation uses weather directly (US2, FR-002, SC-002)

```bash
cargo test quantity
```

**Expected**: the direct-rule tests pass (e.g. 7 weekdays × 1.0 style ×
1.5 weather = 11); the activity-priority tests are gone. Then in the
TUI: set weekdays 7, style standard, weather cold; every auto item
shows 11. No input can suppress the weather multiplier.

## Scenario 3: Legacy files load; new saves are clean (US3, FR-003/FR-004, SC-003/SC-004)

```bash
cargo test
```

**Expected**: the legacy-tolerance tests pass — a config YAML
containing `selected_activities` and `modifiers.activities`, and a
trip YAML containing `selected_activities`, both parse without error;
a serialized config/trip contains no activity keys. Manually: load a
pre-feature config, exit (which saves), and inspect the file — the
activity fields are gone (EC-005).

## Scenario 4: The drift guard holds (FR-005, SC-001)

```bash
cargo test hint
```

**Expected**: the extraction tests pass with the activity-free binding
sets. Break the guard deliberately: re-add an activity entry to
`hint_entries` or the help content and verify the test fails, then
revert.

## Final gate

```bash
cargo check && cargo clippy && cargo clippy --tests && cargo test && cargo fmt --check
```

**Expected**: all clean, zero warnings; the full suite passes with the
revised test set.
