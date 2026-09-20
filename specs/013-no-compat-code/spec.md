# Feature Specification: Remove Version-Compatibility Code

**Feature Branch**: `013-no-compat-code`

**Created**: 2026-09-20

**Status**: Draft

**Input**: User description:
"remove any code related to backwards or forwards compatibility of the evolution of this program"

> **Scope note**: The application currently carries compatibility
> machinery so that data files written by earlier versions of the program
> keep loading: optional fields with fallback defaults, load-time
> substitution of built-in values, and tests that pin the old formats.
> The user wants that machinery gone. The program evolves freely; data
> files are regenerated rather than migrated. This spec defines what is
> removed, what happens to files that no longer match the current format,
> and how the affected existing specs are superseded.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - The program loads only its current file formats (Priority: P1)

The user runs the current version of the program. It reads and writes exactly one format: the current one. There is no code that substitutes defaults for missing fields, no optional-field fallbacks papering over older layouts, and no load paths for formats the program no longer produces. A file is either in the current format and loads, or it is not and the program says so.

**Why this priority**: This is the entire request. Compatibility code exists only to serve old files; removing it means the program's data model and its file format are always in exact agreement, with one code path instead of many.

**Independent Test**: Inspect every load path: each field of the persisted structures is required as-is; no field has a load-time fallback to a default or built-in value. A file missing any field fails to load with a clear error.

**Acceptance Scenarios**:

1. **Given** a configuration or trip file in the current format, **When** the program loads it, **Then** every field is read as written, with no substitution.
2. **Given** a file that does not match the current format (missing or unknown fields), **When** the program loads it, **Then** loading fails with a clear error identifying the file; the program does not silently fill in defaults.
3. **Given** the program's source, **When** searched for compatibility mechanisms (optional persisted fields, load-time default substitution, legacy-format tests), **Then** none are found.

---

### User Story 2 - A rejected file never destroys data (Priority: P1)

The user points the program at a file it cannot load — an old-format file, a hand-edited file with a missing field, or a corrupt file. The program reports the problem and starts with a fresh state (or refuses to proceed past the error), but it never overwrites, truncates, or deletes the rejected file. The user can fix or regenerate the file at their leisure.

**Why this priority**: Removing compatibility makes load failures possible where they previously were masked; the failure must be safe. Data loss on a rejected file would be a regression far worse than the removed convenience.

**Independent Test**: Feed the program a file with a missing required field; verify the error is reported, the file is byte-identical afterward, and the program does not write anything until the user next saves deliberately.

**Acceptance Scenarios**:

1. **Given** an unloadable file, **When** the program attempts to load it, **Then** a clear error names the file and the problem.
2. **Given** an unloadable file, **When** the program reports the error, **Then** the file's contents are unchanged on disk.
3. **Given** an unloadable trip file passed as the start argument, **When** the program cannot load it, **Then** the program reports the error and does not silently start a new trip that would overwrite the file on exit.

---

### User Story 3 - Fresh starts are the migration path (Priority: P2)

The user with an old-format file regenerates their data instead of migrating it: the program's documented path for an incompatible file is to start fresh (the program creates a current-format file) or to hand-edit the old file into the current format using the documented format. The program itself performs no conversion, and its documentation states the current format only — no version history of formats.

**Why this priority**: With compatibility removed, the user needs a defined answer to "my old file no longer loads"; regeneration and hand-editing are that answer. Documenting only the current format keeps the documentation honest with the code.

**Independent Test**: Follow the documented path with an old-format file: start fresh and verify a current-format file is produced; hand-edit the old file per the documented format and verify it loads.

**Acceptance Scenarios**:

1. **Given** no loadable file, **When** the user starts fresh, **Then** the program creates a file in the current format with sensible initial content.
2. **Given** the program's documentation, **When** the user reads the format description, **Then** it describes exactly the current format, with no mention of older formats or migration.
3. **Given** an old-format file, **When** the user hand-edits it to match the documented current format, **Then** the program loads it.

---

### User Story 4 - Superseded compatibility requirements are cleared from the specs (Priority: P2)

The project's specifications currently demand compatibility behavior (optional fields, old-file loading, pre-feature file guarantees). After this feature, no active spec requires the program to load files written by earlier versions, and the implemented-behavior notes in the affected specs record the reversal.

**Why this priority**: The specs are the source of truth for what the program must do; leaving compatibility requirements in place would contradict the code and mislead future work.

**Independent Test**: Search the active specs for compatibility requirements; verify none remain unmarked and that the affected specs carry a supersession note pointing here.

**Acceptance Scenarios**:

1. **Given** the active specifications, **When** reviewed after this feature, **Then** no requirement demands loading files written by earlier versions.
2. **Given** a spec that previously required such behavior, **When** it is read, **Then** it carries a note that this feature supersedes that requirement.

### Edge Cases

- **EC-001**: Does "no compatibility code" extend to defaults that serve first-run behavior (creating a config when none exists)?
  **Answer**: No. First-run creation of a fresh file is not compatibility — it is the normal way a new user starts. What is removed is fallback behavior for files that exist but do not match the current format.
  **Acceptance Criteria**: Starting with no config file creates one; starting with a malformed config file reports an error instead of substituting defaults.

- **EC-002**: Are hand-edited files with extra unknown fields rejected?
  **Answer**: Yes — the current format is the contract; unknown fields mean the file is not in the current format. The error names the file so the user can fix it.
  **Acceptance Criteria**: A file with unknown fields fails to load with a clear error.

- **EC-003**: What happens on load failure — does the program exit or continue?
  **Answer**: The program reports the error and does not proceed with substituted state. Exiting with the error (or returning to a state where the user can choose a different file) is acceptable; silently continuing with defaults is not.
  **Acceptance Criteria**: No load failure results in the program running with silently substituted data.

- **EC-004**: Are the compatibility-focused tests removed too?
  **Answer**: Yes — tests that pin old formats (e.g., "old config without trip params loads") are removed with the code they test. Tests for the current format remain and are the regression net.
  **Acceptance Criteria**: No test constructs a file format the current program cannot produce.

- **EC-005**: Does this apply to both persisted files (input config and trip files)?
  **Answer**: Yes — both are program data files; both load only the current format.
  **Acceptance Criteria**: Neither load path contains default-substitution for existing-but-nonmatching files.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The program MUST load only files that match its current data format; every field of the persisted structures MUST be required as written.
- **FR-002**: The program MUST NOT contain load-time compatibility mechanisms: no optional persisted fields with fallback defaults, no substitution of built-in values for missing sections, no legacy-format load paths.
- **FR-003**: A file that does not match the current format MUST fail to load with a clear error naming the file and the problem.
- **FR-004**: A failed load MUST NOT modify, overwrite, or delete the rejected file, and MUST NOT be followed by an automatic save that would overwrite it.
- **FR-005**: The program MUST keep its first-run behavior: when no file exists, a fresh file in the current format is created.
- **FR-006**: The documentation MUST describe the current file format only, without version histories or migration instructions.
- **FR-007**: All compatibility-focused code and tests MUST be removed, and the active specifications MUST carry supersession notes where they previously required compatibility behavior.

### Key Entities *(include if feature involves data)*

- **Current format**: The exact structure the current version of the program writes; the only structure it reads.
- **Load failure**: The explicit, safe outcome for a file that does not match the current format: a clear error, no data modification, no substituted state.
- **First-run creation**: The non-compatibility default: creating a fresh current-format file when none exists.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of fields in the persisted structures are required; zero load-time default substitutions exist in the codebase.
- **SC-002**: 100% of non-matching files fail to load with a clear error; zero are silently repaired with defaults.
- **SC-003**: 100% of rejected files remain byte-identical on disk after the failed load.
- **SC-004**: Zero tests construct file formats the current program cannot produce.
- **SC-005**: The active specs contain zero unmarked requirements to load files written by earlier versions.

## Assumptions

- The user accepts that files written by earlier versions of the program will need regeneration or hand-editing; that is the explicit trade of this feature.
- "Compatibility" means code whose only purpose is accepting data written by other versions of the program (optional fields, default substitution, legacy layouts). It does not mean robustness in general — clear errors and safe failure handling remain.
- First-run creation of a fresh file is normal behavior, not compatibility, and stays.
- The reconciliation of a resumed trip against the current item catalog (dropping unknown items) is current-format behavior between two live files, not version compatibility, and stays.
- Hand-editing remains supported: a file edited to match the documented current format loads; a file edited away from it does not.
