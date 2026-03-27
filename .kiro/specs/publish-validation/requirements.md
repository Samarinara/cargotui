# Requirements Document

## Introduction

This feature adds a "pre-publish validation" step to the Publish section of the cargotui TUI application. Before a user publishes or packages a crate, the Validator performs a set of offline, local checks against the workspace's `Cargo.toml` manifest to confirm the crate meets crates.io publishing guidelines. Results are displayed in the TUI output panel with clear pass/fail indicators, allowing the user to fix issues before attempting an actual publish.

## Glossary

- **Validator**: The component responsible for running all pre-publish checks against a `Cargo.toml` manifest file.
- **Manifest**: The `Cargo.toml` file located at the workspace or crate root.
- **ValidationResult**: A structured value representing the outcome of a single check, containing a check name, pass/fail status, and an optional diagnostic message.
- **ValidationReport**: The complete collection of `ValidationResult` values produced by a single Validator run.
- **Check**: An individual rule applied by the Validator to the Manifest (e.g., "description is present").
- **Publish_Menu**: The "Publish" submenu within the cargotui TUI menu tree.
- **Output_Panel**: The right-hand panel in the TUI that displays command output and validation results.
- **SPDX_Expression**: A license identifier string conforming to the SPDX license expression syntax (e.g., `MIT`, `Apache-2.0`, `MIT OR Apache-2.0`).

---

## Requirements

### Requirement 1: Validate Required Metadata Fields

**User Story:** As a crate author, I want the Validator to check that all fields required by crates.io are present in my `Cargo.toml`, so that I can fix missing metadata before attempting to publish.

#### Acceptance Criteria

1. THE Validator SHALL check that the `[package]` section of the Manifest contains a non-empty `name` field.
2. THE Validator SHALL check that the `[package]` section of the Manifest contains a non-empty `version` field.
3. THE Validator SHALL check that the `[package]` section of the Manifest contains a non-empty `description` field.
4. THE Validator SHALL check that the `[package]` section of the Manifest contains at least one of `license` or `license-file` fields.
5. WHEN the `version` field is present, THE Validator SHALL check that the value is a valid semantic version string (e.g., `MAJOR.MINOR.PATCH`).
6. IF the `name` field is absent or empty, THEN THE Validator SHALL produce a `ValidationResult` with status `Fail` and the message `"missing required field: name"`.
7. IF the `version` field is absent or empty, THEN THE Validator SHALL produce a `ValidationResult` with status `Fail` and the message `"missing required field: version"`.
8. IF the `description` field is absent or empty, THEN THE Validator SHALL produce a `ValidationResult` with status `Fail` and the message `"missing required field: description"`.
9. IF neither `license` nor `license-file` is present, THEN THE Validator SHALL produce a `ValidationResult` with status `Fail` and the message `"missing required field: license or license-file"`.

---

### Requirement 2: Validate Crate Name Format

**User Story:** As a crate author, I want the Validator to check that my crate name conforms to crates.io naming rules, so that I avoid a rejected publish due to an invalid name.

#### Acceptance Criteria

1. THE Validator SHALL check that the `name` field contains only ASCII alphanumeric characters, hyphens (`-`), or underscores (`_`).
2. THE Validator SHALL check that the `name` field begins with an ASCII alphabetic character.
3. THE Validator SHALL check that the `name` field does not exceed 64 characters in length.
4. IF the `name` field contains characters outside the allowed set, THEN THE Validator SHALL produce a `ValidationResult` with status `Fail` and a message identifying the invalid characters.
5. IF the `name` field begins with a non-alphabetic character, THEN THE Validator SHALL produce a `ValidationResult` with status `Fail` and the message `"crate name must start with an alphabetic character"`.
6. IF the `name` field exceeds 64 characters, THEN THE Validator SHALL produce a `ValidationResult` with status `Fail` and the message `"crate name must not exceed 64 characters"`.

---

### Requirement 3: Validate Version Format

**User Story:** As a crate author, I want the Validator to confirm my version string is a valid semver, so that crates.io accepts the version field.

#### Acceptance Criteria

1. THE Validator SHALL parse the `version` field as a semantic version string conforming to the semver 2.0.0 specification.
2. IF the `version` field is not a valid semver string, THEN THE Validator SHALL produce a `ValidationResult` with status `Fail` and the message `"version is not a valid semver string"`.
3. WHEN the `version` field is a valid semver string, THE Validator SHALL produce a `ValidationResult` with status `Pass` for the version check.

---

### Requirement 4: Validate License Field

**User Story:** As a crate author, I want the Validator to check that my license value is a recognized SPDX expression, so that crates.io can index my crate correctly.

#### Acceptance Criteria

1. WHEN the `license` field is present, THE Validator SHALL check that the value is a non-empty string.
2. WHEN the `license` field is present, THE Validator SHALL check that the value conforms to a valid SPDX_Expression by verifying it contains only recognized SPDX license identifiers and operators (`AND`, `OR`, `WITH`).
3. WHEN the `license-file` field is present instead of `license`, THE Validator SHALL check that the referenced file exists on the local filesystem relative to the Manifest directory.
4. IF the `license` field is present but empty, THEN THE Validator SHALL produce a `ValidationResult` with status `Fail` and the message `"license field is empty"`.
5. IF the `license` field contains an unrecognized SPDX identifier, THEN THE Validator SHALL produce a `ValidationResult` with status `Warn` and the message `"license may not be a recognized SPDX expression"`.
6. IF the `license-file` field references a file that does not exist, THEN THE Validator SHALL produce a `ValidationResult` with status `Fail` and the message `"license-file path does not exist"`.

---

### Requirement 5: Validate Optional but Recommended Fields

**User Story:** As a crate author, I want the Validator to warn me when recommended fields are absent, so that my crate page on crates.io is complete and discoverable.

#### Acceptance Criteria

1. THE Validator SHALL check whether the `readme` field is present in the `[package]` section.
2. THE Validator SHALL check whether the `repository` field is present in the `[package]` section.
3. THE Validator SHALL check whether the `homepage` field is present in the `[package]` section.
4. THE Validator SHALL check whether the `keywords` field is present and contains at least one entry.
5. THE Validator SHALL check whether the `categories` field is present and contains at least one entry.
6. IF the `readme` field is absent, THEN THE Validator SHALL produce a `ValidationResult` with status `Warn` and the message `"readme field is missing (recommended)"`.
7. IF the `readme` field is present, THEN THE Validator SHALL check that the referenced file exists on the local filesystem relative to the Manifest directory.
8. IF the `readme` field references a file that does not exist, THEN THE Validator SHALL produce a `ValidationResult` with status `Fail` and the message `"readme file path does not exist"`.
9. IF the `repository` field is absent, THEN THE Validator SHALL produce a `ValidationResult` with status `Warn` and the message `"repository field is missing (recommended)"`.
10. IF the `keywords` field contains more than 5 entries, THEN THE Validator SHALL produce a `ValidationResult` with status `Fail` and the message `"keywords must not exceed 5 entries"`.
11. IF the `categories` field contains more than 5 entries, THEN THE Validator SHALL produce a `ValidationResult` with status `Fail` and the message `"categories must not exceed 5 entries"`.

---

### Requirement 6: Validate Manifest Parsability

**User Story:** As a crate author, I want the Validator to confirm that my `Cargo.toml` can be parsed, so that I know the file is well-formed before attempting to publish.

#### Acceptance Criteria

1. THE Validator SHALL parse the Manifest file as TOML before running any checks.
2. IF the Manifest file does not exist at the expected path, THEN THE Validator SHALL produce a single `ValidationResult` with status `Fail` and the message `"Cargo.toml not found"` and halt further checks.
3. IF the Manifest file exists but cannot be parsed as valid TOML, THEN THE Validator SHALL produce a single `ValidationResult` with status `Fail` and the message `"Cargo.toml is not valid TOML: <parse error>"` and halt further checks.
4. WHEN the Manifest is successfully parsed, THE Validator SHALL proceed to run all remaining checks.

---

### Requirement 7: Display Validation Results in the TUI

**User Story:** As a crate author, I want to see the validation results displayed in the TUI output panel with clear pass/warn/fail indicators, so that I can quickly identify and fix issues.

#### Acceptance Criteria

1. THE Output_Panel SHALL display each `ValidationResult` on a separate line.
2. THE Output_Panel SHALL prefix each `Pass` result line with a `✓` symbol rendered in green.
3. THE Output_Panel SHALL prefix each `Warn` result line with a `⚠` symbol rendered in yellow.
4. THE Output_Panel SHALL prefix each `Fail` result line with a `✗` symbol rendered in red.
5. THE Output_Panel SHALL display a summary line after all individual results showing the total count of Pass, Warn, and Fail results.
6. WHEN all checks produce a `Pass` result, THE Output_Panel SHALL display the summary `"All checks passed — ready to publish"`.
7. WHEN at least one check produces a `Fail` result, THE Output_Panel SHALL display the summary `"Validation failed: <N> error(s), <M> warning(s)"`.
8. WHEN no checks produce a `Fail` result but at least one produces a `Warn` result, THE Output_Panel SHALL display the summary `"Validation passed with <M> warning(s)"`.

---

### Requirement 8: Integrate Validation into the Publish Menu

**User Story:** As a crate author, I want a "Validate" option in the Publish menu, so that I can run pre-publish checks without leaving the TUI.

#### Acceptance Criteria

1. THE Publish_Menu SHALL contain a menu item named `"validate"` with the description `"Check crates.io publishing requirements (offline)"`.
2. WHEN the user selects the `"validate"` menu item and presses Enter, THE Validator SHALL run all checks against the Manifest in the current workspace root.
3. WHEN the Validator completes, THE Output_Panel SHALL display the ValidationReport.
4. THE Validator SHALL complete all checks without making any network requests.
5. WHEN no workspace is loaded, THE Validator SHALL produce a `ValidationResult` with status `Fail` and the message `"no workspace loaded"`.

---

### Requirement 9: Manifest Parser Round-Trip Integrity

**User Story:** As a developer maintaining the Validator, I want the Manifest parsing logic to preserve all field values faithfully, so that checks operate on accurate data.

#### Acceptance Criteria

1. THE Validator SHALL parse the Manifest using the `toml` crate's deserialization facilities.
2. FOR ALL valid Manifest TOML strings, parsing the TOML into a structured value and then serializing it back to TOML SHALL produce a document that, when re-parsed, yields an equivalent structured value (round-trip property).
3. THE Validator SHALL not modify the Manifest file on disk at any point during validation.
