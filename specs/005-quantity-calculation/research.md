# Research: Quantity Calculation Implementation

**Feature**: Quantity Calculation
**Date**: 2026-09-19
**Spec**: [spec.md](../spec.md)

## Decision: Rat CLI Best Practices

**Rationale**: Rat CLI applications in Rust benefit from modular design, clear separation of concerns, and robust error handling. Using `thiserror` for custom error types and `anyhow` for error handling in main functions ensures maintainability and clarity.

**Alternatives Considered**:
- Using `std::error::Error` directly: Less expressive and harder to maintain.
- Using `snafu`: More complex than necessary for this project.

## Decision: Ratatui Architecture

**Rationale**: Ratatui applications should separate views into different modules to improve maintainability and testability. Each view (e.g., input, list, summary) should be self-contained and communicate via application state.

**Alternatives Considered**:
- Single-file TUI: Harder to maintain and test.
- Using a framework like `egui`: Overkill for a terminal-based application.

## Decision: Error Handling in Rust

**Rationale**: Using `thiserror` for all errors and creating one error enum per function that returns an error ensures consistency and clarity. This approach makes it easy to understand what can go wrong and how to handle it.

**Alternatives Considered**:
- Using `anyhow` for all errors: Less type-safe and harder to maintain.
- Using `std::error::Error` directly: Less expressive.

## Decision: File Organization in Rust

**Rationale**: Splitting Rust projects into sensible modules (e.g., `models`, `services`, `ui`) improves readability and maintainability. Each module should have a clear responsibility and minimal coupling.

**Alternatives Considered**:
- Single-file project: Harder to navigate and maintain.
- Using a monolithic structure: Less scalable.

## Decision: Quantity Calculation Algorithms

**Rationale**: The quantity calculation should consider weekdays, travel style, weather, and activities. A weighted approach ensures that the most relevant inputs (e.g., activities) have the greatest impact on the final quantities.

**Alternatives Considered**:
- Simple linear scaling: Less flexible and less accurate.
- Hardcoded quantities: Not adaptable to user inputs.
