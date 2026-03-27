# Tasks: publish-validation

## Task List

- [x] 1. Create `src/cargo/publish_validator.rs` with core validation logic
  - [x] 1.1 Define `ValidationStatus`, `ValidationResult`, and `ValidationReport` types
  - [x] 1.2 Implement `run_validation(manifest_dir: &Path) -> ValidationReport`
    - [x] 1.2.1 Read and parse `Cargo.toml` as TOML; return early with Fail on missing file or parse error (Requirements 6.1–6.4)
    - [x] 1.2.2 Check required fields: `name`, `version`, `description`, `license`/`license-file` (Requirements 1.1–1.9)
    - [x] 1.2.3 Check crate name format: allowed chars, starts with alpha, max 64 chars (Requirements 2.1–2.6)
    - [x] 1.2.4 Check version is valid semver (Requirements 3.1–3.3)
    - [x] 1.2.5 Check license SPDX expression or license-file existence (Requirements 4.1–4.6)
    - [x] 1.2.6 Check optional recommended fields: `readme`, `repository`, `homepage`, `keywords`, `categories` (Requirements 5.1–5.11)
  - [x] 1.3 Implement `ValidationReport::summary()`, `pass_count()`, `warn_count()`, `fail_count()` (Requirements 7.5–7.8)
  - [x] 1.4 Implement `format_report(report: &ValidationReport) -> Vec<ratatui::text::Line<'static>>` with colored ✓/⚠/✗ symbols (Requirements 7.1–7.4)

- [x] 2. Integrate validator into the cargo module
  - [x] 2.1 Add `pub mod publish_validator;` to `src/cargo/mod.rs`
  - [x] 2.2 Add `CommandAction::Validate` variant to the `CommandAction` enum in `src/cargo/mod.rs`
  - [x] 2.3 Add `"validate"` `CommandNode` to the `Publish` submenu in `COMMAND_TREE` with description `"Check crates.io publishing requirements (offline)"` (Requirement 8.1)
  - [x] 2.4 Update `clone_action` in `src/app.rs` to handle `CommandAction::Validate`

- [x] 3. Handle `CommandAction::Validate` in `App::handle_event`
  - [x] 3.1 In the `AppMode::Menu` Enter branch, match `CommandAction::Validate` and dispatch validation (Requirements 8.2–8.3, 8.5)
  - [x] 3.2 When no workspace is loaded, push a single Fail result `"no workspace loaded"` to the output buffer (Requirement 8.5)
  - [x] 3.3 When workspace is loaded, call `run_validation`, then `format_report`, then push lines into `OutputBuffer` via `start_command` + `push_line` (Requirements 8.2–8.3)

- [x] 4. Write unit tests in `src/cargo/publish_validator.rs`
  - [x] 4.1 Test manifest with all required fields present → no Fail results
  - [x] 4.2 Test manifest missing each required field individually → correct Fail message
  - [x] 4.3 Test invalid semver strings (`"1.0"`, `"abc"`, `""`) → Fail
  - [x] 4.4 Test valid semver strings (`"1.0.0"`, `"0.1.0-alpha"`) → Pass
  - [x] 4.5 Test crate name edge cases: valid names, names starting with digit, names with invalid chars, name exceeding 64 chars
  - [x] 4.6 Test `keywords` with 6 entries → Fail; 5 entries → Pass
  - [x] 4.7 Test non-existent `license-file` path → Fail; non-existent `readme` path → Fail
  - [x] 4.8 Test invalid TOML content → single Fail with parse error message
  - [x] 4.9 Test `format_report` symbols: ✓ for Pass, ⚠ for Warn, ✗ for Fail
  - [x] 4.10 Test `summary()` for all-pass, mixed-warn, and fail cases (Requirements 7.6–7.8)
  - [x] 4.11 Test `"validate"` node exists in the Publish submenu of `COMMAND_TREE` (Requirement 8.1)

- [x] 5. Write property-based tests in `src/cargo/publish_validator.rs`
  - [x] 5.1 Property 1: For any manifest missing one or more required fields, the report contains at least one Fail result per missing field
  - [x] 5.2 Property 2: For any manifest with all required fields valid, the report contains no Fail results for required-field checks
  - [x] 5.3 Property 3: For any string, name check result is Pass iff the string matches `^[a-zA-Z][a-zA-Z0-9_-]{0,63}$`
  - [x] 5.4 Property 4: For any string the validator accepts as valid semver, splitting on `.` yields exactly three non-negative integer components
  - [x] 5.5 Property 5: For any manifest where keywords or categories has more than 5 entries, the report contains a Fail; for ≤5 entries, no Fail for that check
  - [x] 5.6 Property 6: For any invalid TOML string, `run_validation` returns exactly one result with status Fail and message containing `"Cargo.toml is not valid TOML"`
  - [x] 5.7 Property 7: For any ValidationReport, every Pass line from `format_report` contains `"✓"`, every Warn line contains `"⚠"`, every Fail line contains `"✗"`
  - [x] 5.8 Property 8: For any ValidationReport, `summary()` returns the correct string based on fail/warn counts
  - [x] 5.9 Property 9: For any manifest file, the file bytes are identical before and after calling `run_validation`
  - [x] 5.10 Property 10: For any valid TOML value, serializing then re-parsing yields an equivalent value (round-trip)
