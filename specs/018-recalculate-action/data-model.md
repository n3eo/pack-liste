# Data Model: Recalculate Amounts on an Existing List

**Feature**: Recalculate Amounts on an Existing List
**Date**: 2026-09-20
**Spec**: [spec.md](spec.md)

This feature adds one action and one state-layer method; no persisted
entity changes, no file-format changes.

## New code unit

### `AppDrawState::recalculate_quantities`

```rust
impl AppDrawState {
    /// Recalculate every automatic item's quantity from the current
    /// trip parameters and per-item configuration. Fixed items keep
    /// their values; packed, selection, and collapse states are
    /// untouched; nothing is written to disk.
    fn recalculate_quantities(&mut self, calc: &QuantityCalculation) -> Result<()>
}
```

Extracted from `App::update_quantities` (src/ui/app.rs:1907): the
method resolves the style and weather multipliers from
`self.modifiers` (unknown names fall back to 1.0), builds the
`QuantityCalculationInput` (auto items as `ItemQuantitySpec::Auto`,
fixed items as `Fixed(item.quantity)`), runs `calc.calculate`, and
writes the resulting quantities back into
`self.packing_list.items[i].quantity`.

`App::update_quantities` becomes a one-line delegate:

```rust
fn update_quantities(&mut self) -> Result<()> {
    self.draw_state.recalculate_quantities(&self.quantity_calculation)
}
```

## Behavior contract of the action

| Property | Behavior |
|----------|----------|
| Auto items | Quantity := current calculation (weekdays, style, weather, per-item config) |
| Fixed items | Quantity unchanged, always |
| Packed states | Unchanged |
| Selection states | Unchanged |
| Collapse states | Unchanged |
| File writes | None |
| Idempotency | Same inputs → same result, any number of presses |
| Feedback | The redrawn quantities (no dialog/toast) |

## Binding

| Key | Guard | Action |
|-----|-------|--------|
| `R` (or `r`) | `Screen::Main && InputMode::Normal` | `update_quantities()` (the delegate) |

No `focus_side` guard: the action is reachable from both list views
(EC-001). On every other screen/mode the key is unbound (EC-002).

## Hint and help data (single source)

- `hint_entries(HintSurface::LeftPanel)`: gains `("R", "Recalculate")`
- `hint_entries(HintSurface::RightPanel)`: gains `("R", "Recalculate")`
- `help_content` MAIN SCREEN section: gains `entry("R", "Recalculate quantities on the current list")`
- `help_binding_tokens`: gains `"R"`
- `help_screen_text_covers_all_handler_bindings`: gains `"R"` to the handler list
- `hint_entries_match_the_contract_binding_sets`: both list panels' expected sets gain `("R", "Recalculate")`

The `hint_rows` footer reflows automatically; at 80x24 the left
panel's footer grows by one row (verified within the `Min(10)` list
budget).

## State transitions

One: `quantities stale → quantities current`, driven by the user's key
press. No persisted state, no screen, and no mode changes.

## Entities unchanged

`Item`, `PackingList`, `QuantityCalculationInput`,
`QuantityCalculationOutput`, `AppConfig`, `TripFile` — all identical to
today. The action reads existing state and writes existing fields.
