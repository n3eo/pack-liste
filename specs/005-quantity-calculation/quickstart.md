# Quickstart Validation Guide: Quantity Calculation

## Overview

This guide provides runnable validation scenarios to prove the Quantity Calculation feature works end-to-end.

## Prerequisites

- Rust 1.98 (2024 edition) installed
- Cargo package manager
- Terminal with ANSI color support

## Setup

```bash
# Clone the repository (if not already done)
git clone <repository-url>
cd pack-liste

# Build the project
cargo build

# Run the application
cargo run
```

## Validation Scenarios

### Scenario 1: Basic Quantity Calculation with Weekdays

**Test**: Verify that quantities update correctly when weekdays are changed.

**Steps**:
1. Run the application: `cargo run`
2. Navigate to input view (Press '1')
3. Input 5 weekdays
4. Verify that base quantities are calculated for 5 days

**Expected Outcome**:
- Base quantities should be proportional to 5 weekdays
- No errors should be displayed

**Success Criteria**:
- Quantities are calculated within 1 second
- All calculated quantities are accurate based on input

---

### Scenario 2: Travel Style Adjustment

**Test**: Verify that quantities update correctly when travel style is changed.

**Steps**:
1. Run the application: `cargo run`
2. Navigate to input view (Press '1')
3. Select "luxury" travel style
4. Input 7 weekdays
5. Verify that quantities are increased by 1.5x multiplier

**Expected Outcome**:
- Quantities should be 1.5 times the base quantities
- System should display the selected travel style

**Success Criteria**:
- Quantities are adjusted based on travel style multiplier
- Changes are reflected in real-time (<1 second)

---

### Scenario 3: Weather Input Adjustment

**Test**: Verify that quantities update correctly when weather input is changed.

**Steps**:
1. Run the application: `cargo run`
2. Navigate to input view (Press '1')
3. Select "cold" weather
4. Input 3 weekdays
5. Verify that quantities are increased by 1.5x multiplier for clothing items

**Expected Outcome**:
- Clothing item quantities should be increased
- System should display the selected weather input

**Success Criteria**:
- Quantities are adjusted based on weather multiplier
- Changes are reflected in real-time

---

### Scenario 4: Activities Adjustment

**Test**: Verify that quantities update correctly when activities are changed.

**Steps**:
1. Run the application: `cargo run`
2. Navigate to input view (Press '1')
3. Select "hiking" and "camping" activities
4. Input 4 weekdays
5. Verify that quantities are adjusted based on activity multipliers

**Expected Outcome**:
- Quantities should reflect the average multiplier of selected activities
- System should display the selected activities

**Success Criteria**:
- Quantities are adjusted based on activity multipliers
- Changes are reflected in real-time

---

### Scenario 5: Packing List Management

**Test**: Verify packing list creation, renaming, export, and import.

**Steps**:
1. Run the application: `cargo run`
2. Create a new packing list
3. Add items to the list
4. Export the packing list to a file
5. Rename the packing list
6. Import the packing list from the file

**Expected Outcome**:
- Packing list is created with default name
- Packing list can be renamed successfully
- Packing list is exported to JSON file in `~/.config/pack-liste/`
- Packing list is imported successfully with all items and quantities

**Success Criteria**:
- Packing list name is validated (1-50 characters)
- Export/import maintains 100% data fidelity
- File is saved in correct location

---

### Scenario 6: Editable Quantities and Packed Status

**Test**: Verify that users can edit quantities and toggle packed status.

**Steps**:
1. Run the application: `cargo run`
2. Navigate to list view (Press '2')
3. Select an item
4. Edit its quantity
5. Toggle its packed status

**Expected Outcome**:
- Quantity is updated successfully
- Packed status is toggled successfully
- Changes persist across views

**Success Criteria**:
- Quantity validation works (must be >= 1)
- Packed status changes are reflected immediately

---

### Scenario 7: Edge Cases

#### EC-001: Conflicting Data
**Test**: Input conflicting data (e.g., "hot weather" and "snowboarding activity")

**Steps**:
1. Run the application: `cargo run`
2. Select "hot" weather
3. Select "snowboarding" activity
4. Verify system prioritizes activity and shows warning

**Expected Outcome**:
- System prioritizes activity input
- Warning message is displayed
- Quantities are adjusted based on activity

---

#### EC-002: Invalid Inputs
**Test**: Input invalid values (e.g., negative weekdays)

**Steps**:
1. Run the application: `cargo run`
2. Try to input -5 weekdays
3. Verify system displays error message

**Expected Outcome**:
- Error message is displayed
- Packing list is not updated
- User can correct the input

---

#### EC-003: Large Inputs
**Test**: Input very large number of weekdays (e.g., 100)

**Steps**:
1. Run the application: `cargo run`
2. Input 100 weekdays
3. Verify system caps quantities at 20

**Expected Outcome**:
- Quantities are capped at 20
- System handles large inputs gracefully

---

#### EC-005: Unnamed Packing List
**Test**: Try to export a packing list with no name

**Steps**:
1. Run the application: `cargo run`
2. Create a packing list without naming it
3. Try to export it

**Expected Outcome**:
- System assigns default name (e.g., "Unnamed Packing List - <timestamp>")
- Export succeeds with default name

---

#### EC-006: Corrupted JSON File
**Test**: Try to import a corrupted JSON file

**Steps**:
1. Create a corrupted JSON file in `~/.config/pack-liste/`
2. Run the application: `cargo run`
3. Try to import the corrupted file

**Expected Outcome**:
- Error message is displayed
- Import is aborted
- Application continues running

---

## Test Commands

### Run All Tests

```bash
cargo test
```

### Run Specific Tests

```bash
# Unit tests
cargo test --lib

# Integration tests
cargo test --test '*'
```

### Check for Warnings

```bash
cargo build 2>&1 | grep -i warning
```

### Performance Test

```bash
# Measure build time
time cargo build --release

# Measure runtime performance
cargo run --release
```

---

## Success Criteria Verification

| Criteria | Test | Expected Result |
|----------|------|-----------------|
| SC-001 | All scenarios | Quantity calculations consider all inputs |
| SC-002 | Scenario 5 | Packing lists export/import with 100% fidelity |
| SC-003 | Scenario 5 | All packing lists have unique names (max 50 chars) |
| SC-004 | All scenarios | Updates within 1 second |
| SC-005 | All scenarios | 100% accurate quantities |
| SC-006 | EC-001 | Conflicting inputs handled gracefully |
| SC-007 | EC-002 | Invalid inputs show clear errors |

---

## Troubleshooting

### Common Issues

1. **Compilation Errors**:
   - Ensure Rust 1.98 is installed
   - Run `cargo clean` and rebuild
   - Check for missing dependencies in Cargo.toml

2. **Runtime Errors**:
   - Check that `~/.config/pack-liste/` directory exists and is writable
   - Verify terminal supports ANSI colors

3. **Test Failures**:
   - Run tests with `-- --nocapture` to see output
   - Check test assumptions match implementation

---

## Next Steps

After successful validation:

1. Deploy to target environment
2. Monitor performance in production
3. Collect user feedback for improvements

> **Alignment note (2026-09-20)**: This guide predates the shipped
> implementation; validate against the current application behavior
> (trip files, YAML config, welcome-screen inputs) rather than the steps
> described here.
