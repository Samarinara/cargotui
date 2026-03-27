# Design Document: publish-validation

## Overview

The publish-validation feature adds an offline pre-publish validation step to the cargotui TUI. A new `Validator` module inspects the workspace `Cargo.toml` manifest against crates.io publishing rules and renders structured pass/warn/fail results in the existing `OutputBuffer`/output panel. No network requests are made; all checks are purely local.

The feature integrates into the existing `Publish` submenu by adding a `"validate"` `CommandNode` backed by a new `CommandAction::Validate` variant. When selected, the app runs the validator synchronously (it is fast and offline), formats the `ValidationReport` into colored output lines, and pushes them into the `OutputBuffer`.

---

## Architecture

```mermaid
graph TD
    A[User selects 'validate' in Publish menu] --> B[App::handle_event dispatches CommandAction::Validate]
    B --> C[cargo::publish_validator::run_validation(workspace_root)]
    C --> D[parse Cargo.toml via toml crate]
    D --> E{parse ok?}
    E -- No --> F[ValidationReport with single Fail result]
    E -- Yes --> G[Run all checks: name, version, license, optional fields]
    G --> H[ValidationReport]
    H --> I[format_report -> Vec of colored Lines]
    I --> J[OutputBuffer::start_command + push_line per result]
    J --> K[Output panel renders results]
```

The validator is a pure function: `run_validation(manifest_dir: &Path) -> ValidationReport`. It has no side effects beyond reading the filesystem. The TUI integration is a thin dispatch layer in `App::handle_event`.

---

## Components and Interfaces

### `src/cargo/publish_validator.rs` (new file)

Core validation logic. Exposes:

```rust
pub enum ValidationStatus { Pass, Warn, Fail }

pub struct ValidationResult {
    pub check: &'static str,
    pub status: ValidationStatus,
    pub message: String,
}

pub struct ValidationReport {
    pub results: Vec<ValidationResult>,
}

impl ValidationReport {
    pub fn summary(&self) -> String { ... }
    pub fn pass_count(&self) -> usize { ... }
    pub fn warn_count(&self) -> usize { ... }
    pub fn fail_count(&self) -> usize { ... }
}

/// Entry point. Reads and validates the Cargo.toml at `manifest_dir`.
pub fn run_validation(manifest_dir: &Path) -> ValidationReport { ... }

/// Format a ValidationReport into colored ratatui Lines for the output panel.
pub fn format_report(report: &ValidationReport) -> Vec<ratatui::text::Line<'static>> { ... }
```

### `src/cargo/mod.rs` (modified)

- Add `pub mod publish_validator;`
- Add `CommandAction::Validate` variant to the `CommandAction` enum
- Add `"validate"` `CommandNode` to the `Publish` submenu in `COMMAND_TREE`

### `src/app.rs` (modified)

- Handle `CommandAction::Validate` in `handle_event` / `AppMode::Menu` Enter branch:
  1. Determine `manifest_dir` from `self.workspace` (or produce a "no workspace loaded" fail report)
  2. Call `run_validation(&manifest_dir)`
  3. Call `format_report(&report)` to get lines
  4. Push lines into `self.output` via `start_command` + `push_line`

No new `AppMode` variant is needed — validation is synchronous and fast.

---

## Data Models

```rust
// ValidationStatus: the outcome of a single check
pub enum ValidationStatus {
    Pass,
    Warn,
    Fail,
}

// ValidationResult: outcome of one named check
pub struct ValidationResult {
    pub check: &'static str,   // human-readable check name, e.g. "name"
    pub status: ValidationStatus,
    pub message: String,       // diagnostic or success message
}

// ValidationReport: the full set of results from one validator run
pub struct ValidationReport {
    pub results: Vec<ValidationResult>,
}
```

The validator reads the manifest into a `toml::Value` (untyped) rather than a strongly-typed struct, so it can distinguish "field absent" from "field present but empty" and produce precise diagnostic messages.

### SPDX identifier list

A static `&[&str]` of recognized SPDX identifiers is embedded in the validator (covering the most common ones: `MIT`, `Apache-2.0`, `GPL-2.0-only`, `GPL-3.0-only`, `BSD-2-Clause`, `BSD-3-Clause`, `ISC`, `MPL-2.0`, `LGPL-2.1-only`, `LGPL-3.0-only`, `CC0-1.0`, `Unlicense`). The check tokenizes the `license` field by splitting on `AND`, `OR`, `WITH`, and whitespace, then verifies each token against the list. An unrecognized token produces a `Warn` (not `Fail`) per Requirement 4.5.

### Semver validation

Version strings are validated by splitting on `.` and checking that exactly three numeric components exist (MAJOR.MINOR.PATCH), optionally followed by pre-release/build metadata. This avoids adding a new dependency; the `semver` crate is not currently in `Cargo.toml`.

---

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system — essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Missing required fields produce Fail results

*For any* manifest that omits one or more of `name`, `version`, `description`, or both `license`/`license-file`, the `ValidationReport` SHALL contain at least one `Fail` result for each missing field.

**Validates: Requirements 1.1, 1.2, 1.3, 1.4, 1.6, 1.7, 1.8, 1.9**

### Property 2: Valid required fields produce Pass results

*For any* manifest that contains non-empty `name`, `version` (valid semver), `description`, and `license` (recognized SPDX), the `ValidationReport` SHALL contain no `Fail` results for the required-field checks.

**Validates: Requirements 1.1, 1.2, 1.3, 1.4, 3.3**

### Property 3: Crate name validity is consistent with the allowed character set

*For any* string, the name check result SHALL be `Pass` if and only if the string matches `^[a-zA-Z][a-zA-Z0-9_-]{0,63}$` (starts with alpha, only alphanumeric/hyphen/underscore, at most 64 chars).

**Validates: Requirements 2.1, 2.2, 2.3, 2.4, 2.5, 2.6**

### Property 4: Semver validation round-trip

*For any* string that the validator accepts as a valid semver (produces `Pass`), splitting on `.` SHALL yield exactly three components that are all parseable as non-negative integers (ignoring pre-release/build suffixes).

**Validates: Requirements 3.1, 3.2, 3.3**

### Property 5: Keywords and categories count enforcement

*For any* manifest where `keywords` or `categories` contains more than 5 entries, the `ValidationReport` SHALL contain a `Fail` result for that field; for 5 or fewer entries the result SHALL be `Pass` or `Warn` (not `Fail`).

**Validates: Requirements 5.10, 5.11**

### Property 6: Unparseable manifest halts further checks

*For any* string that is not valid TOML, calling `run_validation` with a manifest directory containing that content SHALL return a `ValidationReport` with exactly one result, whose status is `Fail` and whose message contains `"Cargo.toml is not valid TOML"`.

**Validates: Requirements 6.2, 6.3**

### Property 7: Format report output contains check name and status symbol

*For any* `ValidationReport`, every line produced by `format_report` for a `Pass` result SHALL contain `"✓"`, every `Warn` line SHALL contain `"⚠"`, and every `Fail` line SHALL contain `"✗"`.

**Validates: Requirements 7.1, 7.2, 7.3, 7.4**

### Property 8: Summary line correctness

*For any* `ValidationReport`, the string returned by `summary()` SHALL be:
- `"All checks passed — ready to publish"` when `fail_count() == 0 && warn_count() == 0`
- `"Validation passed with <M> warning(s)"` when `fail_count() == 0 && warn_count() > 0`
- `"Validation failed: <N> error(s), <M> warning(s)"` when `fail_count() > 0`

**Validates: Requirements 7.5, 7.6, 7.7, 7.8**

### Property 9: Manifest file is never modified

*For any* manifest file on disk, the byte content of the file SHALL be identical before and after calling `run_validation`.

**Validates: Requirement 9.3**

### Property 10: TOML round-trip integrity

*For any* valid TOML string, parsing it to a `toml::Value` and serializing it back with `toml::to_string`, then re-parsing, SHALL yield a value equal to the original parsed value.

**Validates: Requirement 9.2**

---

## Error Handling

| Scenario | Behavior |
|---|---|
| `Cargo.toml` not found | Single `Fail` result: `"Cargo.toml not found"` |
| `Cargo.toml` not valid TOML | Single `Fail` result: `"Cargo.toml is not valid TOML: <parse error>"` |
| `license-file` path does not exist | `Fail` result: `"license-file path does not exist"` |
| `readme` file path does not exist | `Fail` result: `"readme file path does not exist"` |
| No workspace loaded in TUI | Single `Fail` result: `"no workspace loaded"` |
| Unrecognized SPDX identifier | `Warn` result: `"license may not be a recognized SPDX expression"` |
| Optional field absent | `Warn` result with `"(recommended)"` suffix |

All errors are surfaced as `ValidationResult` values — the validator never panics and never returns a Rust `Err`. This keeps the TUI integration simple: it always receives a `ValidationReport` it can render.

---

## Testing Strategy

### Unit tests

Unit tests cover specific examples and edge cases:

- Manifest with all required fields present → no `Fail` results
- Manifest missing `name` → `Fail` with message `"missing required field: name"`
- Manifest missing `version` → `Fail` with message `"missing required field: version"`
- Manifest missing `description` → `Fail` with message `"missing required field: description"`
- Manifest missing both `license` and `license-file` → `Fail`
- Invalid semver strings: `"1.0"`, `"abc"`, `""` → `Fail`
- Valid semver strings: `"1.0.0"`, `"0.1.0-alpha"` → `Pass`
- Crate name `"my-crate"` → `Pass`; `"1bad"` → `Fail`; `"a".repeat(65)` → `Fail`
- `keywords` with 6 entries → `Fail`; 5 entries → `Pass`
- Non-existent `license-file` path → `Fail`
- Non-existent `readme` path → `Fail`
- Invalid TOML content → single `Fail` with parse error message
- `format_report` symbols: `✓` for Pass, `⚠` for Warn, `✗` for Fail
- `summary()` for all-pass, mixed warn, and fail cases

### Property-based tests

Using `proptest` (already in `[dev-dependencies]`), minimum 100 iterations each:

```
// Feature: publish-validation, Property 1: Missing required fields produce Fail results
// Feature: publish-validation, Property 2: Valid required fields produce Pass results
// Feature: publish-validation, Property 3: Crate name validity consistent with allowed charset
// Feature: publish-validation, Property 4: Semver validation round-trip
// Feature: publish-validation, Property 5: Keywords and categories count enforcement
// Feature: publish-validation, Property 6: Unparseable manifest halts further checks
// Feature: publish-validation, Property 7: Format report output contains check name and status symbol
// Feature: publish-validation, Property 8: Summary line correctness
// Feature: publish-validation, Property 9: Manifest file is never modified
// Feature: publish-validation, Property 10: TOML round-trip integrity
```

Each property-based test is tagged with the format:
`// Feature: publish-validation, Property N: <property_text>`

Property tests use `tempfile` (already in `[dev-dependencies]`) to create temporary manifest files on disk for filesystem-dependent checks (Properties 6, 9).
