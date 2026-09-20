# Quickstart Validation Guide: Recalculate Amounts on an Existing List

**Feature**: Recalculate Amounts on an Existing List
**Spec**: [spec.md](spec.md) | **Contract**: [contracts/recalculate-binding.md](contracts/recalculate-binding.md)

Runnable scenarios proving the action end-to-end. Prerequisites: the
quality gate clean before starting; a terminal for the TUI checks.

## Scenario 1: Recalculate a stale list (US1, US3, FR-001/FR-002, SC-001)

```bash
cargo test recalculate
```

**Expected**: the state-layer test passes — with one auto and one
fixed item whose quantities were made stale, the recalculate updates
the auto item to the current calculation (7 weekdays × 1.0 × 1.0 = 7)
and leaves the fixed item untouched; packed/selection/collapse state
is unchanged; a second run produces identical quantities.

Then in the TUI (`cargo run -- --force-tui <trip-file>`):

1. Mark one item fixed with a custom quantity (E → Mode → toggle →
   set quantity → save).
2. Change weekdays on the welcome screen, continue, and note the
   auto items' amounts.
3. Press `R` on the left panel; press `R` again on the right panel.

**Expected**: auto items show the freshly calculated amounts from
either panel; the fixed item never changes; pressing `R` twice changes
nothing the second time.

## Scenario 2: The binding is discoverable (US2, FR-005, SC-004)

```bash
cargo test hint
```

**Expected**: the extraction tests pass with `("R", "Recalculate")` in
both list panels' hint sets and `"R"` in the token/handler lists.
Break the guard deliberately: remove the `R` entry from
`hint_entries` and verify the contract-pin test fails, then revert.

Then in the TUI: read both panels' hint lines (R Recalculate appears in
both) and the help screen's MAIN SCREEN section (R documented after D).

## Scenario 3: No side effects (FR-003/FR-004, SC-003)

In the TUI: pack an item, collapse a category, deselect another item;
press `R`; verify packed, collapse, and selection states are all
unchanged and no file was written (check the trip file's mtime).

## Scenario 4: Hint footer still fits at the minimum size (spec 016 carry-over)

Open the main screen at 80x24; verify both panels' hint footers show
every entry including R Recalculate, with no overlap with list content
or the stats block (the left footer grows by one row and reflows).

## Final gate

```bash
cargo check && cargo clippy && cargo clippy --tests && cargo test && cargo fmt --check
```

**Expected**: all clean, zero warnings; the full suite passes including
the new recalculate test and the extended extraction tests.
