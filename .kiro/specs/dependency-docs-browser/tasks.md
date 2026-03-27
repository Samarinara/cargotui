# Implementation Plan: Dependency Docs Browser

## Overview

Implement the `DepBrowser` panel as a new `AppMode` variant that loads workspace packages via `cargo metadata`, displays them in a scrollable list, and opens docs.rs URLs in the system browser. The work is broken into five incremental steps: data helpers, state + command wiring, UI panel, app integration, and status bar.

## Tasks

- [x] 1. Add pure helper functions in `src/ui/dep_browser.rs`
  - Create `src/ui/dep_browser.rs` with three pure functions:
    - `build_doc_url(name: &str, version: &str) -> String` — returns `format!("https://docs.rs/{}/{}", name, version)`
    - `format_package_row(pkg: &PackageInfo) -> String` — returns `format!("{} v{}", pkg.name, pkg.version)`
    - `format_open_success_message(name: &str, version: &str) -> String` — returns `format!("Opening docs for {} v{}…", name, version)`
  - Add `open_url(url: &str) -> std::io::Result<()>` using `#[cfg(target_os)]` guards for `xdg-open` / `open` / `cmd /c start`
  - _Requirements: 3.1, 3.3, 6.1, 6.2, 6.3_

  - [ ]* 1.1 Write property test for `build_doc_url`
    - **Property 1: Doc URL construction correctness**
    - **Validates: Requirements 3.1, 6.1, 6.2, 6.3**

  - [ ]* 1.2 Write property test for `format_package_row`
    - **Property 2: Package row display format**
    - **Validates: Requirements 1.2**

  - [ ]* 1.3 Write property test for `format_open_success_message`
    - **Property 6: Success status message format**
    - **Validates: Requirements 3.3**

  - [ ]* 1.4 Write unit tests for `build_doc_url`
    - `test_doc_url_construction` — known name/version
    - `test_doc_url_hyphen_underscore` — names with `-` and `_` preserved

- [x] 2. Add `DepBrowserState` and extend `AppMode` in `src/app.rs`
  - Define `DepBrowserStatus` enum (`Loading`, `Loaded`, `Error(String)`)
  - Define `DepBrowserState` struct with fields: `packages: Vec<PackageInfo>`, `selected: usize`, `status: DepBrowserStatus`, `message: Option<String>`
  - Implement `DepBrowserState::from_packages(packages: Vec<PackageInfo>) -> Self` — sorts by name, sets `selected = 0`, `status = Loaded`
  - Implement `move_down(&mut self)` and `move_up(&mut self)` with wrapping; guard against empty list
  - Add `AppMode::DepBrowser(DepBrowserState)` variant
  - Add `metadata_buf: String` field to `App` (cleared on DepBrowser entry)
  - _Requirements: 1.5, 2.1, 2.2, 2.3, 2.5_

  - [ ]* 2.1 Write property test for `DepBrowserState::from_packages` sort
    - **Property 3: Dependency list is sorted alphabetically**
    - **Validates: Requirements 1.5**

  - [ ]* 2.2 Write property test for selection index invariant
    - **Property 4: Selection index invariant**
    - **Validates: Requirements 2.1**

  - [ ]* 2.3 Write property test for navigation round trip
    - **Property 5: Navigation round trip**
    - **Validates: Requirements 2.2, 2.3**

  - [ ]* 2.4 Write unit tests for `DepBrowserState`
    - `test_esc_returns_to_menu` — mode transition on Esc/q
    - `test_empty_list_navigation_no_panic` — navigation on empty list stays at 0

- [x] 3. Add `CommandAction::BrowseDocs` and wire the menu entry in `src/cargo/mod.rs`
  - Add `BrowseDocs` variant to `CommandAction`
  - Update `clone_action` in `src/app.rs` to handle `BrowseDocs`
  - Add a `"Browse Docs"` `CommandNode` to the `"Dependencies"` submenu with description `"Browse dependencies and open documentation in browser"` and action `CommandAction::BrowseDocs`
  - _Requirements: 4.1, 4.3_

  - [ ]* 3.1 Write unit test for menu entry
    - `test_browse_docs_in_command_tree` — verifies entry exists with correct description

- [x] 4. Handle `AppMode::DepBrowser` in `App::handle_event` in `src/app.rs`
  - On `CommandAction::BrowseDocs` in `AppMode::Menu` Enter handler: clear `metadata_buf`, set mode to `AppMode::DepBrowser(DepBrowserState { packages: vec![], selected: 0, status: Loading, message: None })`, set `pending_command = Some(CargoCommand::Metadata)`
  - In `AppMode::DepBrowser` match arm handle:
    - `Event::Output(Stdout(line))` → append to `metadata_buf`
    - `Event::Output(Done(status))` → if non-zero: set `DepBrowserStatus::Error(stderr)`; if zero: call `parse_metadata`, sort, set `Loaded`
    - `Event::Key(j/Down)` → `state.move_down()`
    - `Event::Key(k/Up)` → `state.move_up()`
    - `Event::Key(Enter)` on non-empty list → construct URL, call `open_url` via `spawn_blocking`, set `state.message`
    - `Event::Key(Esc/q)` → return to `AppMode::Menu`
  - Ensure `main.rs` event loop launches `CargoCommand::Metadata` when `pending_command` is set from DepBrowser entry (reuses existing pending_command mechanism)
  - _Requirements: 1.1, 1.3, 1.4, 2.1, 2.2, 2.3, 2.4, 3.1, 3.2, 3.3, 3.4, 3.5, 4.2_

  - [ ]* 4.1 Write unit tests for DepBrowser event handling
    - `test_metadata_error_stays_open` — non-zero exit keeps panel open with Error status
    - `test_open_browser_error_message` — failed open sets correct message format

- [x] 5. Checkpoint — ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 6. Implement `render_dep_browser` in `src/ui/dep_browser.rs`
  - Render a bordered `List` of `format_package_row` strings with the selected row highlighted (yellow + bold, matching existing menu style)
  - When `status == Loading`: show centred "Loading dependencies…" paragraph
  - When `status == Error(msg)`: show the error text in the panel
  - When `packages` is empty and `status == Loaded`: show "No packages found"
  - Show `state.message` as a subtitle or inline line when `Some`
  - _Requirements: 1.2, 1.3, 1.4, 5.3_

  - [ ]* 6.1 Write unit test for empty list rendering
    - `test_empty_package_list_message` — "No packages found" is rendered

- [x] 7. Extend `src/ui/mod.rs` and `src/ui/status_bar.rs` for DepBrowser
  - In `src/ui/mod.rs`: add `pub mod dep_browser;`, import `render_dep_browser`, add `AppMode::DepBrowser(state)` arm in `render` to call `render_dep_browser(state, frame, main_area)` (full main area, no menu/output split)
  - In `src/ui/status_bar.rs`: extend `mode_name` and `mode_hints` for `AppMode::DepBrowser`:
    - Loading: `"Loading dependencies…"`
    - Loaded: `"↑/↓ navigate  Enter open docs  Esc back"`
    - Error: `"Esc back"`
  - _Requirements: 5.1, 5.2, 5.3_

  - [ ]* 7.1 Write unit tests for status bar hints
    - `test_status_bar_loading` — "Loading dependencies…"
    - `test_status_bar_loaded` — "↑/↓ navigate  Enter open docs  Esc back"

- [x] 8. Final checkpoint — ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for a faster MVP
- Each task references specific requirements for traceability
- Property tests use `proptest` (already a dev-dependency)
- `open_url` is called inside `tokio::task::spawn_blocking` to keep the TUI event loop non-blocking (Requirement 3.5)
- The metadata fetch reuses the existing `pending_command` / `spawn_cargo` / `EventHandler` pipeline — no new async infrastructure needed
