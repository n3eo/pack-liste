# Quickstart Validation Guide: Implemented-Specs Compliance Review

**Feature**: Implemented-Specs Compliance Review
**Spec**: [spec.md](spec.md) | **Contract**: [contracts/input-and-help.md](contracts/input-and-help.md)

Runnable scenarios that prove the six gap-closing stories work
end-to-end. Prerequisites: Rust 1.98+, the pack-liste repository, a
terminal; the quality gate must be clean before starting.

## Scenario 1: Toggle auto/fixed quantity in the TUI (US1, SC-001)

```bash
cargo run -- --force-tui <trip-file>
```

1. On the main screen, move to an item and press `E` (edit dialog).
2. Press Tab until the **Mode** field is focused; verify it shows the
   current mode (automatic).
3. Press Space (mode flips to fixed), then Enter to save.
4. Change the trip length on the welcome screen; return to the list.

**Expected**: the item's quantity did not change with the trip length
(fixed mode took effect). Repeat the edit, toggle the mode back to
automatic, save: the quantity recalculates immediately.

## Scenario 2: Category packing indicator (US2, SC-002)

1. On the compiled list (right panel), pack every item of one category
   (Space on each), leave another category partially unpacked.
2. Read the two category headers.

**Expected**: the fully packed category's header shows the all-packed
marker (`✓`, green); the partially packed one shows the open marker
(`○`, red). Toggling the last unpacked item flips its header within one
render. The left panel's headers are unchanged.

## Scenario 3: Unknown modifier fallback (US3, SC-003)

```bash
cargo test quantity
```

**Expected**: the calculation tests include the unknown-activity case:
a selected activity with no configured multiplier contributes 1.0 to
the activity average (the activity path stays in effect), matching the
travel-style and weather fallback. The existing known-activity tests
are unchanged and pass.

## Scenario 4: Help screen completeness (US4, SC-004)

```bash
cargo test help
```

**Expected**: the extraction test passes — every handler binding has a
help entry, every help entry has a handler, no duplicates. Manually:
press `?` on the main screen and verify the groups from the contract
(Global, Welcome, Main, Dialog, Config, Editor) are all present,
including Ctrl+C, F, BackTab, Home/End, and the modifier editor keys.

## Scenario 5: Input-handling test net (US5, SC-005)

```bash
cargo test
```

**Expected**: the suite includes input-level tests driving key events
against the extracted state handlers (navigation, toggles, collapse,
edit-dialog save). Mutation check: temporarily break one navigation
binding, run the suite, verify a test fails, restore the binding.

## Scenario 6: Multi-category items (US6, SC-006)

1. Edit an item; in the category field, select two categories (Space
   toggles membership); save.
2. View the left tree: the item appears under both headers.
3. Deselect it under one header (Space).

**Expected**: the item is deselected — gone from the compiled list and
shown deselected under both headers (one shared selection state). The
compiled list shows the item at most once. A trip file saved with the
item resumes correctly after the catalog's categories changed
(reconcile matches by name across memberships).

## Final gate

```bash
cargo check && cargo clippy && cargo clippy --tests && cargo test && cargo fmt --check
```

**Expected**: all clean, zero warnings (constitution Principle II);
all tests pass including the new input, help-extraction, and
calculation-fallback tests.
