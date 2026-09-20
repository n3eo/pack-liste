# Specification Quality Checklist: Remove Activity Options

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

- The removal surface was verified in code before writing: activities
  appear in the welcome screen's focus cycle and rendering, the config
  screen's third section, the calculation's activity-priority rule,
  both persisted files, and the help screen. All are enumerated in the
  spec's removal table.
- This spec deliberately reverses implemented behavior: spec 005's
  US-4 (activity weighting) and FR-004/FR-006 (activity priority), and
  spec 002's activity management. Alignment notes for both are flagged
  in Assumptions; spec 015's US3 (unknown-activity fallback) becomes
  moot. The constitution's Principle I governs: removed, not annotated.
- The load-side tolerance of leftover activity fields (US-3) is scoped
  as "ignore harmless extra data", explicitly not the version
  compatibility machinery that draft spec 013 removes — 013 targets
  default-substitution for missing fields, not ignoring extra ones.
- All checklist items pass; no [NEEDS CLARIFICATION] markers were
  needed. The calculation reverts to spec 005's pre-activity rule
  (weather applies directly).
