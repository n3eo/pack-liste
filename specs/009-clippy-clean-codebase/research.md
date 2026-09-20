# Research: Warning-Free Codebase via Static Analysis

**Feature**: Warning-Free Codebase via Static Analysis
**Date**: 2026-09-20
**Spec**: [spec.md](../spec.md)

This research resolves the open questions for the cleanup by inventorying the
actual warnings on the current codebase (Rust 1.98, clippy 1.98) and deciding
the resolution strategy for each class.

## Inventory (measured 2026-09-20)

`cargo check`: 12 warnings. `cargo clippy`: 37 warnings. `cargo clippy --tests`:
37 warnings (same set; the test build adds none beyond the normal build).

### Dead code (12 warnings, removal per FR-004)

| Item | Location | Origin | Resolution |
|------|----------|--------|------------|
| `PackingService` + 5 methods | src/services/packing_service.rs | Superseded JSON list management (spec 004 alignment) | Remove file |
| `ItemService` + 3 methods | src/services/item_service.rs | Superseded; UI toggles items directly | Remove file |
| `services` module wiring | src/services/mod.rs | Wiring for the above | Remove directory |
| `PackingListConfig` | src/config/model.rs | Superseded config model | Remove file + mod entry |
| `ConfigError` | src/config/error.rs | Never used | Remove file + mod entry |
| `QuantityError` | src/quantity/error.rs | Never used | Remove file + mod entry |
| `UiError` | src/ui/error.rs | Never used | Remove file + mod entry |
| `Item::toggle_packed`, `Item::set_quantity` | src/models/item.rs | UI mutates fields directly | Remove methods |
| `PackingList::{remove_item, get_item_mut, total_items, packed_items, unpacked_items}` | src/models/packing_list.rs | UI iterates items directly | Remove methods |

### Mechanical/idiomatic fixes (25 clippy-only warnings)

| Lint | Count | Locations | Resolution |
|------|-------|-----------|------------|
| `collapsible_if` | 17 | src/ui/app.rs (16), src/config/trip_file.rs (1) | Collapse to `if a && b` |
| `manual_clamp` | 3 | popup draw fns (popup_width/popup_height) | Use `.min(..).max(..)` → `clamp` idiom as clippy suggests |
| `too_many_arguments` | 2 | `draw_add_item_popup`, `draw_edit_item_popup` (8 args each) | Introduce a `PopupFields`-style struct bundling field/name/category/quantity/category_list/category_index; pass one arg |
| `for_kv_map` | 1 | `right_item_order` iterates `&categories_map` | Iterate `categories_map.values()` |
| `while_let_loop` | 1 | `App::run` main loop | Rewrite as `while let AppState::Running = self.state` with the exit arm breaking; keep the post-loop saves |
| `unused_mut` | 1 | src/ui/app.rs:484 | Drop `mut` |
| `needless_return`-style parens | 1 | src/ui/app.rs:3165 | Remove parens around block return |
| `enum_variant_names` (`UiError` all `..Failed`) | 1 | src/ui/error.rs | Resolved by removing the dead enum |

## Decisions

### Decision: Remove dead code rather than annotate

**Rationale**: FR-004 and the repo conventions require removal. All dead code
stems from the superseded packing-list-management design (documented in the
spec 004/005 alignment notes); nothing in the shipped feature set references
it. Git history preserves it.

**Alternatives considered**:
- `#[allow(dead_code)]` annotations: rejected — silences the signal, keeps
  untested code rotting in the tree.
- Wiring services into the UI to make them "used": rejected — adds
  indirection with no user value; the UI already implements the behavior.

### Decision: Fix `too_many_arguments` with a parameter struct, not an allow

**Rationale**: The two popup draw functions take 8 positional arguments of
which 6 travel together (the popup's editing state). Bundling them into a
small struct reduces the signature and removes a real footgun (adjacent
`&str` args are easy to transpose).

**Alternatives considered**:
- `#[allow(clippy::too_many_arguments)]`: rejected — FR-005 makes suppression
  the exception; this case has a clean structural fix.
- Splitting the functions: rejected — the popups are genuinely one widget each.

### Decision: Enforce via `[lints]` table in Cargo.toml, not CLI flags

**Rationale**: FR-007 requires the guard to live in the standard verification
flow. A `[lints]` section (`rust.unsafe_code = "deny"`, plus
`workspace = true`-style warning denial via `warnings` where supported) makes
every plain `cargo clippy` run fail on warnings without remembering flags.
AGENTS.md already documents `cargo clippy -- -D warnings`; the lint table
makes that the default rather than an opt-in.

**Alternatives considered**:
- Documenting `cargo clippy -- -D warnings` only: rejected — relies on humans
  remembering the flag; the spec requires the gate to be structural.
- A CI pipeline: out of scope for this repository (no CI configured).

### Decision: Iterate fix waves until a clean pass

**Rationale**: Removing dead modules can expose new unused imports (e.g.,
`serde_json` in Cargo.toml after `PackingService` goes, `chrono` if
`PackingList::new`'s timestamp id becomes unused). The plan is to re-run
`cargo check && cargo clippy` after each wave and only stop on a fully clean
pass (spec edge case: cascading warnings).

**Alternatives considered**: Single-pass fix: rejected — the cascade risk is
known and measured.

## Dependency note

Removing `PackingService` (the only `serde_json` user) and possibly
`PackingList::new`'s chrono usage may leave unused dependencies in
Cargo.toml. Per repo guardrails, Cargo.lock is not hand-edited; `cargo remove`
(or dropping the Cargo.toml entry and rebuilding) is the clean path. This is
in scope: an unused dependency is dead code at the manifest level.

## Verification approach

After each wave: `cargo check`, `cargo clippy`, `cargo clippy --tests`,
`cargo test` (11 tests must pass). Final gate: introduce a deliberate unused
variable, confirm the lint run fails, remove it, confirm it passes (SC-005).
