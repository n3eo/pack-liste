# Quickstart: Label-Based Item Storage (revision 3)

Runnable validation for the feature. Prerequisite: the full quality gate exits 0 with zero warnings.

## 0. The migration (one-time, before the first run)

```bash
cp ~/.config/pack-liste/config.yaml ~/.config/pack-liste/config.yaml.bak
# Apply the external conversion:
#   categories: ['Ausland: Karten'] → labels: [Ausland, Karten] + singleton: true
#   categories: ['Kulturbeutel']    → labels: [Kulturbeutel]
#   exact duplicates merge (first wins; the report lists every merge)
# The top-level `categories:` list is dropped.
```

**Expected**: the config stores `labels` per item; multi-label items carry `singleton: true`; no ": " anywhere.

## 1. The schema and the dedup (US1/US2)

```bash
cargo test config_io
```

**Expected**: passes — the strict label load, the dedup on load (duplicates merge, different-labels variants remain), the singleton optionality, the round trip, and the old-format rejection.

## 2. The old format is rejected (US4)

```bash
cargo run   # with an old-format config in place
```

**Expected**: a clear load error naming the unknown field (`categories`); the file untouched.

## 3. The flat grouping and the singleton toggles (US3/US5)

```bash
cargo run -- /path/to/trip.yaml
```

- The panels show one flat header per label; a multi-label item appears under each of its labels.
- The chips: every item shows its labels behind its name in grey.
- A singleton (e.g. a migrated "Ausland: Karten" item): toggle its selection under "Ausland" — the "Karten" occurrence flips too; its chips are uniformly dark grey.
- A non-singleton: toggle under one label — only that occurrence changes; its chip under the other label turns light grey (differs).
- `C` collapses a label group; `Ctrl+Space` selects a label's items; `R`/`Ctrl+R` recalculate a label's items.
- The add/edit dialog: assign multiple labels (Space toggles, free text adds), toggle Singleton, save.

**Expected**: the flat grouping, the synchronized singleton toggles, the independent non-singleton toggles, and the chips all behave per the contract.

## 4. Scrolling at both extremes (US3, FR-008)

With either panel focused: scroll fully down (the last item visible, highlighted) and fully up (the first label header visible, highlighted); repeat with a label collapsed.

**Expected**: every row reachable; the sticky header never displaces the cursor; the highlight follows it.

## 5. The round trip (US1)

```bash
# Load, navigate, quit (saves); compare the label fields
grep -E "labels|singleton" ~/.config/pack-liste/config.yaml > /tmp/after.txt
```

**Expected**: every label/singleton value byte-identical; each item once (the dedup holds through the save).

## 6. Regression (SC-007)

```bash
cargo test
```

**Expected**: the full suite passes — the category-era tests are gone with the model; the behavior tests (collapse, selection, recalc, sticky, scroll) pass against the label constructors.
