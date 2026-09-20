# Feature Specification: Warning-Free Codebase via Static Analysis

**Feature Branch**: `009-clippy-clean-codebase`

**Created**: 2026-09-20

**Status**: Draft

**Input**: User description: "use cargo check and cargo clippy and fix all issues to create good rust code"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Clean static analysis with no warnings (Priority: P1)

A developer runs the project's static analysis commands (`cargo check` and `cargo clippy`) and gets a completely clean result: no compiler warnings and no linter warnings. Every warning that exists today — unused code, dead modules, needless returns, and anything else the toolchain reports — is resolved by removing the cause, fixing the code, or (only where justified) suppressing it with a documented, scoped annotation. The result is a codebase where the standard quality gates pass with zero output.

**Why this priority**: This is the entire request. A warning-free build is the baseline the user asked for; without it, nothing else in this feature matters.

**Independent Test**: Run `cargo check` and `cargo clippy` (with the project's warning-as-error setting); both complete successfully with no warnings emitted.

**Acceptance Scenarios**:

1. **Given** the current codebase with its existing warnings, **When** the developer runs `cargo check`, **Then** it completes with zero warnings.
2. **Given** the current codebase, **When** the developer runs `cargo clippy`, **Then** it completes with zero warnings (no errors, no warnings).
3. **Given** the codebase is warning-free, **When** the developer builds the tests, **Then** the test build is also warning-free.

---

### User Story 2 - Fixes improve the code, not silence the tools (Priority: P2)

A developer reviews the changes made to reach the warning-free state and finds genuine improvements: dead code is removed rather than annotated away, needless patterns are simplified to idiomatic form, and any remaining suppression is rare, narrowly scoped, and carries a comment explaining why it is justified. No functionality changed: the application behaves exactly as before, and all existing tests still pass.

**Why this priority**: The point of the exercise is good code, not a green checkmark. Mechanically silencing the linter would defeat the request; the fixes must be real.

**Independent Test**: Review the change set: dead code deleted, simplifications applied, every suppression annotated with a reason; run the full test suite and verify identical results.

**Acceptance Scenarios**:

1. **Given** a warning about unused code, **When** it is resolved, **Then** the code is removed (with its call sites and module wiring) rather than kept and annotated.
2. **Given** a warning about a non-idiomatic pattern, **When** it is resolved, **Then** the code is rewritten in the idiomatic form the linter suggests.
3. **Given** a warning that cannot be resolved without harming the code, **When** it is suppressed, **Then** the suppression is scoped to the specific item and carries a comment justifying it.
4. **Given** all fixes are applied, **When** the developer runs the application, **Then** behavior is unchanged and all existing tests pass.

---

### User Story 3 - The clean state is guarded going forward (Priority: P3)

A developer contributes to the project later and accidentally introduces a warning. The project's quality gates catch it: the lint configuration treats warnings as errors in the standard verification flow, so the mistake surfaces immediately instead of accumulating. The guard is part of the project's documented verification commands.

**Why this priority**: A warning-free state erodes quickly without enforcement; the guard keeps the codebase good after the cleanup, not just on the day of the cleanup.

**Independent Test**: Introduce a deliberate unused variable, run the verification command, and verify it fails; remove it and verify the command passes again.

**Acceptance Scenarios**:

1. **Given** the warning-free codebase, **When** the developer runs the project's documented lint command with warnings-as-errors, **Then** any introduced warning fails the check.
2. **Given** the project documentation, **When** a new developer reads the verification section, **Then** the clean-build requirement and the commands to run are stated.

---

### Edge Cases

- What happens if a warning points to code that is planned for a future feature (e.g., scaffolding)?
  **Answer**: The code is removed. Per the project's own principles, scaffolding for future features is not kept in the codebase; it is re-added when the feature is actually built.
- What happens if a linter suggestion conflicts with readability or the project's style?
  **Answer**: The suggestion is not applied blindly; the code is restructured so the warning's cause disappears while keeping the code clear. If that is impossible, a scoped, documented suppression is used.
- What happens if a warning originates in generated code or dependencies?
  **Answer**: Warnings from the project's own code are fixed; dependency warnings are out of scope (they are not emitted by the project's source and cannot be fixed there).
- What happens if fixing one warning reveals another (e.g., removing a dead module exposes unused imports)?
  **Answer**: The analysis is re-run after each wave of fixes until a fully clean pass is reached; the fix loop is iterative, not single-pass.
- What about warnings that only appear in the test build or under specific feature flags?
  **Answer**: The clean requirement covers all standard build configurations of the project: the normal build, the test build, and the lint run.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: `cargo check` MUST complete on the current codebase with zero warnings.
- **FR-002**: `cargo clippy` MUST complete on the current codebase with zero warnings.
- **FR-003**: The test build (`cargo check --tests` or equivalent) MUST complete with zero warnings.
- **FR-004**: Dead or unused code MUST be removed, including its module wiring and call sites, rather than annotated to silence the warning.
- **FR-005**: Any linter suppression MUST be scoped to the smallest possible item and MUST carry a comment stating why it is justified.
- **FR-006**: The fixes MUST NOT change application behavior; all existing tests MUST pass unchanged after the cleanup.
- **FR-007**: The project's lint verification MUST treat warnings as errors so regressions fail the check.
- **FR-008**: The project documentation MUST state the verification commands and the warning-free requirement.

### Key Entities *(include if feature involves data)*

- **Static analysis**: The compiler check and linter passes over the project's own source; the quality gate for this feature.
- **Warning**: A compiler or linter diagnostic on the project's own code; each one is resolved (fixed, removed, or justified-and-suppressed).
- **Suppression**: A scoped, documented annotation that silences a specific lint for a specific item; the exception, not the rule.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: `cargo check` emits zero warnings on the project's own code.
- **SC-002**: `cargo clippy` emits zero warnings on the project's own code.
- **SC-003**: 100% of the warnings present before the cleanup are resolved (fixed, removed, or justified-and-suppressed); none are ignored silently.
- **SC-004**: All existing tests pass with identical results after the cleanup.
- **SC-005**: The warnings-as-errors lint gate fails when a warning is introduced and passes when it is removed.

## Assumptions

- The toolchain (compiler and linter) is available in the development environment; no new dependencies are added by this feature.
- Warnings from third-party dependencies are out of scope; only the project's own source is subject to the clean requirement.
- The cleanup is behavior-preserving: refactors are limited to what the warnings require, with no drive-by redesign.
- Code that is currently dead but plausibly useful later is still removed; it lives in version control history and can be restored when needed.
- The project's existing style conventions (formatting, error handling) are respected; the linter's idiomatic suggestions are applied within them.
- The warnings-as-errors guard applies to the standard verification flow (local runs and CI-equivalent commands), configured in the project's lint setup rather than per-developer settings.
