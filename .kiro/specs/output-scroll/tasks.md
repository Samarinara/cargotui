# Implementation Plan: output-scroll

## Overview

Add keyboard-driven focus management to the TUI so the output panel can be independently focused and scrolled. The change touches three layers: state (`App`), event routing (`handle_event`), and rendering (`render_output`, `render_menu`, `render`).

## Tasks

- [x] 1. Add `FocusedPanel` enum and `focused_panel` field to `App`
  - Define `FocusedPanel` enum (`Menu`, `Output`) in `src/app.rs`
  - Add `pub focused_panel: FocusedPanel` field to the `App` struct
  - Initialize `focused_panel` to `FocusedPanel::Menu` in `App::new`
  - _Requirements: 1.3_

  - [ ]* 1.1 Write unit test: `test_app_initializes_with_menu_focus`
    - Assert `app.focused_panel == FocusedPanel::Menu` after `App::new`
    - _Requirements: 1.3_

- [x] 2. Implement Tab focus toggle and Esc-to-menu in `handle_event`
  - In `AppMode::Menu` key handler, match `KeyCode::Tab` → toggle `focused_panel` between `Menu` and `Output`
  - In `AppMode::Menu` key handler, when `KeyCode::Esc` and `focused_panel == Output`, set `focused_panel = Menu` before normal Esc handling
  - In `AppMode::Running` key handler, match `KeyCode::Tab` → toggle `focused_panel`
  - _Requirements: 1.1, 1.2, 1.4_

  - [ ]* 2.1 Write property test: `prop_tab_toggles_focus_round_trip`
    - **Property 1: Tab toggles focus (round trip)**
    - **Validates: Requirements 1.1, 1.2**

  - [ ]* 2.2 Write property test: `prop_esc_from_output_focus_returns_menu`
    - **Property 2: Esc from Output focus returns to Menu focus**
    - **Validates: Requirements 1.4**

- [x] 3. Gate scroll key routing on `focused_panel` in `AppMode::Menu`
  - In `AppMode::Menu` key handler, route `Up`/`k` and `Down`/`j` to `output.scroll_up`/`scroll_down` when `focused_panel == Output`; keep existing menu navigation when `focused_panel == Menu`
  - _Requirements: 2.1, 2.2, 2.3, 2.4_

  - [ ]* 3.1 Write property test: `prop_scroll_keys_blocked_when_menu_focused`
    - **Property 3: Scroll keys are blocked when Menu is focused**
    - **Validates: Requirements 2.4**

  - [ ]* 3.2 Write property test: `prop_scroll_down_increases_offset_when_output_focused`
    - **Property 4: Scroll down increases offset when Output is focused**
    - **Validates: Requirements 2.1**

  - [ ]* 3.3 Write property test: `prop_scroll_up_decreases_offset_disables_autoscroll`
    - **Property 5: Scroll up decreases offset and disables auto_scroll when Output is focused**
    - **Validates: Requirements 2.2, 2.3**

- [x] 4. Reset `focused_panel` on mode transitions
  - When entering `AppMode::Input`, `AppMode::Help`, `AppMode::Confirm`, or `AppMode::CratePicker`: set `focused_panel = Menu`
  - When entering `AppMode::Running` (via `launch_command` or `pending_command` dispatch): set `focused_panel = Output`
  - When `OutputChunk::Done` returns app to `AppMode::Menu`: set `focused_panel = Menu`
  - _Requirements: 4.1, 4.2, 4.3_

  - [ ]* 4.1 Write unit test: `test_running_mode_sets_output_focus`
    - Assert `focused_panel == Output` after mode transitions to `AppMode::Running`
    - _Requirements: 4.2_

  - [ ]* 4.2 Write unit test: `test_done_event_resets_focus_to_menu`
    - Assert `focused_panel == Menu` after `OutputChunk::Done` is handled
    - _Requirements: 4.3_

  - [ ]* 4.3 Write property test: `prop_modal_transitions_reset_focus_to_menu`
    - **Property 7: Modal transitions reset focus to Menu**
    - **Validates: Requirements 4.1**

- [x] 5. Checkpoint — ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 6. Update `render_output` and `render_menu` to accept a `focused: bool` parameter
  - Change signature of `render_output` in `src/ui/output.rs` to `pub fn render_output(buffer: &OutputBuffer, frame: &mut Frame, area: Rect, focused: bool)`
  - Change signature of `render_menu` in `src/ui/menu.rs` to `pub fn render_menu(menu: &MenuState, frame: &mut Frame, area: Rect, focused: bool)`
  - When `focused == true`, render the panel's outer `Block` border with `Color::Yellow`; otherwise use the default style
  - _Requirements: 3.1, 3.2, 3.3, 3.4_

  - [ ]* 6.1 Write property test: `prop_focused_panel_has_highlighted_border`
    - **Property 6: Focused panel has highlighted border**
    - **Validates: Requirements 3.1, 3.2, 3.3, 3.4**

- [x] 7. Update call sites in `src/ui/mod.rs`
  - Pass `app.focused_panel == FocusedPanel::Output` as the `focused` argument to `render_output`
  - Pass `app.focused_panel == FocusedPanel::Menu` as the `focused` argument to `render_menu`
  - _Requirements: 3.1, 3.2, 3.3, 3.4_

- [x] 8. Verify `OutputBuffer` auto-scroll behavior with new lines
  - Confirm `push_line` already respects `auto_scroll` (no code change needed if already correct)
  - Add property tests to cover the two auto-scroll properties
  - _Requirements: 5.2, 5.3_

  - [ ]* 8.1 Write property test: `prop_autoscroll_off_new_lines_preserve_offset`
    - **Property 8: Auto-scroll off — new lines do not change scroll offset**
    - **Validates: Requirements 5.2**

  - [ ]* 8.2 Write property test: `prop_autoscroll_on_new_lines_keep_bottom`
    - **Property 9: Auto-scroll on — new lines keep latest line visible**
    - **Validates: Requirements 5.3**

- [x] 9. Final checkpoint — ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for a faster MVP
- Each task references specific requirements for traceability
- Property tests use the `proptest` crate already present in `[dev-dependencies]`
- All property tests must include the comment `// Feature: output-scroll, Property N: <property text>`
