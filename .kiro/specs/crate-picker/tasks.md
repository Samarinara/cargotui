# Implementation Plan: crate-picker

## Overview

Implement the CratePicker overlay by adding state types and event handling to `app.rs`, a new `PickCrate` command action variant in `cargo/mod.rs`, a new `src/ui/crate_picker.rs` render function, and wiring everything together in `ui/mod.rs`, `ui/status_bar.rs`, and `main.rs`.

## Tasks

- [x] 1. Add `CommandAction::PickCrate` to `src/cargo/mod.rs`
  - Add `PickCrate(Box<CommandAction>)` variant to the `CommandAction` enum
  - Update `remove <crate>` entry in `COMMAND_TREE` to use `CommandAction::PickCrate(Box::new(CommandAction::Execute(CargoCommand::Remove { krate: String::new() })))`
  - Update `update <crate>` entry in `COMMAND_TREE` to use `CommandAction::PickCrate(Box::new(CommandAction::Execute(CargoCommand::Update { krate: Some(String::new()) })))`
  - _Requirements: 8.1, 8.3, 8.4_

- [x] 2. Add `CratePickerStatus`, `CratePickerState`, and `AppMode::CratePicker` to `src/app.rs`
  - [x] 2.1 Add `CratePickerStatus` enum (mirrors `DepBrowserStatus`)
    - `Loading`, `Loaded`, `Error(String)` variants
    - _Requirements: 1.5, 2.2_

  - [x] 2.2 Add `CratePickerState` struct with `filtered_packages`, `move_down`, `move_up`, `update_filter`
    - Fields: `packages: Vec<PackageInfo>`, `filter: String`, `selected: usize`, `status: CratePickerStatus`, `pending_action: Box<CommandAction>`
    - `filtered_packages(&self) -> Vec<&PackageInfo>`: case-insensitive substring filter; returns all when filter is empty
    - `move_down(&mut self)`: wrapping increment over `filtered_packages().len()`; no-op when empty
    - `move_up(&mut self)`: wrapping decrement over `filtered_packages().len()`; no-op when empty
    - `update_filter(&mut self, new_filter: String)`: sets `self.filter = new_filter` and resets `self.selected = 0`
    - _Requirements: 3.2, 3.3, 3.4, 3.5, 4.1, 4.2, 4.3, 4.4, 4.5_

  - [x] 2.3 Write property test for `filtered_packages` (Property 2)
    - **Property 2: Filtered list is a case-insensitive substring match**
    - **Validates: Requirements 3.2, 3.3**

  - [x] 2.4 Write property test for `update_filter` (Property 3)
    - **Property 3: Filter change resets selected index to zero**
    - **Validates: Requirements 3.4**

  - [x] 2.5 Write property test for `move_down` / `move_up` (Property 5)
    - **Property 5: Navigation wraps correctly within the filtered list**
    - **Validates: Requirements 4.1, 4.2, 4.3, 4.4, 4.5**

  - [x] 2.6 Add `AppMode::CratePicker(CratePickerState)` variant to the `AppMode` enum
    - _Requirements: 1.3_

  - [x] 2.7 Extend `clone_action` to handle `PickCrate`
    - Add arm: `CommandAction::PickCrate(inner) => CommandAction::PickCrate(Box::new(clone_action(inner)))`
    - _Requirements: 8.2_

- [x] 3. Add `launch_metadata_for_crate_picker` and `CratePicker` event handling to `src/app.rs`
  - [x] 3.1 Add `launch_metadata_for_crate_picker` method (mirrors `launch_metadata_for_dep_browser`)
    - Spawns `CargoCommand::Metadata` without changing `self.mode`
    - Returns early if `self.runner.is_some()`
    - _Requirements: 1.4_

  - [x] 3.2 Add `CommandAction::PickCrate` arm in the `AppMode::Menu` Enter handler
    - Clears `metadata_buf` and `stderr_buf`
    - Sets `self.mode = AppMode::CratePicker(CratePickerState { packages: vec![], filter: String::new(), selected: 0, status: CratePickerStatus::Loading, pending_action: inner })`
    - Sets `self.pending_command = Some(CargoCommand::Metadata)`
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 8.2_

  - [x] 3.3 Add `AppMode::CratePicker` arm in `handle_event`
    - `OutputChunk::Stdout` / `Stderr`: accumulate into `metadata_buf` / `stderr_buf`
    - `OutputChunk::Done(status)`: on failure set `status = Error(stderr)`; on success parse metadata, sort packages alphabetically, set `status = Loaded`
    - `Key(Esc)`: set `self.mode = AppMode::Menu`, no `pending_command`
    - `Key(Down | 'j')`: call `state.move_down()` (only when `Loaded`)
    - `Key(Up | 'k')`: call `state.move_up()` (only when `Loaded`)
    - Printable char: call `state.update_filter(filter + char)` (only when `Loaded`)
    - `Key(Backspace)`: call `state.update_filter(filter minus last char)` (only when `Loaded`)
    - `Key(Enter)` with non-empty filtered list: resolve `pending_action` via `apply_input_to_action`, transition to `Menu` (with `pending_command`) or `Input` as appropriate
    - `Key(Enter)` with empty filtered list: no-op
    - _Requirements: 1.5, 2.1, 2.2, 2.3, 2.4, 3.1, 3.2, 3.3, 3.4, 3.5, 4.1, 4.2, 4.3, 4.4, 4.5, 5.1, 5.2, 5.3, 5.4, 6.1, 6.2_

  - [x] 3.4 Write property test for sorted package list on load (Property 1)
    - **Property 1: Package list is sorted alphabetically**
    - **Validates: Requirements 2.3**

  - [x] 3.5 Write property test for Enter resolving pending action (Property 6)
    - **Property 6: Enter resolves the pending action with the selected package name**
    - **Validates: Requirements 5.1, 5.2**

- [x] 4. Checkpoint — ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 5. Create `src/ui/crate_picker.rs` with `render_crate_picker`
  - Render a centered overlay using `frame.render_widget(Clear, area)` first
  - Layout: filter input row at top, scrollable package list in the middle, hint row at bottom (`Enter: Select   Esc: Cancel`)
  - Show "Loading…" when `status == Loading`
  - Show error message when `status == Error(_)`
  - Show "No packages found" when `status == Loaded` and `filtered_packages()` is empty
  - Highlight selected item with yellow foreground + bold (consistent with `dep_browser.rs`)
  - Scroll list so selected item is always visible (use `ListState`)
  - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5, 1.5, 2.4_

- [x] 6. Wire `CratePicker` into `src/ui/mod.rs` and `src/ui/status_bar.rs`
  - [x] 6.1 Add `pub mod crate_picker;` and import `render_crate_picker` in `src/ui/mod.rs`
    - In the `render` function, add a `CratePicker` overlay arm (after the existing `Confirm` arm) that computes a centered rect and calls `render_crate_picker`
    - _Requirements: 7.1_

  - [x] 6.2 Add `CratePicker` arms to `mode_name` and `mode_hints` in `src/ui/status_bar.rs`
    - `mode_name`: return `"Crate Picker"`
    - `mode_hints`: match on `state.status` — `Loading` → `"Loading…"`, `Loaded` → `"↑↓/jk: Navigate  Enter: Select  Esc: Cancel"`, `Error(_)` → `"Esc: Cancel"`
    - _Requirements: 7.4_

- [x] 7. Extend `main.rs` pending_command loop for `CratePicker`
  - Add `else if matches!(app.mode, AppMode::CratePicker(_))` branch that calls `app.launch_metadata_for_crate_picker(&root, output_tx).await`
  - _Requirements: 1.4_

- [x] 8. Final checkpoint — ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Property tests use the `proptest` crate (already a dependency)
- Each property test should be tagged with its property number and the requirements clause it validates
- `filtered_packages()` is a pure computed view — no separate cached `Vec` to keep state minimal
