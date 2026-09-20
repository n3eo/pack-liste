# Research: Recalculate Amounts on an Existing List

**Feature**: Recalculate Amounts on an Existing List
**Date**: 2026-09-20
**Spec**: [spec.md](spec.md)

Code-level findings for adding an explicit recalculate action. The
action is a thin, well-supported addition: the calculation path
already exists and is idempotent by construction.

## Verified facts

### F1: `R` is unbound
`grep "Char('r')|Char('R')"` over `src/ui/app.rs` returns nothing. The
main screen's bound letters are C (collapse), A (add), E (edit), D
(delete), plus global q/?/O and Ctrl+E/Ctrl+C/Ctrl+Space. `R` is free
and mnemonic (Recalculate). Decision: bind `R`.

### F2: The calculation path is one method and it is idempotent
`App::update_quantities` (`src/ui/app.rs:1907`) resolves the style and
weather multipliers from the in-memory config, builds
`QuantityCalculationInput` (auto items as `Auto`, fixed items as
``Fixed(item.quantity)``), runs `QuantityCalculation::calculate`, and
writes the resulting quantities back. It touches nothing else — no
packed state, no selection, no collapse, no file I/O. Running it twice
with unchanged inputs produces identical quantities (the calculation
is a pure function of its inputs). The recalculate action is therefore
exactly `update_quantities()` — no new calculation logic (spec
Assumption).

### F3: The handler pattern is established
Main-screen letter handlers follow one shape (see the `C` handler at
~2413): guard `screen == Screen::Main && input_mode == Normal`, then
delegate to a state-layer method. The recalculate handler needs no
`focus_side` guard — the spec requires the action from both list views
(EC-001: "reachable with one key press from both list views"), and
`update_quantities` is side-independent.

### F4: The state layer is the test surface
`AppDrawState` carries everything `update_quantities` reads except the
`QuantityCalculation` instance, which lives on `App`. The existing
state-layer tests (`test_state`/`test_item` helpers, app.rs:243-272)
construct `AppDrawState` directly. For a state-layer test of the
recalculate semantics, the calculation itself is already covered in
`src/quantity/calculation.rs` (auto/fixed/cap/floor tests); what needs
a test here is the *action's* contract: auto items update, fixed items
do not, non-quantity state is untouched. The natural seam: extract the
quantity-writeback into a state-layer method (see D1) so the test can
drive it without a terminal.

### F5: Hint/help/test integration points are fixed
- Hint data: `hint_entries(HintSurface::LeftPanel)` and
  `HintSurface::RightPanel` (app.rs:3598+) — add `("R", "Recalculate")`
  to both.
- Help: the MAIN SCREEN section (app.rs:3823-3835) — add an entry after
  `D`.
- Extraction tests: `hint_entries_match_the_contract_binding_sets`
  (app.rs:743, the both-directions contract pin),
  `hint_keys_are_help_tokens` (728), `help_binding_tokens` (~676),
  `help_screen_text_covers_all_handler_bindings` (~660). All four are
  updated in the same change; the contract pin makes a dropped or
  stale `R` entry a test failure (spec 018 US2/AC3, spec 016's drift
  guard).
- Hint footer width: `hint_rows` packs entries to the panel width; the
  left panel gains one entry (its rows reflow automatically — verified
  behavior from spec 016's implementation; at 80x24 the left footer
  grows from 4 to 5 rows, still within the `Min(10)` list budget).

### F6: No persistence coupling
`save_config`/`save_trip_file` run on exit and on specific actions
(add/edit/delete); `update_quantities` never writes. FR-004 (no file
writes per action) holds by construction. Quantities are session state
today (spec 012 not implemented); US4's persistence is the existing
exit-save flow, unchanged.

### F7: Stale-quantity reality check
Today quantities go stale in two ways the user cannot fix without a
parameter round-trip: (a) the raw config editor's apply flow reloads
config data but the welcome-screen parameter changes are the
recalculation triggers; (b) hand-edited modifier values applied via
Ctrl+S update the multipliers without a parameter change. Verified:
`save_config_editor` (~2515) applies the edited config and calls
`update_quantities` — so (b) is already covered; the remaining gap is
the explicit, user-controlled action the spec requests, plus future
spec 012 resume-without-recalculate.

## Decisions

### D1: Extract the writeback into `AppDrawState::recalculate_quantities`
**Decision**: Move the body of `update_quantities`'s
input-construction + writeback into a state-layer method
`AppDrawState::recalculate_quantities(&mut self, calc:
&QuantityCalculation) -> Result<()>`; `App::update_quantities` becomes
a one-line delegate (matching the spec 015 state-layer pattern). The
`R` handler calls the same delegate.
**Rationale**: The action's contract (auto recalculate, fixed
untouched, non-quantity state untouched) becomes testable at the state
layer without a terminal; the handler stays a one-liner.
**Alternatives**: Call `update_quantities` directly from the handler
and test only via the calculation module (rejected — the action's
state contract deserves its own test; the state-layer seam is the
established pattern).

### D2: Bind `R`, no focus-side guard
**Decision**: `KeyCode::Char('r') | KeyCode::Char('R')` guarded by
`Screen::Main && InputMode::Normal` only. Works from both panels
(EC-001); does nothing on non-list screens (EC-002 — the guard makes
it unbound elsewhere).
**Rationale**: F1/F3; the spec fixes the behavior, the key is the
natural mnemonic.
**Alternatives**: Two keys per panel (rejected — one action, one key).

### D3: No feedback beyond the quantities
**Decision**: No toast, dialog, or status line; the redrawn quantities
are the feedback (EC-005). The handler returns through the existing
input loop, the next `draw()` shows the new amounts.
**Rationale**: The spec's own answer; matches parameter-change
behavior.

### D4: Hints/help/tokens in the same change
**Decision**: Per F5, all four extraction-test surfaces plus the hint
data and the help content gain the `R` entry together; the
both-directions contract pin (`hint_entries_match_the_contract_binding_sets`)
is extended with `("R", "Recalculate")` in both list panels' expected
sets.
**Rationale**: Spec 016's drift-guard contract; US2/AC3.
**Alternatives**: None.

### D5: Tests
**Decision**: One state-layer test module addition:
`recalculate_updates_auto_and_preserves_fixed` — construct a state
with one auto and one fixed item, make quantities stale (set them to
bogus values), run the state-layer recalculate, assert the auto item
matches the current calculation (7 weekdays × 1.0 × 1.0 = 7) and the
fixed item keeps its value; assert packed/selection/collapse state
unchanged; run twice and assert idempotency. The extraction tests
cover the binding's presence (D4).
**Rationale**: SC-001/SC-002/SC-003/SC-005; constitution Principle III.
**Alternatives**: None — the calculation math itself is already
covered.

## Risks

- **R1**: Interaction with spec 017 (remove activities) — both touch
  `update_quantities`'s input construction. Sequencing: whichever
  lands second rebases trivially (017 deletes the activity fields from
  the same literal). The plans are written to not conflict
  structurally: 018 extracts the method, 017 edits its body.
- **R2**: Left-panel hint footer grows by one row at 80x24 (F5) —
  verified acceptable against the `Min(10)` list budget; the live
  render check in the quickstart catches any regression.

## No open questions

EC-001 fixes `R`; every other edge case maps onto existing behavior
verified above.
