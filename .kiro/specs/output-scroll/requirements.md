# Requirements Document

## Introduction

This feature adds a focusable output panel to the TUI application. Currently the output panel on the right side of the UI is always visible but cannot be independently focused or scrolled when a command is not actively running. The user wants to press Tab to shift keyboard focus to the output panel, then use arrow keys or j/k to scroll through past command output. Pressing Tab again (or Esc) returns focus to the menu panel.

## Glossary

- **App**: The top-level application struct managing all state and event routing.
- **OutputPanel**: The right-hand panel that displays command output, rendered by `render_output`.
- **OutputBuffer**: The data structure (`OutputBuffer`) holding command output history and scroll state.
- **FocusedPanel**: The currently keyboard-focused panel — either `Menu` or `Output`.
- **MenuPanel**: The left-hand panel containing the cargo command menu.
- **ScrollOffset**: The number of lines from the top of the output that have been scrolled past.

## Requirements

### Requirement 1: Panel Focus Toggle

**User Story:** As a developer, I want to press Tab to move keyboard focus between the menu panel and the output panel, so that I can scroll through command output without leaving the application.

#### Acceptance Criteria

1. WHEN the user presses Tab and `FocusedPanel` is `Menu`, THE App SHALL set `FocusedPanel` to `Output`.
2. WHEN the user presses Tab and `FocusedPanel` is `Output`, THE App SHALL set `FocusedPanel` to `Menu`.
3. THE App SHALL initialize with `FocusedPanel` set to `Menu`.
4. WHEN `FocusedPanel` is `Output` and the user presses Esc, THE App SHALL set `FocusedPanel` to `Menu`.

### Requirement 2: Output Panel Scrolling

**User Story:** As a developer, I want to scroll through command output using arrow keys or j/k when the output panel is focused, so that I can review earlier lines of output.

#### Acceptance Criteria

1. WHEN `FocusedPanel` is `Output` and the user presses Down or `j`, THE OutputBuffer SHALL increase `ScrollOffset` by 1, clamped to the maximum scrollable position.
2. WHEN `FocusedPanel` is `Output` and the user presses Up or `k`, THE OutputBuffer SHALL decrease `ScrollOffset` by 1, clamped to a minimum of 0.
3. WHEN `FocusedPanel` is `Output` and the user presses Up or `k`, THE OutputBuffer SHALL set `auto_scroll` to `false`.
4. WHILE `FocusedPanel` is `Menu`, THE App SHALL NOT route Up, Down, `j`, or `k` key events to the OutputBuffer.

### Requirement 3: Visual Focus Indicator

**User Story:** As a developer, I want a visual indicator showing which panel is focused, so that I know where keyboard input is directed.

#### Acceptance Criteria

1. WHEN `FocusedPanel` is `Output`, THE OutputPanel SHALL render its border with a highlighted style distinct from the unfocused style.
2. WHEN `FocusedPanel` is `Menu`, THE MenuPanel SHALL render its border with a highlighted style distinct from the unfocused style.
3. THE OutputPanel SHALL render its border in the default (unfocused) style WHEN `FocusedPanel` is `Menu`.
4. THE MenuPanel SHALL render its border in the default (unfocused) style WHEN `FocusedPanel` is `Output`.

### Requirement 4: Focus Preserved Across Modes

**User Story:** As a developer, I want focus to return to the menu automatically when a modal overlay appears, so that keyboard input is never silently swallowed by an invisible panel.

#### Acceptance Criteria

1. WHEN the App transitions to `AppMode::Input`, `AppMode::Help`, `AppMode::Confirm`, or `AppMode::CratePicker`, THE App SHALL set `FocusedPanel` to `Menu`.
2. WHEN the App transitions to `AppMode::Running`, THE App SHALL set `FocusedPanel` to `Output`.
3. WHEN `AppMode::Running` completes and the App returns to `AppMode::Menu`, THE App SHALL set `FocusedPanel` to `Menu`.

### Requirement 5: Scrolling During Active Command

**User Story:** As a developer, I want to scroll through output while a command is still running, so that I can review earlier lines without waiting for the command to finish.

#### Acceptance Criteria

1. WHILE `AppMode` is `Running` and `FocusedPanel` is `Output`, THE OutputBuffer SHALL respond to Up/`k` and Down/`j` key events for scrolling.
2. WHEN `auto_scroll` is `false` and a new output line arrives, THE OutputBuffer SHALL NOT change `ScrollOffset`.
3. WHEN `auto_scroll` is `true` and a new output line arrives, THE OutputBuffer SHALL update `ScrollOffset` to keep the latest line visible.
