# Implementation Plan: cargotui

## Overview

Incremental implementation of a Rust TUI application wrapping Cargo. Each task builds on the previous, wiring components together progressively. The app uses `ratatui` for rendering, `crossterm` for terminal I/O, `tokio` for async subprocess streaming, and `proptest` for property-based tests.

## Tasks

- [x] 1. Project scaffolding and dependencies
  - Add `ratatui`, `crossterm`, `tokio` (full features), `toml`, `serde`, `serde_json`, `proptest` to `Cargo.toml`
  - Create the module skeleton: `src/app.rs`, `src/event.rs`, `src/ui/mod.rs`, `src/ui/menu.rs`, `src/ui/output.rs`, `src/ui/status_bar.rs`, `src/ui/input.rs`, `src/ui/help.rs`, `src/cargo/mod.rs`, `src/cargo/runner.rs`, `src/cargo/workspace.rs`, `src/cargo/metadata.rs`
  - Declare all modules in `main.rs` so the project compiles (stub implementations with `todo!()`)
  - _Requirements: all_

- [x] 2. Workspace detection
  - [x] 2.1 Implement `cargo/workspace.rs`: walk parent dirs from `current_dir()` to find `Cargo.toml`, parse it with `toml` crate into `Workspace` / `Manifest` structs (members, dependencies by section, binary targets)
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 6.7_
  - [x] 2.2 Write property test for workspace ancestor detection (Property 1)
    - **Property 1: Workspace detection finds Cargo.toml in any ancestor**
    - **Validates: Requirements 1.1, 1.2**
    - Use `proptest` to generate random dir depths, create a temp dir tree, place `Cargo.toml` at a random ancestor, assert detection succeeds
    - Tag: `// Feature: cargo-tui, Property 1: Workspace detection finds ancestor`
  - [x] 2.3 Write property test for missing workspace error (Property 2)
    - **Property 2: Missing workspace produces error**
    - **Validates: Requirements 1.3**
    - Generate paths with no `Cargo.toml` anywhere, assert the function returns `Err`
    - Tag: `// Feature: cargo-tui, Property 2: Missing workspace produces error`
  - [x] 2.4 Write property test for dependency grouping (Property 12)
    - **Property 12: Dependency list reflects manifest sections**
    - **Validates: Requirements 6.7**
    - Generate random `Cargo.toml` content with arbitrary dependency sections, assert the union of all three groups equals the full declared set
    - Tag: `// Feature: cargo-tui, Property 12: Dependency grouping`

- [x] 3. Cargo command definitions and argv serialization
  - [x] 3.1 Implement `cargo/mod.rs`: define `CargoCommand` enum (all variants from design), `CommandNode` / `CommandAction` tree, and a `to_argv() -> Vec<OsString>` method on `CargoCommand`
    - _Requirements: 3.1–3.4, 4.1, 4.3, 4.4, 5.1, 6.1, 6.2, 6.4, 7.2–7.4, 8.1–8.7_
  - [x] 3.2 Write property test for CargoCommand argv correctness (Property 3)
    - **Property 3: CargoCommand serializes to correct argv**
    - **Validates: Requirements 3.1–3.4, 4.1, 4.3, 4.4, 5.1, 6.1, 6.2, 6.4, 7.2–7.4, 8.1–8.7**
    - Use `proptest` to generate arbitrary `CargoCommand` variants, assert `to_argv()[0] == "cargo"` and all expected flags are present
    - Tag: `// Feature: cargo-tui, Property 3: CargoCommand argv correctness`

- [x] 4. Terminal setup and event loop skeleton
  - [x] 4.1 Implement `main.rs`: `TerminalGuard` struct with `Drop` impl calling `disable_raw_mode` + `LeaveAlternateScreen`; enter alternate screen, enable raw mode, create `tokio` runtime, run event loop
    - _Requirements: 11.2, 11.4_
  - [x] 4.2 Implement `event.rs`: `Event` enum (`Key`, `Resize`, `Output`, `Tick`); background task that polls `crossterm::event::poll` and an `mpsc` receiver for subprocess chunks, forwarding both to the main loop channel
    - _Requirements: 9.1, 11.3_

- [x] 5. App state and mode machine
  - [x] 5.1 Implement `app.rs`: `App` struct with all fields from design, `AppMode` enum; `App::new(workspace)` constructor; `App::handle_event(&mut self, event: Event)` stub that dispatches by mode
    - _Requirements: 2.4, 2.5, 12.1_
  - [x] 5.2 Write property test for menu stack round-trip (Property 9)
    - **Property 9: Menu stack back-navigation is a round trip**
    - **Validates: Requirements 2.5**
    - Generate random submenu depths N, push N levels, pop N times with Escape, assert stack depth equals initial depth
    - Tag: `// Feature: cargo-tui, Property 9: Menu stack round trip`
  - [x] 5.3 Write property test for concurrent command prevention (Property 10)
    - **Property 10: Concurrent command prevention**
    - **Validates: Requirements 3.7**
    - Set `runner` to `Some(...)`, attempt to launch a second command, assert `runner` handle is unchanged
    - Tag: `// Feature: cargo-tui, Property 10: Concurrent command prevention`

- [x] 6. Checkpoint — ensure project compiles and all existing tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 7. Command runner (subprocess streaming)
  - [x] 7.1 Implement `cargo/runner.rs`: `spawn_cargo` async function that builds a `tokio::process::Command` from `CargoCommand::to_argv()`, spawns it in the workspace root, reads stdout/stderr line-by-line, sends `OutputChunk` variants over `mpsc`, sends `OutputChunk::Done(exit_status)` on completion; returns `RunnerHandle` with kill channel
    - _Requirements: 3.5, 3.6, 4.5, 5.4, 9.1, 12.4_
  - [x] 7.2 Wire `spawn_cargo` into `App::handle_event`: when a `CargoCommand` is dispatched and `runner` is `None`, call `spawn_cargo`, store the `RunnerHandle`, transition to `AppMode::Running`; when `OutputChunk::Done` arrives, clear `runner` and return to `AppMode::Menu`
    - _Requirements: 3.7, 11.1_

- [x] 8. Output buffer
  - [x] 8.1 Implement `OutputBuffer` in `ui/output.rs`: `VecDeque<CommandOutput>` capped at 10, `push_line` method, `scroll_up` / `scroll_down` / `scroll_to_bottom` methods, `auto_scroll` flag toggling
    - _Requirements: 9.2, 9.3, 9.4, 9.5, 9.6_
  - [x] 8.2 Write property test for output buffer max 10 entries (Property 4)
    - **Property 4: Output buffer retains at most 10 entries**
    - **Validates: Requirements 9.5**
    - Generate sequences of N > 10 command completions, assert `history.len() <= 10`
    - Tag: `// Feature: cargo-tui, Property 4: Output buffer max 10 entries`
  - [x] 8.3 Write property test for auto-scroll disables on scroll-up (Property 5)
    - **Property 5: Auto-scroll disables on manual scroll-up**
    - **Validates: Requirements 9.3, 9.4**
    - Generate scroll-up amounts > 0, assert `auto_scroll == false` after the call
    - Tag: `// Feature: cargo-tui, Property 5: Auto-scroll disables on scroll-up`
  - [x] 8.4 Write property test for auto-scroll round-trip (Property 6)
    - **Property 6: Auto-scroll re-enables on scroll-to-bottom**
    - **Validates: Requirements 9.3, 9.4**
    - Scroll up (disables auto-scroll), then scroll to bottom, assert `auto_scroll == true`
    - Tag: `// Feature: cargo-tui, Property 6: Auto-scroll round trip`

- [x] 9. Input field widget
  - [x] 9.1 Implement `InputState` in `ui/input.rs`: `handle_key` method supporting `Backspace`, `Ctrl+A`, `Ctrl+E`, `Ctrl+U`, character insertion, cursor tracking; `validate` method returning error for empty/whitespace when `required = true`
    - _Requirements: 10.1, 10.2, 10.3, 10.4, 10.5_
  - [x] 9.2 Write property test for cursor invariant (Property 8)
    - **Property 8: Input editing operations preserve cursor invariant**
    - **Validates: Requirements 10.2**
    - Generate random sequences of key operations, assert `cursor <= value.len()` after each operation
    - Tag: `// Feature: cargo-tui, Property 8: Cursor invariant`
  - [x] 9.3 Write property test for empty input rejection (Property 7)
    - **Property 7: Empty/whitespace input is rejected for required fields**
    - **Validates: Requirements 10.5**
    - Generate whitespace-only strings, call `validate` with `required = true`, assert error is `Some`
    - Tag: `// Feature: cargo-tui, Property 7: Empty input rejected`

- [x] 10. Checkpoint — ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 11. UI rendering
  - [x] 11.1 Implement `ui/menu.rs`: render `MenuState` as a bordered list widget; highlight selected item; show command description below the list
    - _Requirements: 2.1, 2.2, 2.3_
  - [x] 11.2 Implement `ui/output.rs` render function: render `OutputBuffer` as a scrollable paragraph; show auto-scroll paused indicator when `auto_scroll == false`; render exit status line styled by success/failure
    - _Requirements: 3.5, 3.6, 9.2, 9.3, 9.4, 11.1_
  - [x] 11.3 Implement `ui/status_bar.rs`: render one-line bar showing workspace root path, current mode, and context-sensitive key bindings from the key binding registry
    - _Requirements: 1.2, 2.6_
  - [x] 11.4 Implement `ui/input.rs` render function: render inline input field with prompt, current value, cursor, placeholder, and optional error message
    - _Requirements: 10.1, 10.5_
  - [x] 11.5 Implement `ui/help.rs`: render a centered overlay listing all key bindings for the current `AppMode`; close on `?`, `Escape`, or `q`
    - _Requirements: 12.2, 12.3_
  - [x] 11.6 Implement `ui/mod.rs` root render function: compose all widgets into the terminal frame using ratatui layout constraints (menu panel, output panel, status bar, optional input/help overlay)
    - _Requirements: 11.3_

- [x] 12. Keyboard event handling (full implementation)
  - [x] 12.1 Complete `App::handle_event` for `AppMode::Menu`: arrow keys / `j`/`k` for navigation, `Enter` to select (execute command, enter submenu, or open input), `Escape`/`q` to go back or exit, `?` to open help
    - _Requirements: 2.4, 2.5, 12.1, 12.5_
  - [x] 12.2 Complete `App::handle_event` for `AppMode::Input`: delegate key events to `InputState::handle_key`; on `Enter` validate and dispatch command or show error; on `Escape` cancel and return to menu
    - _Requirements: 10.2, 10.3, 10.4, 10.5_
  - [x] 12.3 Complete `App::handle_event` for `AppMode::Running`: forward output chunks to `OutputBuffer`; handle `Ctrl+C` to send kill signal without exiting; handle `j`/`k`/arrow keys to scroll output; handle `c` to clear output
    - _Requirements: 3.7, 9.2, 9.6, 12.4_
  - [x] 12.4 Complete `App::handle_event` for `AppMode::Confirm`: `Enter` to confirm and dispatch, `Escape`/`q` to cancel
    - _Requirements: 7.1_
  - [x] 12.5 Handle `Resize` events: store new dimensions in `AppState`, trigger re-render (ratatui handles layout recalculation)
    - _Requirements: 11.3_

- [x] 13. Dependency management wiring
  - [x] 13.1 Wire dependency list display: after any `Add`, `Remove`, or `Update` command completes, re-parse the `Cargo.toml` manifest and refresh `App::workspace`; display the updated dependency list grouped by section in the menu or a dedicated panel
    - _Requirements: 6.3, 6.5, 6.6, 6.7_

- [x] 14. Metadata and cargo tree display
  - [x] 14.1 Implement `cargo/metadata.rs`: parse `cargo metadata --format-version 1` JSON output into a displayable tree structure; render it in the output panel
    - _Requirements: 8.1_

- [x] 15. Integration wiring and startup flow
  - [x] 15.1 Complete `main.rs` startup: call workspace detection, handle the no-workspace error (print to stderr, exit code 1), construct `App`, enter the event loop
    - _Requirements: 1.1, 1.2, 1.3_
  - [x] 15.2 Wire the full event loop: `tokio::select!` over terminal events and subprocess output channel, call `App::handle_event`, call `ui::render`, flush the terminal frame
    - _Requirements: 9.1_

- [x] 16. Final checkpoint — ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for a faster MVP
- Each task references specific requirements for traceability
- Property tests use `proptest` with default 256 iterations (satisfies the 100-iteration minimum)
- Each property test must include the tag comment: `// Feature: cargo-tui, Property <N>: <text>`
- The `TerminalGuard` drop pattern ensures terminal restoration on all exit paths including panics
