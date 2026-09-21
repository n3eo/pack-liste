# Research: Harmonized Navigation Text

## R1 — The current duplication inventory (verified in code)

The per-surface literal lists in `hint_entries` produce this state:

**Pairs already consistent across surfaces** (14): `↑/↓→Navigate` (5 surfaces), `←/→→Adjust` (2), `Tab→Switch panel` (2), `F→Fullscreen` (2), `R/Ctrl+R/Ctrl+Shift+R→Recalc…` (2 each), `?→Help` (2), `A/E/D→Add/Edit/Delete` (2 each), `Ctrl+E→Raw editor` (2), `Enter→Confirm` (2), `Esc→Cancel` (4).

**Keys with multiple action texts** (the drift, 5 keys):
- `Enter`: Confirm / Edit file / Continue / New line / Save
- `Esc`: Back / Cancel
- `F`: Edit path / Fullscreen
- `Space`: Select / Packed / Mode / Buffer / Category
- `Tab`: Switch panel / Switch section / Next field

**Same-screen duplication** (LeftPanel ∩ RightPanel, 7 pairs): `↑/↓`, `Tab`, `F`, `R`, `Ctrl+R`, `Ctrl+Shift+R`, `?` — both main panels restate these.

**Decision**: The drift list is the harmonization target for US1; the 7 shared pairs are the de-duplication target for US2.

**Rationale**: These are the exact pairs the spec's SC-001/SC-002 count; fixing exactly these satisfies both criteria.

**Alternatives considered**: Rewriting all action texts from scratch — rejected; the consistent 14 are already canonical, only the 5 drifted keys need one wording each.

## R2 — The canonical wording per drifted key (decision)

Where a key genuinely means different things per surface, each *meaning* gets one canonical text (the spec's edge case allows per-meaning entries):

| Key | Canonical text(s) | Surfaces |
|-----|-------------------|----------|
| `Enter` | `Confirm` (popups), `Save` (add/edit dialog), `Continue` (welcome), `New line` (raw editor) | per meaning |
| `Esc` | `Cancel` (popups/editors), `Back` (config screen) | per meaning |
| `F` | `Fullscreen` (main), `Edit path` (welcome) | per meaning |
| `Space` | `Select` (left panel), `Packed` (right panel), `Toggle` (dialog fields) | per meaning |
| `Tab` | `Switch panel` (main), `Switch section` (config), `Next field` (dialogs) | per meaning |

**Decision**: Keep the per-meaning texts but define each exactly once in a canonical table; surfaces reference entries by identity, never re-type the string.

**Rationale**: The meanings genuinely differ (the spec's edge case); harmonizing wording, not semantics (FR-001/FR-006).

**Alternatives considered**: One text per key regardless of surface (e.g. `Space→Action`) — rejected as uselessly vague; the spec explicitly permits per-meaning canonical entries.

## R3 — The canonical table mechanism (decision)

**Decision**: A `Binding` table in `src/ui/hints.rs`: one `const` array of `(key, action)` entries — the canonical definitions — plus per-surface composition functions that reference entries by index/const name. `hint_entries(surface)` returns slices of the canonical entries; the strings exist exactly once in the source.

**Rationale**: Compile-time single source; a wording change is a one-line edit (SC-004); no runtime cost; the existing `hint_line`/`hint_rows` renderers consume the same `&[(&str, &str)]` shape unchanged (FR-007).

**Alternatives considered**:
- Keep per-surface lists and add a test enforcing equality — rejected: the duplication would remain in source; the spec demands "defined in exactly one place" (FR-005).
- A macro generating surfaces from a DSL — rejected: over-engineering for ~20 entries.

## R4 — Same-screen de-duplication design (decision)

**Decision**: The main screen renders ONE shared hint line for both panels instead of two per-panel lines: the left panel's footer keeps the left-specific entries (`Space Select`, `Ctrl+Space`, `A/E/D/C`), and the right panel's footer keeps the right-specific entry (`Space Packed`); the 7 shared pairs (`↑/↓`, `Tab`, `F`, `R`, `Ctrl+R`, `Ctrl+Shift+R`, `?`) move to a single line rendered once — on the right panel's footer (the compiled list is the primary workspace), or as a shared footer row when the layout allows.

**Rationale**: SC-002 counts pairs across "the hint lines visible on the same screen"; stating each shared pair once satisfies it exactly.

**Alternatives considered**:
- Keeping both panel lines but removing shared pairs from the left — rejected: the left panel would lose its own bindings' visibility; asymmetric.
- A third full-width footer row — rejected: costs a list row on every screen for content that fits in the existing footers.

**Open point resolved**: which panel carries the shared line — the RIGHT panel (the compiled list is where the shared bindings act: navigate, recalculate, fullscreen, help).

## R5 — Help screen agreement (decision)

**Decision**: `help_content()` keeps its verbose descriptions but its key tokens come from the canonical table's key strings (single source); the help test asserts every help key token exists in the canonical table and that the canonical action word appears in the help description for that key.

**Rationale**: FR-004 requires agreement, not identical verbosity; token-level derivation prevents drift (SC-004).

**Alternatives considered**: Generating the help screen entirely from the table — rejected: the help screen's long descriptions (e.g. the recalculate scopes' explanations) are intentional content, not duplication.

## R6 — Test design (decision)

**Decision**: Three new tests in `src/ui/tests/hints.rs`:
1. `canonical_action_text_is_unique_per_key` — for every key, the set of action texts across all surfaces has at most one entry per meaning (i.e., no two surfaces show different texts for the same key unless the meanings are listed as distinct canonical entries).
2. `no_pair_repeats_on_one_screen` — the main screen's combined visible hint lines contain no duplicate (key, action) pair; same for the config screen with the modifier editor open.
3. `help_tokens_match_canonical_table` — every help key token is a canonical key; the canonical action word appears in the corresponding help description.

**Rationale**: These pin SC-001, SC-002, and FR-004 directly; they fail on any future drift.

**Alternatives considered**: Snapshot-testing rendered lines — rejected: brittle against legitimate wording changes; the property tests express the spec's intent.
