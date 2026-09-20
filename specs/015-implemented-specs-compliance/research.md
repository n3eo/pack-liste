# Research: Implemented-Specs Compliance Review

**Feature**: Implemented-Specs Compliance Review
**Date**: 2026-09-20
**Spec**: [spec.md](../spec.md)

This research resolves the design questions raised by the six gap-closing
user stories, grounded in the current code (verified during the review).

## US1 — Auto/fixed quantity toggle in the edit dialog

### Decision: a fourth dialog field `Mode`, cycled with Tab like the others

**Rationale**: The edit dialog already cycles Name → Category → Quantity
with Tab (`InputField` enum, `src/ui/app.rs:79`). Adding `InputField::Mode`
keeps one interaction idiom; Space or Left/Right toggles the value while
the field is focused (Space is the app's toggle verb; Left/Right is the
adjust verb per the uniform-navigation model of spec 014). The
`InputMode::EditingItem` variant gains an `auto_quantity: bool` carried
from the item; `save_edited_item` writes it back. Toggling to fixed keeps
the current quantity (EC-003); toggling to auto triggers
`update_quantities` after save.

**Alternatives considered**:
- A dedicated global key (e.g., `f` on the list): rejected — the mode is
  per-item edit state, and the dialog is where per-item state is already
  edited; a global key adds a second path to the same change.
- Editing quantity on an auto item implicitly flipping it to fixed:
  rejected — that is exactly the silent behavior US1 removes; the mode
  must be an explicit toggle.

### Side finding to fix in the same story

`save_edited_item` (`src/ui/app.rs:2152`) writes the quantity of auto
items, which the next recalculation overwrites — the "silent overwrite"
from the review. With the mode field, the dialog shows the mode
explicitly, and the quantity edit applies per the mode shown.

## US2 — Category packing indicator

### Decision: derive per category at draw time; marker pair `✓`/`○` in the existing color language

**Rationale**: The compiled list's draw loop
(`draw_selected_items_panel`, `src/ui/app.rs:3317`) already groups items
by category per frame; computing `all_packed = items.iter().all(|i|
i.packed)` per category header is O(items) per frame — negligible. The
marker reuses the per-item symbol vocabulary (`✓` green / `○` red,
`src/ui/app.rs:3225`) so no new visual language is introduced (EC-001).
It is presentation-only: no state is stored.

**Alternatives considered**:
- Storing a per-category packed flag: rejected — derived state that can
  drift; the draw-time derivation cannot be wrong.
- A percentage per category: rejected — spec asks for a binary
  all-packed signal; percentages add noise.

## US3 — Uniform unknown-modifier fallback

### Decision: map unknown activity names to 1.0 at the multiplier-collection site

**Rationale**: `update_quantities` collects activity multipliers with
`filter_map(...activity_multiplier(name))` (`src/ui/app.rs:853-858`),
silently dropping unknowns. Changing `filter_map` to `map(...unwrap_or(1.0))`
matches the travel-style and weather fallback two lines above
(`:847`, `:852`) and spec 005 FR-010. The calculation itself
(`src/quantity/calculation.rs:28-33`) is untouched: a non-empty activity
list keeps the activity path, now including 1.0 entries (EC-005 — the
"activities take priority" note stays truthful).

**Alternatives considered**:
- Fixing it inside `QuantityCalculation`: rejected — the input struct
  already receives resolved multipliers; the resolution site is the
  caller's job and the fix is one line there.
- Warning the user on unknown names: rejected — spec 005 EC-004 defines
  the fallback as silent and deterministic; a warning adds a behavior
  nobody asked for.

## US4 — Complete help screen

### Decision: rebuild the help content as a structured table grouped by context; add a test that extracts bindings from both sides

**Rationale**: The help screen (`draw_help_screen`, `src/ui/app.rs:2630`)
is a hand-maintained `Vec<Line>` that has drifted (7+ missing bindings,
one duplicate). The fix is twofold: (a) rewrite the content grouped by
context (Global, Welcome, Main list, Add/Edit dialog, Config screen,
Config editor, Resume prompt) covering every handler found in the
review; (b) add a test that greps the handler match arms and the help
text for the binding tokens and fails on any mismatch — making drift a
test failure, not a review finding (SC-004).

**Alternatives considered**:
- Generating the help screen from a single binding table at runtime:
  attractive but a larger refactor of every handler; the grouped-list +
  extraction-test approach delivers the 1:1 guarantee with a smaller
  diff. Revisit if bindings churn further.

## US5 — Input-handling test net

### Decision: extract testable state+handlers; drive key events against `AppDrawState`-level logic without a terminal

**Rationale**: The handlers (`handle_up/down`, `toggle_*`,
`set_category_collapsed`, `save_edited_item`, `handle_add_edit_input`)
are methods on `App`, which owns the terminal backend — not
constructible in tests. Two options exist:

1. **Refactor for testability**: move the pure state transitions into
   `AppDrawState` methods (or free functions taking `&mut AppDrawState`),
   leaving `App` as a thin terminal shell. Handlers become one-line
   delegates. Tests construct an `AppDrawState` directly and call the
   transitions — no TTY needed (EC-007).
2. **pty-driven tests only**: rejected as the primary net — slow,
   brittle, and the existing pty scripts are not part of `cargo test`.

Chosen: option 1, applied incrementally to the handlers this feature
touches (navigation, toggles, edit dialog, collapse). The refactor is
behavior-preserving; the existing 11 tests plus the new input tests are
the regression net. Key events are constructed with
`crossterm::event::KeyEvent` (already a dependency) and dispatched
through the extracted functions.

**Alternatives considered**:
- Testing through `App` with a mock backend: ratatui's `Terminal` needs
  a `Write` sink; possible but couples tests to rendering. The state-level
  extraction is simpler and targets exactly the logic under test.

## US6 — Multi-category items

### Decision: `Item.category: String` → `Item.categories: Vec<String>`; one item identity, occurrences are views

**Rationale**: The single-field model (`src/models/item.rs:7`) is the
root of the gap. A `Vec<String>` keeps one `Item` (one selection
`editable`, one `packed`, one quantity) that the tree renders once per
membership — matching the spec's "one underlying item, shown in multiple
places". Grouping sites (`left_rows`, `draw_all_items_panel`,
`draw_selected_items_panel`, `right_item_order`) iterate memberships.
Selection toggles operate on the item, so all occurrences stay
consistent by construction (EC-006). The config format gains
`categories:` per item with a serde default that lifts a legacy
single `category:` value — the additive, non-breaking path (spec 013's
no-compat rule is a separate Draft; until it is implemented, the
additive default keeps current files loading).

**Ripple sites** (all verified in the review):
- `TripFile::reconcile` (`src/config/trip_file.rs:68`): match by name
  against any of the item's categories; remap to the current category
  list.
- `SavedItem` serialization (`src/ui/app.rs:147`): write `categories`,
  read legacy `category` via default.
- Edit dialog: the category dropdown becomes a multi-select over the
  category list (Space toggles membership of the highlighted entry).
- `get_all_categories` / `get_unique_categories`: collect from all
  memberships.

**Alternatives considered**:
- Join-table item↔category with separate rows: rejected — duplicates
  selection/packed state per row and breaks "one item identity".
- Keeping `category` plus an optional `extra_categories`: rejected —
  two sources of truth for the same concept.

## Dependency note

No new dependencies. US5's refactor is the enabling step for testing
US1/US2/US6 flows, so it lands first in the task order. US4's help
update must follow the binding changes of US1 (new Mode field keys) so
the help lists the final binding set once.

## Verification approach

Per story: unit tests at the extracted state layer (US5 net), the
existing calculation tests extended for US3, the extraction test for
US4, and the full quality gate (`cargo check`, `cargo clippy`,
`cargo clippy --tests`, `cargo test`, `cargo fmt --check`) after every
task, per the constitution.
