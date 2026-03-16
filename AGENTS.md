# AGENTS.md - Developer Guide for Aquascope SVG

This file provides guidance for AI agents working on this codebase.

## Project Overview

Aquascope SVG is a Rust tool that converts Aquascope JSON output into standalone SVG
diagrams for visualizing Rust memory behavior. The project parses JSON trace data
representing stack and heap states and renders them as publication-quality SVGs.

## Build Commands

```bash
# Build the project
cargo build                    # Debug build
cargo build --release          # Release build

# Run all tests
cargo test

# Run a single test by name
cargo test test_basic_golden

# Run tests in integration file
cargo test --test integration

# Check code (faster than build, no linking)
cargo check

# Run clippy for linting
cargo clippy

# Format code
cargo fmt
```

## Project Structure

```
aquascope_svg/
├── src/
│   ├── main.rs           # Entry point, CLI argument parsing
│   └── mtrace.rs         # Type definitions matching TypeScript types in mtrace.ts
├── tests/
│   └── integration.rs    # Integration tests
├── testdata/             # Test JSON files (*.golden files)
├── Cargo.toml            # Project manifest
└── README.md            # Project documentation
```

## Code Style Guidelines

### Imports

- Group imports: standard library first, then external crates
- Use explicit imports rather than glob imports
- Order: `use std::`, then `use crate::`, then `use external_crate::`

```rust
// Good
use clap::Parser;
use serde_json::Value;
use std::fs;

mod mtrace;

// Bad
use std::*;
```

### Formatting

- Use `cargo fmt` for automatic formatting
- Maximum line length: 100 characters (default Rustfmt)
- Use 4 spaces for indentation (Rust standard)
- One blank line between top-level items

### Types

- Use explicit type annotations where helpful for clarity
- Match TypeScript types from `src/mtrace.ts` when parsing JSON
- Use appropriate integer sizes: `u32` for small indices, `u64` for bigint
- Use `Option<T>` for nullable values

### Naming Conventions

- **Structs/Enums**: PascalCase (e.g., `MValue`, `CharRange`)
- **Fields**: snake_case (e.g., `body_span`, `moved_paths`)
- **Functions**: snake_case (e.g., `parse_json`)
- **Modules**: snake_case (e.g., `mod mtrace`)
- **Constants**: SCREAMING_SNAKE_CASE
- Prefix interfaces with `M` (e.g., `MValue`, `MTrace`) to match TypeScript

### Serde Attributes

Use `#[serde(...)]` attributes for JSON field mapping:
- `#[serde(rename = "...")]` for field name mapping
- `#[serde(tag = "type")]` for externally tagged enums
- `#[serde(skip)]` to skip fields

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MValue {
    Bool { value: bool },
    // ...
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MValueAdt {
    pub name: String,
    #[serde(rename = "alloc_kind")]
    pub alloc_kind: Option<MHeapAllocKind>,
}
```

### Error Handling

- Use `.expect()` for unrecoverable errors (file not found, parsing failure)
- For production code, consider `anyhow` or `thiserror` for richer error types
- Match the error messages in existing code

```rust
let content = fs::read_to_string(&args.input).expect("Failed to read input file");
let json: Value = serde_json::from_str(&content).expect("Failed to parse JSON");
```

### Derive Macros

Always derive these traits on types:
- `Debug` - for debugging output
- `Clone` - for value semantics
- `Serialize` - for JSON output
- `Deserialize` - for JSON input

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharPos { ... }
```

### Recursive Types

When a type contains itself (like `MValue` containing `Vec<MValue>`), use `Box<T>` to
avoid infinite size:

```rust
// Good - use Box for recursive type
Only { value: (Vec<MValue>, Box<MValue>) }

// Avoid - recursive without indirection causes compile error
Only { value: (Vec<MValue>, MValue) }
```

### Testing

- Integration tests go in `tests/` directory
- Unit tests can be in source files with `#[cfg(test)]` module
- Use `assert_cmd` crate for CLI testing

```rust
// tests/integration.rs
use assert_cmd::prelude::*;
use std::process::Command;

#[test]
fn test_basic_golden() {
    let mut cmd = Command::cargo_bin("aquascope_svg").unwrap();
    cmd.arg("testdata/basic.golden").assert().success();
}
```

### TypeScript Alignment

The `src/mtrace.rs` types should mirror `src/mtrace.ts`:
- Enum variants match TypeScript discriminated unions
- Field types map: `number` -> `u32`, `bigint` -> `u64`, `string` -> `String`
- Nullable types map to `Option<T>`

## Existing Cursor/Copilot Rules

No existing `.cursor/rules/`, `.cursorrules`, or `.github/copilot-instructions.md`
files found.

## Common Tasks

### Adding a new test data file
1. Add JSON file to `testdata/` (e.g., `testdata/new_test.golden`)
2. Add integration test in `tests/integration.rs`

### Adding new type definitions
1. Add type to `src/mtrace.rs`
2. Add corresponding TypeScript type in `src/mtrace.ts` (if applicable)
3. Add `#[derive(Debug, Clone, Serialize, Deserialize)]`
4. Run `cargo test` to verify

### Modifying CLI arguments
1. Edit `src/main.rs` - update `Args` struct with clap attributes
2. Run `cargo check` to verify
3. Add test for new argument

## Notes

- Edition 2024 in Cargo.toml
- Dependencies: clap, serde, serde_json, assert_cmd (dev)
- No external linter CI configured currently