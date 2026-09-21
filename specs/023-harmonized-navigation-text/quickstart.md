# Quickstart: Harmonized Navigation Text

Runnable validation for the feature. Prerequisite: the full quality gate (`cargo check`, `cargo clippy`, `cargo clippy --tests`, `cargo test`, `cargo fmt --check`) exits 0 with zero warnings.

## 1. Canonical uniqueness (US1)

```bash
cargo test canonical_action_text_is_unique_per_key
```

**Expected**: passes — no key shows two different action texts for the same meaning on any surface.

## 2. No duplication on one screen (US2)

```bash
cargo test no_pair_repeats_on_one_screen
```

**Expected**: passes — the main screen's visible hint lines contain no (key, action) pair twice; same for the config screen with the modifier editor open.

## 3. Help agreement (US1/US3)

```bash
cargo test help_tokens_match_canonical_table
```

**Expected**: passes — every help key token is a canonical key; each help description contains its key's canonical action word.

## 4. Single-source check (US3)

```bash
# The action strings exist exactly once in the source:
grep -c '"Navigate"' src/ui/hints.rs        # 1
grep -c '"Recalc item"' src/ui/hints.rs     # 1
grep -c '"Switch panel"' src/ui/hints.rs    # 1
```

**Expected**: each canonical action string appears exactly once in `src/ui/hints.rs` (the canonical table); the surface compositions reference it.

## 5. Visual check (all stories)

Run the application and inspect the hint lines:

```bash
cargo run -- /tmp/pack-liste-verify/ref.yaml
```

- Main screen: the left panel's footer shows only left-specific bindings; the right panel's footer carries the shared bindings — no binding appears in both footers.
- Welcome screen: the footer's wording matches the canonical table.
- Open the add dialog (`A`): the hint rows use the canonical wording (`Tab Next field`, `↑/↓ Navigate`, `←/→ Adjust`, `Space Toggle`).
- Open help (`?`): every listed key matches the canonical table's keys; the descriptions contain the canonical action words.

**Expected**: identical wording for the same key everywhere it appears; no visible duplication on any screen.

## 6. Behavior unchanged (FR-006)

Navigate every screen and trigger every binding from the canonical table.

**Expected**: every key does exactly what it did before — the harmonization changed only the displayed text. The full test suite passes unmodified (plus the three new tests).
