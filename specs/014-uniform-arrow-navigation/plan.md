# Implementation Plan: Uniform Arrow-Key Navigation Model (Closure)

**Branch**: `014-uniform-arrow-navigation` | **Date**: 2026-09-20 | **Spec**: [spec.md](spec.md)

## Summary

Close out the uniform arrow model: **Up/Down always navigate,
Left/Right always adjust the focused value**. The model itself is
implemented — spec 016 removed the main screen's Left/Right collapse
binding, the dialog's category field cycles with Left/Right, and
the hints/help state the final binding set (the spec's own alignment
note records this). The remaining work is scope bookkeeping: verify
every surface against the model, and record the spec as satisfied by
016's implementation with the one known deviation (the add/edit
dialog's Up/Down field cycling) assigned to spec 020.

**Sequencing note**: This is a verification-and-closure plan, not an
implementation plan. It depends on 020 landing for the dialog's full
compliance; the closure verification runs after 020.

## Technical Context

**Language/Version**: Rust 1.98 (2024 edition)

**Primary Dependencies**: none (verification only)

**Storage**: none

**Testing**: The verification is a checklist against the running
application (the quickstart), plus the existing extraction tests
(the binding set is already pinned).

**Constraints**: Warning-free gate (no code changes expected; any
deviation found becomes a task against the owning spec).

## Project Structure

No code changes. The deliverable is the verification record in this
plan and the spec's status update.

## Phase 0 Research — the model's surface inventory

Verified against the current code:

| Surface | Up/Down | Left/Right | Status |
|---------|---------|------------|--------|
| Main screen (normal) | cursor through rows | unbound (016 removed collapse) | compliant |
| Welcome screen | field focus | adjust focused value (weekdays, cycle style/weather) | compliant |
| Add/edit dialog | category dropdown only | category cycling | **deviation — 020** |
| Config screen | modifier cursor | unbound | compliant |
| Modifier editor | field toggle (Tab) | unbound | compliant |
| Raw config editor | cursor | cursor | compliant (text editing) |
| Help screen | n/a (fits) | n/a | compliant |

The one deviation is the add/edit dialog, where Up/Down do not cycle
fields — exactly what spec 020 implements. No other surface
repurposes the arrows.

## Phase 1 — Closure

1. After 020 lands, re-run the surface inventory above; every row
   must read compliant.
2. Update this spec's status to reflect satisfaction (Implemented
   via 016 + 020), with the alignment note already in the spec
   serving as the record.
3. No code changes are expected; any surprise deviation found in
   step 1 becomes a task in the owning spec (not here).

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None. | | |
