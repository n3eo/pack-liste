# Specification Quality Checklist: Recalculate Amounts on an Existing List

**Purpose**: Validate specification completeness and quality before proceeding to planning
**Created**: 2026-09-20
**Feature**: [spec.md](../spec.md)

## Content Quality

- [X] No implementation details (languages, frameworks, APIs)
- [X] Focused on user value and business needs
- [X] Written for non-technical stakeholders
- [X] All mandatory sections completed

## Requirement Completeness

- [X] No [NEEDS CLARIFICATION] markers remain
- [X] Requirements are testable and unambiguous
- [X] Success criteria are measurable
- [X] Success criteria are technology-agnostic (no implementation details)
- [X] All acceptance scenarios are defined
- [X] Edge cases are identified
- [X] Scope is clearly bounded
- [X] Dependencies and assumptions identified

## Feature Readiness

- [X] All functional requirements have clear acceptance criteria
- [X] User scenarios cover primary flows
- [X] Feature meets measurable outcomes defined in Success Criteria
- [X] No implementation details leak into specification

## Notes

- The need was verified against the code: recalculation currently
  happens only as a side effect of parameter/config changes
  (update_quantities call sites); no explicit user action exists. The
  gap becomes acute with draft spec 012 (persisted quantities restored
  without recalculation on resume).
- The action is scoped as a pure in-session recalculation: no file
  writes, no non-quantity state changes, idempotent (US-1, FR-004).
  The auto/fixed mode semantics (spec 003, spec 015's Mode field) are
  explicitly preserved (US-3).
- The key binding is deliberately left to planning with `R` as the
  documented preference (EC-001); the spec pins the requirement (one
  key, hints + help coverage, extraction-test enforced) rather than
  the letter.
- All checklist items pass; no [NEEDS CLARIFICATION] markers were
  needed.
- Amended 2026-09-20 (two refinements, recorded verbatim in the
  Input section): (1) manual quantity edits become authoritative and
  all automatic recalculation triggers are removed — recalculation
  only on explicit key press; (2) the action gains three scopes —
  current item, current category, everything. The requested
  space-chords ("space + r", "space + shift + r") are not
  implementable (terminals report only Shift/Ctrl/Alt as key
  modifiers, not Space); the user chose R / Ctrl+R / Ctrl+Shift+R as
  the binding scheme. The spec was rewritten in place: new US-1
  (manual authority), US-2 (no automatic triggers), US-3 (three
  scopes), US-5 (persistence/load), amended US-4 (documentation of
  all three keys), new EC-001..EC-008, and FR-001..FR-009 replacing
  the original six. The original single-scope US-1/US-3 content is
  superseded by the scoped model. Note: this spec was previously
  implemented and converged in its original form; the amendments
  reopen it — the implementation must be re-planned
  (/speckit-plan) before the changes are built.
- All checklist items pass after the amendment; no [NEEDS
  CLARIFICATION] markers remain (the binding question was asked and
  answered: R / Ctrl+R / Ctrl+Shift+R).
