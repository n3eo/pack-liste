# Contract: Post-Removal Bindings and File Formats

**Feature**: Remove Activity Options
**Date**: 2026-09-20
**Spec**: [spec.md](../spec.md)

This feature removes bindings and fields; it adds none. The contract
records the final binding sets and file formats after the removal, and
amends the spec 016 hint/help contract accordingly.

## Binding changes (amendment to spec 016's contract)

### Welcome screen (after)

| Key | Action |
|-----|--------|
| ↑/↓ | Move focus between fields (file, weekdays, style, weather) |
| ←/→ | Adjust the focused value (weekdays ±1, cycle style/weather) |
| +/- | Adjust weekdays |
| Enter | Edit file name (on file field) / continue to main screen |
| F | Edit the trip file path |
| Esc | Quit |

**Removed**: `Space` (toggle activity) — the activities field is gone;
the `("Space", "Toggle activity")` entry leaves the Welcome hint
surface and the help screen together with the handler.

### Config screen (after)

| Key | Action |
|-----|--------|
| Tab / BackTab | Switch section (travel styles / weather) |
| ↑/↓ | Move the modifier cursor |
| A | Add a modifier |
| E | Edit the modifier under the cursor |
| D | Delete the modifier under the cursor |
| Esc | Close the modifier editor if open; otherwise leave the config screen |

**Removed**: the Activities section from the Tab cycle and the help
text "(travel styles / weather / activities)".

### Unchanged

Main screen, add/edit dialog, delete popup, modifier editor, raw
editor, resume prompt, and all global keys — identical to
`specs/016-harmonized-ui-navigation/contracts/hints-and-bindings.md`.

## Hint/help agreement guarantee (unchanged mechanism)

- `hint_entries(surface)` remains the single data source.
- The extraction tests (`hint_keys_are_help_tokens`,
  `hint_entries_match_the_contract_binding_sets`,
  `help_screen_text_covers_all_handler_bindings`) are updated in the
  same change as the removal; a stale activity entry in hints, help,
  or the token list is a test failure.

## File format contract

### Config file (config.yaml, written)

```yaml
categories: [...]
items:
  - name: ...
    categories: [...]
    auto_quantity: true|false
    quantity: <n>        # only for fixed items
weekdays: <n>
travel_style: <name>
weather: <name>
modifiers:
  travel_styles: [{name, multiplier}, ...]
  weather: [{name, multiplier}, ...]
last_trip_file: <path>   # when set
```

No `selected_activities` key; `modifiers` has no `activities` key.

### Trip file (written)

```yaml
weekdays: <n>
travel_style: <name>
weather: <name>
items:
  - name: ...
    category: ...
    packed: true|false
```

No `selected_activities` key.

### Legacy load tolerance

Files containing `selected_activities` (config or trip) or
`modifiers.activities` (config) MUST load without error; the unknown
keys are ignored. This tolerance is serde's default
unknown-field behavior, not added code, and it is not
version-compatibility machinery: nothing reads the values, and the
next save drops them.

## Calculation contract (revised)

```
quantity(auto) = max(round(min(weekdays, 20) × style_mult × weather_mult), 1)
quantity(fixed) = the stored fixed value
```

The weather multiplier always participates (EC-001). The calculation
notes mention weekdays, style, and weather only.
