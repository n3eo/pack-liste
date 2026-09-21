# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-09-21

The storage model release: items are stored with **labels** instead of
categories — a flat, single-level grouping with optional multi-label
membership, deduplicated on load, with **singletons** whose toggles
synchronize across all their labels.

### Added

- **Label-based item storage** (spec 025): every item carries one or
  more `labels` (plain strings, no hierarchy, no `": "` separator);
  the panels group by label — one flat header per distinct label, an
  item appears under every label it carries.
- **Deduplication on load** (spec 025): entries with the same name and
  the same labels load as one item (first occurrence wins); same name
  with different labels remain distinct items kept for review.
- **Singletons** (spec 025): an item flagged `singleton: true` has its
  selection and packed toggles synchronized across all its labels —
  one physical thing appearing in several groups. Non-singletons keep
  independent toggle states per label occurrence.
- **Label chips** (spec 025): every item renders its labels as small
  grey chips behind its name in both panels — dark grey when the
  occurrence's toggle state matches the item's own, light grey when it
  differs. The chips never push the quantity out of the row.
- **Labels multi-select in the add/edit dialog** (spec 025): the
  category field became a labels picker (Left/Right cycles, Space
  toggles membership, free text adds a new label; at least one label
  required to save) plus a Singleton field (Space toggles).
- **Per-label group actions** (spec 025): `Ctrl+Space` selects every
  item carrying the cursor's label; `R` on a label header and
  `Ctrl+R` recalculate every item carrying that label.
- **Harmonized navigation text** (spec 023): one canonical binding
  table drives every screen's footer; the full hint lines collapse to
  a single `? Help` line when the lists would be squeezed.
- **Two-layer category nesting** (spec 024): categories split on
  `": "` rendered as a group with sub-categories beneath — later
  superseded by the flat label model (spec 025), which removed the
  separator convention entirely.

### Changed

- **The configuration format** (specs 024, 025): `categories` (with the
  `": "` separator convention) is replaced by `labels` plus the
  optional `singleton` flag; the top-level `categories` list is gone.
  The old format is rejected with an error naming the unknown field —
  no dual-format parsing, no compatibility layer (spec 013). The
  migration is one-time and external: each `"X: Y"` category became
  the labels `X` and `Y` with `singleton: true`; exact duplicates
  merged (first wins, with a report of every merge).
- **The trip file format** (spec 025): trip items carry `labels` and
  `singleton`; reconciliation matches by name and updates the labels
  and the singleton flag from the config.
- **The config screen binding** moved from `O` to `Ctrl+O` so the
  letter `O` can be typed in the add/edit dialogs.
- **Sticky label headers**: when the scroll window starts mid-group,
  the group's header is pinned to the viewport's first row; the pin
  never displaces the last row at the list's end.

### Fixed

- Scrolling crashes on the compiled list pane (the first element out
  of focus) — the cursor and the scroll window now stay in sync in
  both panels at every terminal size, and every row is reachable at
  both extremes.
- The upper rows missing after scrolling down once — the sticky-header
  displacement logic no longer hides the row under the pinned header.
- The category change in the edit dialog not persisting, and the
  category selection colliding with the Up/Down field navigation.
- The weather box stretching to the bottom of the screen — it stays a
  single line (spec 019).
- The packed marker no longer shows on the left split (spec 008) —
  packing is a right-panel concern only.

## [0.1.0] - 2026-09-20

Initial release: the terminal-based packing list with dynamic quantity
calculation, trip parameters (weekdays, travel style, weather), the
two-panel main screen, the compiled-list persistence, the in-app
configuration screens, and the raw config editor.
