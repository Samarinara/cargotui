# Requirements Document

## Introduction

This feature adds a dependency browser to cargo-tui that allows users to view all dependencies in the current Cargo workspace and open the documentation for any selected package in the system's default web browser. The browser is accessible as a dedicated panel within the existing TUI, navigable with keyboard controls consistent with the rest of the application.

## Glossary

- **Dep_Browser**: The dependency documentation browser component — the new TUI panel that lists packages and handles doc-open actions.
- **Dependency_List**: The scrollable list of packages derived from `cargo metadata` output, displayed in the Dep_Browser panel.
- **Package**: A single entry in the Dependency_List, identified by its name and version as reported by `cargo metadata`.
- **Doc_URL**: The docs.rs URL for a given package, constructed as `https://docs.rs/{name}/{version}`.
- **Browser**: The operating system's default web browser, opened via the platform's standard open mechanism (`xdg-open` on Linux, `open` on macOS, `start` on Windows).
- **Workspace**: The Cargo workspace detected at startup, as defined in the existing `cargo::workspace` module.
- **Metadata_Runner**: The component responsible for invoking `cargo metadata --format-version 1` and parsing its JSON output using the existing `cargo::metadata` module.

---

## Requirements

### Requirement 1: Load and Display Dependency List

**User Story:** As a developer, I want to see all packages in my Cargo workspace listed in the TUI, so that I can quickly browse what dependencies are present.

#### Acceptance Criteria

1. WHEN the Dep_Browser panel is opened, THE Metadata_Runner SHALL invoke `cargo metadata --format-version 1` in the Workspace root directory.
2. WHEN `cargo metadata` completes successfully, THE Dep_Browser SHALL display each Package as a row showing the package name and version in the format `{name} v{version}`.
3. WHEN `cargo metadata` returns zero packages, THE Dep_Browser SHALL display the message "No packages found".
4. IF `cargo metadata` exits with a non-zero status, THEN THE Dep_Browser SHALL display the error output in the panel and remain open.
5. THE Dependency_List SHALL be sorted alphabetically by package name.

---

### Requirement 2: Keyboard Navigation of the Dependency List

**User Story:** As a developer, I want to navigate the dependency list with keyboard controls consistent with the rest of the app, so that I don't have to learn new keybindings.

#### Acceptance Criteria

1. WHEN the Dep_Browser panel is active, THE Dep_Browser SHALL highlight exactly one Package as the selected item at all times (when the list is non-empty).
2. WHEN the user presses `j` or `Down`, THE Dep_Browser SHALL move the selection to the next Package in the Dependency_List, wrapping to the first item after the last.
3. WHEN the user presses `k` or `Up`, THE Dep_Browser SHALL move the selection to the previous Package in the Dependency_List, wrapping to the last item before the first.
4. WHEN the user presses `Esc` or `q`, THE Dep_Browser SHALL close and return the application to the Menu mode.
5. WHILE the Dependency_List is empty, THE Dep_Browser SHALL ignore navigation key presses without panicking.

---

### Requirement 3: Open Documentation in Browser

**User Story:** As a developer, I want to press Enter on a dependency to open its docs.rs page in my browser, so that I can read the documentation without leaving the terminal.

#### Acceptance Criteria

1. WHEN the user presses `Enter` on a selected Package, THE Dep_Browser SHALL construct the Doc_URL as `https://docs.rs/{name}/{version}`.
2. WHEN the Doc_URL is constructed, THE Dep_Browser SHALL open the Doc_URL in the Browser using the platform's default open mechanism.
3. WHEN the Browser is launched successfully, THE Dep_Browser SHALL display a status message "Opening docs for {name} v{version}…" in the status bar.
4. IF the platform open command fails, THEN THE Dep_Browser SHALL display an error message "Failed to open browser: {error}" in the status bar.
5. WHILE a browser open operation is in progress, THE Dep_Browser SHALL remain interactive and SHALL NOT block the TUI event loop.

---

### Requirement 4: Access Dep_Browser from the Menu

**User Story:** As a developer, I want to open the dependency browser from the existing Dependencies menu, so that it is discoverable alongside related commands.

#### Acceptance Criteria

1. THE Dep_Browser SHALL be accessible via a "Browse Docs" entry in the existing "Dependencies" submenu of the command tree.
2. WHEN the user selects "Browse Docs" and presses `Enter`, THE Dep_Browser SHALL open and immediately begin loading the Dependency_List.
3. THE Dep_Browser entry SHALL display the description "Browse dependencies and open documentation in browser".

---

### Requirement 5: Status Bar Integration

**User Story:** As a developer, I want the status bar to reflect the current state of the dependency browser, so that I always know what the app is doing.

#### Acceptance Criteria

1. WHILE the Dep_Browser is loading the Dependency_List, THE status bar SHALL display "Loading dependencies…".
2. WHEN the Dependency_List has loaded, THE status bar SHALL display "↑/↓ navigate  Enter open docs  Esc back".
3. WHEN a doc-open action completes or fails, THE status bar SHALL update to reflect the result message within the same render cycle.

---

### Requirement 6: Doc URL Construction Correctness

**User Story:** As a developer, I want the generated docs.rs URL to be correct for any valid package name and version, so that the browser always opens the right page.

#### Acceptance Criteria

1. THE Dep_Browser SHALL construct Doc_URL by concatenating `https://docs.rs/`, the Package name, `/`, and the Package version without any additional path segments.
2. FOR ALL Packages in the Dependency_List, the Doc_URL SHALL be a valid URL with scheme `https`, host `docs.rs`, and path `/{name}/{version}`.
3. WHEN a Package name contains hyphens or underscores, THE Dep_Browser SHALL preserve the name exactly as reported by `cargo metadata` in the Doc_URL.
