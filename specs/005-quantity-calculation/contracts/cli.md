# CLI Contract: Quantity Calculation

**Feature**: Quantity Calculation
**Date**: 2026-09-19
**Spec**: [spec.md](../spec.md)

## Overview
The CLI contract defines the command-line interface for the quantity calculation feature. It specifies the commands, arguments, and flags available to users.

## Commands

### `pack-liste` (Main Command)
The main command for the application.

**Usage**:
```sh
pack-liste [OPTIONS] <COMMAND>
```

**Options**:
- `-h, --help` - Print help information.
- `-V, --version` - Print version information.

**Subcommands**:
- `new` - Create a new packing list.
- `open <name>` - Open an existing packing list.
- `list` - List all packing lists.
- `export <name> <file>` - Export a packing list to a file.
- `import <file> <name>` - Import a packing list from a file.
- `calculate` - Calculate quantities based on inputs.

---

### `pack-liste new`
Create a new packing list.

**Usage**:
```sh
pack-liste new [OPTIONS] <NAME>
```

**Arguments**:
- `<NAME>` - Name of the packing list.

**Options**:
- `-h, --help` - Print help information.

---

### `pack-liste open`
Open an existing packing list.

**Usage**:
```sh
pack-liste open [OPTIONS] <NAME>
```

**Arguments**:
- `<NAME>` - Name of the packing list to open.

**Options**:
- `-h, --help` - Print help information.

---

### `pack-liste list`
List all packing lists.

**Usage**:
```sh
pack-liste list [OPTIONS]
```

**Options**:
- `-h, --help` - Print help information.

---

### `pack-liste export`
Export a packing list to a file.

**Usage**:
```sh
pack-liste export [OPTIONS] <NAME> <FILE>
```

**Arguments**:
- `<NAME>` - Name of the packing list to export.
- `<FILE>` - Path to the output file.

**Options**:
- `-h, --help` - Print help information.

---

### `pack-liste import`
Import a packing list from a file.

**Usage**:
```sh
pack-liste import [OPTIONS] <FILE> <NAME>
```

**Arguments**:
- `<FILE>` - Path to the input file.
- `<NAME>` - Name for the imported packing list.

**Options**:
- `-h, --help` - Print help information.

---

### `pack-liste calculate`
Calculate quantities based on inputs.

**Usage**:
```sh
pack-liste calculate [OPTIONS]
```

**Options**:
- `--weekdays <DAYS>` - Number of weekdays for the trip.
- `--travel-style <STYLE>` - Travel style (e.g., "minimalist", "luxury", "backpacker").
- `--weather <CONDITION>` - Weather condition (e.g., "hot", "cold", "rainy").
- `--activities <ACTIVITY>` - Planned activity (can be specified multiple times).
- `-h, --help` - Print help information.

---

## Error Handling

The CLI will return the following exit codes:
- `0` - Success.
- `1` - General error (e.g., invalid input, file not found).
- `2` - Usage error (e.g., missing required arguments).

## Examples

### Create a new packing list
```sh
pack-liste new "Summer Trip 2026"
```

### Open an existing packing list
```sh
pack-liste open "Summer Trip 2026"
```

### Calculate quantities
```sh
pack-liste calculate --weekdays 7 --travel-style minimalist --weather hot --activities hiking --activities beach
```

### Export a packing list
```sh
pack-liste export "Summer Trip 2026" "summer-trip-2026.json"
```

### Import a packing list
```sh
pack-liste import "summer-trip-2026.json" "Summer Trip 2026"
```

> **Alignment note (2026-09-20)**: This contract predates the shipped
> implementation and is superseded. The application has no CLI subcommands;
> the only CLI surface is `pack-liste [OPTIONS] [FILE]` (debug, force-tui,
> and an optional trip file path). Packing-list management commands were
> superseded by the trip-file flow (spec 004).
