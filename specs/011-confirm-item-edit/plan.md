# Implementation Plan: Confirm Before Saving Item Edits

**Branch**: `011-confirm-item-edit` | **Date**: 2026-09-20 | **Spec**: [spec.md](spec.md)

## Summary

Insert a confirmation step between the edit dialog's save and the
write: pressing the dialog's save key shows a prompt describing the
pending change (the item's new values, US2); only confirming applies
the edit and writes the config. Cancelling (or Esc) discards the edit
with nothing written — the item and the config file are byte-identical
to before the dialog opened (US3). The flow mirrors the existing
delete confirmation (`InputMode::ConfirmDelete`); adding keeps its
current no-confirmation behavior.

**Sequencing note**: Lands after 020/021 (the dialog's field set is
final first — the confirmation shows the final field values). The
pending-edit snapshot must carry whatever fields exist at
implementation time (name, categories, quantity, mode; plus rate and
buffer once 010 lands).

## Technical Context

**Language/Version**: Rust 1.98 (2024 edition)

**Primary Dependencies**: ratatui 0.30, crossterm 0.29 (unchanged)

**Storage**: None new — the confirmation gates the existing
`save_edited_item` → `save_config` path.

**Testing**: `cargo test` — 42 existing tests; new state-layer
tests: confirm applies the edit and saves; cancel discards (item and
config unchanged); the prompt carries the new values. The
delete-confirmation flow's tests must pass unchanged.

**Target Platform**: Terminal TUI (Linux)

**Project Type**: CLI/TUI application (single binary)

**Performance Goals**: Unchanged.

**Constraints**: Warning-free gate; cancel is side-effect-free (US3 —
no file write on the cancel path); the prompt fits small terminals
(US2/AC3, consistent with the other popups); adding an item is not
confirmed (scope note).

**Scale/Scope**: A new `InputMode::ConfirmEdit` variant carrying the
pending edit, the dialog handler's save arm routes there, a
confirmation popup renderer (the delete popup's pattern), and the
confirm/cancel arms.

## Constitution Check

- **Principle I**: satisfied — the new mode is reachable and tested.
- **Principle II**: satisfied — gate clean per task.
- **Principle III**: satisfied — the confirmation binding documented
  in hints/help; tests ship with the feature.
- No violations.

## Project Structure

```text
src/
└── ui/
    ├── state.rs          # InputMode::ConfirmEdit { pending: PendingEdit }
    │                     #   PendingEdit: the dialog's values + item index
    ├── handlers/
    │   ├── welcome.rs    # the dialog's save arm routes to ConfirmEdit;
    │   │                 #   Enter confirms (apply + save), Esc/N cancels
    │   │                 #   (discard, nothing written)
    │   └── dialogs.rs    # save_edited_item unchanged; called only on
    │                     #   confirm
    ├── draw.rs           # draw_confirm_edit_popup (the delete popup's
    │                     #   pattern; shows the new values, US2)
    ├── hints.rs / help.rs    # the confirmation's bindings
    └── tests/
        └── input.rs / state.rs   # confirm/cancel/discard tests
```

**Structure Decision**: The pending edit is a snapshot struct on the
new `InputMode` variant — the dialog's state is consumed into it, so
cancelling cannot half-apply. The popup renderer follows
`draw_confirm_delete_popup` (centered, terminal-fitting).

## Phase 0 Research

- **D1 — The existing pattern (verified)**:
  `InputMode::ConfirmDelete { index }` + `draw_confirm_delete_popup`
  + the handler's Enter/Esc arms is the exact flow to mirror; the
  delete popup renders the item name and the Enter/Esc hint.
- **D2 — What the prompt shows (US2)**: the pending edit's new values
  — name, categories, quantity, mode (and rate/buffer post-010).
  Old values shown where space allows; the popup fits small
  terminals (the delete popup's `min()` sizing).
- **D3 — Cancel is byte-identical (US3)**: the edit is applied only
  in the confirm arm (`save_edited_item` + `save_config`); the cancel
  arm drops the pending snapshot and returns to Normal mode. No code
  path writes the config on cancel.
- **D4 — Esc layering**: Esc in `ConfirmEdit` cancels the
  confirmation (back to the list, edit discarded) — one Esc, not two
  (the dialog is already consumed into the snapshot).

## Phase 1 Design

### Data model

```text
PendingEdit { index, name, categories, quantity, auto_quantity, (rate, buffer post-010) }
InputMode::ConfirmEdit { pending: PendingEdit }
```

Dialog save → `ConfirmEdit { pending }`. Enter → apply + save. Esc/N
→ discard. The dialog's `EditingItem` state is consumed into the
snapshot.

### Contracts

- Bindings: Enter = confirm, Esc = cancel (the delete popup's set);
  hints (a ConfirmEditPopup surface or the shared DeletePopup-style
  line) and help document them; extraction tests pin them.
- The prompt names the item and shows the new values.

### Quickstart

- Edit name + quantity, save the dialog → the prompt shows both new
  values; confirm → applied, config written.
- Same edit, cancel → the item and the config file are unchanged
  (diff the file).
- Esc at the prompt → back on the list, nothing written.
- Add flow: no confirmation (unchanged).

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None. | | |
