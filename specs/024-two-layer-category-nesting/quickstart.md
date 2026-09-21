# Quickstart: Two-Layer Category Nesting

Runnable validation for the feature. Prerequisite: the full quality gate exits 0 with zero warnings.

## 1. The split (US1)

```bash
cargo test category_split
```

**Expected**: passes — the split on the first ": " (including the multi-separator and separator-free cases) matches the contract in [contracts/display-tree.md](contracts/display-tree.md).

## 2. The row model (US1)

```bash
cargo test two_layer_rows
```

**Expected**: passes — categories sharing a top layer produce one `Group` row followed by their sub-category rows; separator-free categories produce a group with items directly beneath; the right panel builds the same tree over selected items.

## 3. Collapse and selection at both layers (US2)

```bash
cargo test group_collapse
cargo test group_selection
```

**Expected**: passes — collapsing a group hides its sub-tree; collapsing a sub-category hides its items only; `Ctrl+Space` on a group selects everything beneath it; navigation never lands on a hidden row.

## 4. The format round trip (US3)

```bash
# Save a list, note the category strings, reload:
cp ~/.config/pack-liste/config.yaml /tmp/before.yaml
cargo run -- /tmp/pack-liste-verify/ref.yaml   # navigate, quit (saves)
diff <(grep -o "'[^']*'" /tmp/before.yaml | sort) \
     <(grep -o "'[^']*'" ~/.config/pack-liste/config.yaml | sort)
```

**Expected**: no differences in the category strings — stored verbatim, no migration.

## 5. Visual check (US1/US2)

```bash
cargo run -- /tmp/pack-liste-verify/ref.yaml
```

- The left panel shows the two-layer tree: `▼ Ausland`, indented `▼ Karten`, `▼ Handgepäck` beneath the same group, items beneath their sub-category.
- The right panel shows the same tree over the selected items.
- `C` on a group header collapses the whole group; `C` on a sub-category collapses only its items.
- `Ctrl+Space` on a group header selects/deselects everything beneath.
- Scrolling deep into a list pins the enclosing header (group or sub-category) at the viewport top.
- The add dialog's category picker still lists the full strings ("Ausland: Karten").

## 6. Regression (SC-006)

```bash
cargo test
```

**Expected**: the full suite passes — quantities, packing, persistence, and all existing behaviors unchanged.
