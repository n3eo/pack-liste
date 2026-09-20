# Data Model: Code Review Remediation

**Feature**: 022-code-review-refactoring | **Date**: 2026-09-20 | **Spec**: [spec.md](spec.md)

This feature changes no persisted data. The data model below documents
the two persisted formats (unchanged) and the in-memory model changes
(US3 removals, US4 rename, US5 status line) so tasks can verify
format-neutrality.

## Persisted formats (UNCHANGED by this feature)

### Input config file (`config.yaml`)

Serialized via `AppConfig`/`SavedItem` (moves to `src/ui/config_io.rs`
in US1; schema identical).

| Field | Type | Notes |
|-------|------|-------|
| `categories` | `Vec<String>` | Known categories |
| `items` | `Vec<SavedItem>` | Item definitions |
| `weekdays` | `u32` (default 0) | Trip length |
| `travel_style` | `String` (default "standard") | |
| `weather` | `String` (default "mixed") | |
| `modifiers` | `ModifiersConfig` (default) | Travel/weather/activity multipliers |
| `last_trip_file` | `Option<String>` | Resume hint, omitted when none |

`SavedItem`: `name: String`, `categories: Vec<String>` (a legacy
single `category:` string is lifted to a one-element list on load),
`quantity: Option<u32>` (only for fixed-quantity items),
`auto_quantity: bool`.

### Trip file (human-editable, reconciled on load)

Serialized via `TripFile`/`TripItem` (`src/config/trip_file.rs`;
unchanged).

| Field | Type | Notes |
|-------|------|-------|
| `weekdays` | `u32` | Trip parameters at last save |
| `travel_style` | `String` | |
| `weather` | `String` | |
| `items` | `Vec<TripItem>` (default empty) | Selected items with packed state |

`TripItem`: `name: String`, `category: String`, `packed: bool`
(default false). Quantities are never persisted; recalculated on load.

**Verification rule for US3/US4**: neither format contains `id`,
`base_quantity`, `editable`/`selected`, or list `name`/`file_path`
(research R1, R2). A config and trip file saved before the refactor
must load identically after it.

## In-memory model changes

### `Item` (`src/models/item.rs`)

| Field | Change | Story |
|-------|--------|-------|
| `id: String` | **Removed** — written, never read | US3 |
| `name: String` | Kept | — |
| `categories: Vec<String>` | Kept | — |
| `base_quantity: u32` | **Removed** — written, never read | US3 |
| `quantity: u32` | Kept | — |
| `packed: bool` | Kept | — |
| `editable: bool` | **Renamed** `selected: bool` — means "selected for the trip" | US4 |
| `auto_quantity: bool` | Kept | — |

`Item::new` loses its `id` and `base_quantity` parameters; the
constructor becomes `Item::new(name, categories, quantity)`. All call
sites (`save_new_item`, `load_config`, `create_default_config`, test
helpers) updated.

### `PackingList` (`src/models/packing_list.rs`)

| Field | Change | Story |
|-------|--------|-------|
| `id: String` | **Removed** (chrono dependency removed with it) | US3 |
| `name: String` | **Removed** | US3 |
| `items: Vec<Item>` | Kept | — |
| `file_path: Option<PathBuf>` | **Removed** | US3 |

`PackingList::new` becomes parameterless (`PackingList::new()` → or
`PackingList::default()`); the trip file path already lives in
`AppDrawState.trip_file_path`.

### `AppDrawState` (`src/ui/state.rs` after US1)

| Field | Change | Story |
|-------|--------|-------|
| `status: Option<StatusMessage>` | **Added** — user-visible error/info line | US5 |
| all others | Moved unchanged from `src/ui/app.rs` | US1 |

New type:

```text
StatusMessage { message: String, kind: StatusKind }
StatusKind { Error, Info }
```

Set by save/recalculate failure sites (replacing `.ok()`); rendered in
the footer of the active screen; cleared on the next successful
operation or keypress that changes mode.

### `HandlerOutcome` (`src/ui/input.rs`, new)

```text
HandlerOutcome { Handled, Exit }
```

Replaces the `Result<bool>` return of `handle_input` and per-screen
handlers (research R4). `Exit` propagates to the run loop; `Handled`
continues it.

## Validation rules

- **V1**: A `config.yaml` and trip file written before this feature
  load without error and produce identical in-memory state after it
  (no `id`, `base_quantity`, or `editable` keys are required by either
  format).
- **V2**: `Item.selected` (formerly `editable`) defaults to `true` for
  all items on a new/empty list (spec 015 behavior, preserved).
- **V3**: The status line shows the most recent unsurfaced error and
  does not persist across a mode change (cleared on transition). It is
  rendered on every screen that can raise a save/recalculate failure:
  the main screen and the config screen (Phase 9 refinement of R6).
- **V4**: Removing `chrono`, `tracing`, `tracing-subscriber`, and
  `mockall` leaves `cargo build` and `cargo test` green; `Cargo.lock`
  is updated only via `cargo` itself.
