# Data Model: Warning-Free Codebase via Static Analysis

**Feature**: Warning-Free Codebase via Static Analysis
**Date**: 2026-09-20
**Spec**: [spec.md](../spec.md)

This feature is behavior-preserving: no persisted data changes, no new
storage, no format changes. This document records the code-level model
changes (removals and one addition) so the implementation phase has an
exact target.

## Removals (dead code)

| Item | Kind | Reason |
|------|------|--------|
| `src/services/` (module, `PackingService`, `ItemService`) | Module + 2 structs + 8 methods | Superseded by trip-file flow (spec 004); never constructed |
| `src/config/model.rs` (`PackingListConfig`) | Struct + Default impl | Superseded config model; never constructed |
| `src/config/error.rs` (`ConfigError`) | Error enum | Never used |
| `src/quantity/error.rs` (`QuantityError`) | Error enum | Never used |
| `src/ui/error.rs` (`UiError`) | Error enum | Never used |
| `Item::toggle_packed`, `Item::set_quantity` | Methods | UI mutates the fields directly |
| `PackingList::{remove_item, get_item_mut, total_items, packed_items, unpacked_items}` | Methods | UI iterates `items` directly |

**Surviving model surface** (what remains in use):

- `Item`: `id`, `name`, `category`, `base_quantity`, `quantity`, `packed`,
  `editable`, `auto_quantity`; constructor `Item::new`.
- `PackingList`: `id`, `name`, `items`, `file_path`; `PackingList::new`,
  `add_item`.
- `Weekdays`: validated 1–30, `new`, `get`, `Default`.
- `TripFile`/`TripItem`, `AppConfig`/`SavedItem`, `ModifiersConfig`/
  `ModifierConfig`, `QuantityCalculationInput`/`Output`/`ItemQuantitySpec`:
  unchanged.

## Addition

### PopupFields (new, internal to `src/ui/app.rs`)

Bundles the editing state shared by the add-item and edit-item popups so the
draw functions take one argument instead of eight (resolves
`clippy::too_many_arguments`).

**Fields**:
- `field: InputField` — which input field is focused
- `name: String` — current name input
- `category: String` — current category input
- `quantity: String` — current quantity input
- `category_list: Vec<String>` — selectable categories
- `category_index: usize` — selected category in the list

**Construction**: from the `InputMode::AddingItem`/`EditingItem` variants at
the draw call site; the `InputMode` variants themselves are unchanged (the
input handling code keeps operating on them directly).

**Validation rules**: none (a view-state bundle; invariants are those of
`InputMode` today, e.g. `category_index < category_list.len()` guarded at
use sites as now).

**State transitions**: none (created per frame from `InputMode`, not stored).

## Manifest-level changes

- `Cargo.toml`: add `[lints]` table (see contracts/verification.md).
- `Cargo.toml`: remove `serde_json` if, after the removals, no user remains
  (expected: `PackingService` was the only one). `chrono` stays only if
  `PackingList::new`'s timestamp id remains; otherwise remove. Verify with a
  clean build after removal; never hand-edit `Cargo.lock` (repo guardrail).

## State transitions

None. No persisted state, no runtime state machine changes. The application's
behavioral state (screens, input modes, trip data) is untouched.
