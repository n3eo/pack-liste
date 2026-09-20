# Implementation Plan: Per-Item Quantity Configuration

**Branch**: `010-item-quantity-config` | **Date**: 2026-09-20 | **Spec**: [spec.md](spec.md)

## Summary

Give every item its own quantity modifiers: a **per-day rate**
(0.05–10.0, default 1.0) and exactly one **buffer** — either a flat
**reserve** (0–10, added after the rate-based amount) or a **buffer
multiplier** (1.0–3.0, scaling the rate-based amount), never both. The
calculation rule becomes per item: `rate amount = min(weekdays, 20) ×
style × weather × rate`, then `+ reserve` or `× multiplier`, rounded
and clamped to 1–99. Items without modifiers keep the exact current
behavior (rate 1.0, no buffer). The modifiers are edited in the
existing add/edit dialog and persisted with the item in the config
file.

**Sequencing note**: This plan targets the post-022 module tree. It
must land **before** spec 013 (remove compat code): the new persisted
fields use serde defaults for pre-feature files (FR-008), and 013
removes that machinery — 013 then re-baselines the format. It also
composes with spec 018 (amended): once 018 lands, a modifier change
does not auto-recalculate; the user presses a recalculate key. If 018
has not landed yet, the FR-009 "recalculate immediately" behavior is
the current auto-trigger path; the plan notes both.

## Technical Context

**Language/Version**: Rust 1.98 (2024 edition)

**Primary Dependencies**: ratatui 0.30, crossterm 0.29, serde/serde_yaml
(unchanged)

**Storage**: The input config (`config.yaml`) — `SavedItem` gains
optional `rate: f32` (default 1.0) and a buffer field: either
`reserve: u32` (default 0) or `buffer_multiplier: f32` (default 1.0 =
none). Exactly one buffer kind is persisted per item (FR-011): a
reserve item writes `reserve` and omits the multiplier; a
multiplier item writes `buffer_multiplier` and omits the reserve.
Pre-feature files load with the defaults (FR-008).

**Testing**: `cargo test` — 42 existing tests (the contract); new
unit tests in `src/quantity/calculation.rs` cover the per-item rule
(rate, reserve, multiplier, exclusivity, clamps, defaults-identical);
state-layer tests cover the dialog's exclusivity enforcement; config
round-trip tests cover persistence.

**Target Platform**: Terminal TUI (Linux)

**Project Type**: CLI/TUI application (single binary)

**Performance Goals**: Unchanged — the calculation is O(items) as
before.

**Constraints**: Warning-free gate (constitution Principle II);
defaults must reproduce current behavior exactly (SC-002, SC-005);
the dialog reuses the existing field-cycling model (spec 016/020
bindings); no new dependencies.

**Scale/Scope**: `Item`/`SavedItem` gain fields; the calculation
input gains per-item specs; the add/edit dialog gains two fields
(Rate, Buffer); hints/help/extraction tests updated; ~10 new tests.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **Principle I (No Unused Code)**: satisfied — every new field is
  read by the calculation and writable from the dialog.
- **Principle II (Zero Warnings and Errors)**: satisfied — gate clean
  per task.
- **Principle III (Documented and Tested)**: satisfied — the rule is
  documented in `docs/quantity-calculation.md` in the same change;
  unit + state + round-trip tests ship with the feature.
- No violations. Proceeding.

## Project Structure

### Documentation (this feature)

```text
specs/010-item-quantity-config/
├── plan.md              # This file
├── research.md          # Phase 0 (below)
├── data-model.md        # Phase 1 (below)
├── quickstart.md        # Phase 1 (below)
├── contracts/           # Phase 1 (below)
└── tasks.md             # Phase 2 (/speckit-tasks)
```

### Source Code (repository root, post-022 module tree)

```text
src/
├── models/
│   └── item.rs          # Item gains rate + buffer (an enum: None |
│                        #   Reserve(u32) | Multiplier(f32))
├── quantity/
│   ├── model.rs         # ItemQuantitySpec::Auto gains the item's
│   │                    #   rate + buffer (or a new Auto { rate, buffer })
│   │                    #   variant; Fixed unchanged
│   └── calculation.rs   # the per-item rule; clamps 1–99
├── ui/
│   ├── config_io.rs     # SavedItem gains rate + buffer; serde defaults
│   ├── state.rs         # InputMode::AddingItem/EditingItem gain the
│   │                    #   rate/buffer dialog fields; apply_item_edit
│   │                    #   writes them; exclusivity enforcement
│   ├── handlers/
│   │   └── dialogs.rs   # dialog save carries the new fields
│   ├── screens/
│   │   └── (draw.rs)    # the add/edit dialog renders the new fields
│   ├── hints.rs / help.rs   # new dialog bindings
│   └── tests/           # new tests
└── docs/
    └── quantity-calculation.md  # the per-item rule documented
```

**Structure Decision**: The buffer is a Rust enum on `Item`
(`QuantityBuffer { None, Reserve(u32), Multiplier(f32) })` — the
exclusivity is a type guarantee, not a runtime check. The calculation
input carries the per-item spec; the dialog gains two fields after
Mode (Rate, Buffer), cycled by the existing field-cycling model.

## Phase 0 Research

- **R1 — Where the rate lives**: `Item` gains `rate: f32` (default
  1.0). The calculation's `ItemQuantitySpec::Auto` becomes
  `Auto { rate: f32, buffer: QuantityBuffer }` so the rule is
  per item; `Fixed(u32)` is unchanged. Verified: `src/quantity/model.rs`
  currently has a unit `Auto` variant — the change is additive.
- **R2 — The buffer as an enum**: `QuantityBuffer` with the three
  variants; `None` is the default. serde serializes `None` as absent
  (skip_serializing_if), `Reserve(n)` as `reserve: n`, `Multiplier(x)`
  as `buffer_multiplier: x` — one field on disk per item, exclusivity
  structurally guaranteed (SC-007).
- **R3 — The rule (from the spec's examples)**: rate amount =
  `round(min(weekdays,20) × style × weather × rate)`; then `+ reserve`
  or `× multiplier`; clamp 1–99. The existing minimum-1 clamp (spec
  005) applies to the rate amount before the reserve is added (EC-002);
  the cap is 99 (EC-003). Trip multipliers touch only the rate-based
  part (EC-006, FR-010).
- **R4 — Defaults identical**: rate 1.0, buffer None reproduces
  `round(min(weekdays,20) × style × weather)` — the exact current
  rule. The existing calculation tests must pass unchanged (SC-005);
  new tests pin the default equivalence.
- **R5 — Dialog integration**: the add/edit dialog (post-020 it has
  Name/Category/Quantity/Mode with Up/Down cycling) gains Rate and
  Buffer fields. Buffer editing: Left/Right steps the value; a
  dedicated key or the Mode-field pattern toggles the kind
  (None → Reserve → Multiplier); setting a kind clears the other's
  value (FR-011). Exact bindings are a Phase 1 contract decision.
- **R6 — Editor validation**: rate 0.05–10.0, reserve 0–10, multiplier
  1.0–3.0 (FR-006); the dialog rejects out-of-range input (keeps the
  previous valid value).
- **R7 — Recalculate trigger**: FR-009 says recalculate immediately
  on modifier change. Under the amended spec 018 model (manual
  authority, explicit-only recalculation), "immediately" becomes "the
  user presses a recalculate key after editing". Implementation
  follows whichever of 018/010 lands first; the plan notes the
  dependency and the tasks phrase the trigger accordingly.

## Phase 1 Design

### Data model

```text
QuantityBuffer { None, Reserve(u32), Multiplier(f32) }   // exclusive by type
Item { ..., rate: f32 (default 1.0), buffer: QuantityBuffer (default None) }
ItemQuantitySpec::Auto { rate: f32, buffer: QuantityBuffer } | Fixed(u32)
SavedItem { ..., rate: Option<f32>, reserve: Option<u32>, buffer_multiplier: Option<f32> }
```

Calculation: `Auto` items → rate amount (R3), buffer applied, clamp
1–99. `Fixed` items → unchanged.

### Contracts

- Dialog bindings (composed with spec 020's model): Up/Down cycle
  Name → Category → Quantity → Mode → Rate → Buffer → Name; Left/Right
  adjust the focused value (Rate steps 0.05, Buffer steps its kind's
  value); the Buffer kind cycles None → Reserve → Multiplier on a
  dedicated key (Space on the Buffer field, matching the Mode-field
  pattern).
- Hints (AddEditPopup): the new keys added; entries must fit the
  minimum width (hint tests).
- Help: the dialog's bindings documented; extraction tests gain the
  new tokens.
- `docs/quantity-calculation.md`: the per-item rule replaces the
  global-multiplier-only description.

### Quickstart

- The user's own examples as verification: 5-day trip, underwear rate
  1.0 + reserve 2 → 7; trousers rate 0.15 → 1; 14-day trip trousers →
  2 (0.15 × 14 = 2.1 → 2); socks rate 1.0 × 1.25 on 5 days → 7
  (5 × 1.25 = 6.25 → 6, then clamp; verify against the rule).
- Pre-feature config loads and calculates identically (defaults).
- Exclusivity: set a reserve, then a multiplier — the reserve clears.
- Round-trip: save, restart, reload — modifiers intact.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None. The buffer enum is the simplest structure that makes the spec's exclusivity a type guarantee. | | |
