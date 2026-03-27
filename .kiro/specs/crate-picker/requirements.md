# Requirements Document

## Introduction

The crate-picker feature replaces plain text input prompts for commands that operate on existing workspace dependencies (e.g. `cargo remove`, `cargo update <crate>`) with an interactive picker overlay. The picker loads the workspace package list via `cargo metadata`, lets the user type to filter results, navigate with arrow keys, confirm with Enter, and cancel with Esc. This reuses the existing `cargo metadata` infrastructure already present in the `DepBrowser` component.

## Glossary

- **CratePicker**: The interactive overlay widget that presents a filterable, scrollable list of workspace packages for selection.
- **CratePickerState**: The application state struct that holds the package list, current filter text, selected index, loading status, and the pending command action awaiting the selected crate name.
- **PackageInfo**: An existing struct in `cargo::metadata` representing a single workspace package with `name` and `version` fields.
- **DepBrowser**: The existing dependency docs browser component that loads packages via `cargo metadata` and displays them in a scrollable list.
- **PendingAction**: The `CommandAction` that will be resolved once the user selects a crate name from the CratePicker.
- **FilteredList**: The subset of packages whose names contain the current filter text as a case-insensitive substring.
- **AppMode**: The existing enum in `app.rs` that tracks the current UI mode of the application.
- **Workspace**: The existing struct representing the resolved Cargo workspace.

## Requirements

### Requirement 1: CratePicker Activation

**User Story:** As a developer, I want commands that require an existing crate name to open an interactive picker instead of a plain text box, so that I can select from my actual workspace dependencies without typing from memory.

#### Acceptance Criteria

1. WHEN the user selects a menu action that requires an existing workspace crate name as input, THE CratePicker SHALL open as an overlay displaying the workspace package list.
2. THE CratePicker SHALL be activated for `cargo remove <crate>` and `cargo update <crate>` commands.
3. WHEN the CratePicker is activated, THE App SHALL set `AppMode` to `AppMode::CratePicker(CratePickerState)`.
4. WHEN the CratePicker is activated, THE App SHALL initiate a `cargo metadata` command to load the current workspace package list.
5. WHILE the `cargo metadata` command is running, THE CratePicker SHALL display a loading indicator to the user.

### Requirement 2: Package List Loading

**User Story:** As a developer, I want the crate picker to show my actual workspace dependencies, so that I only see relevant options.

#### Acceptance Criteria

1. WHEN `cargo metadata` completes successfully, THE CratePicker SHALL populate its package list from the parsed `MetadataTree`.
2. WHEN `cargo metadata` fails, THE CratePicker SHALL display the error message returned by the metadata command.
3. THE CratePicker SHALL sort the package list alphabetically by package name before displaying it.
4. WHEN the package list is empty after a successful load, THE CratePicker SHALL display a "No packages found" message.

### Requirement 3: Filter Input

**User Story:** As a developer, I want to type to filter the package list, so that I can quickly narrow down to the crate I need.

#### Acceptance Criteria

1. THE CratePicker SHALL display a text input field at the top of the overlay for entering a filter string.
2. WHEN the user types a character, THE CratePicker SHALL update the FilteredList to include only packages whose names contain the typed string as a case-insensitive substring.
3. WHEN the filter string is empty, THE CratePicker SHALL display all packages in the list.
4. WHEN the filter string changes, THE CratePicker SHALL reset the selected index to 0.
5. WHEN the user presses Backspace, THE CratePicker SHALL remove the last character from the filter string and update the FilteredList.

### Requirement 4: Keyboard Navigation

**User Story:** As a developer, I want to navigate the filtered list with arrow keys, so that I can select a crate without leaving the keyboard.

#### Acceptance Criteria

1. WHEN the user presses the Down arrow key or `j`, THE CratePicker SHALL move the selection to the next item in the FilteredList.
2. WHEN the user presses the Up arrow key or `k`, THE CratePicker SHALL move the selection to the previous item in the FilteredList.
3. WHILE the selected index is at the last item in the FilteredList and the user presses Down, THE CratePicker SHALL wrap the selection to the first item.
4. WHILE the selected index is at the first item in the FilteredList and the user presses Up, THE CratePicker SHALL wrap the selection to the last item.
5. WHILE the FilteredList is empty, THE CratePicker SHALL not change the selected index in response to navigation keys.

### Requirement 5: Selection Confirmation

**User Story:** As a developer, I want to confirm my selection with Enter, so that the chosen crate name is passed to the pending command.

#### Acceptance Criteria

1. WHEN the user presses Enter and the FilteredList is non-empty, THE CratePicker SHALL resolve the PendingAction by substituting the selected package name into the command.
2. WHEN the PendingAction resolves to a `CommandAction::Execute`, THE App SHALL set `pending_command` to the resolved `CargoCommand` and return to `AppMode::Menu`.
3. WHEN the PendingAction resolves to another `CommandAction::RequiresInput`, THE App SHALL transition to `AppMode::Input` with the next input prompt.
4. WHEN the user presses Enter and the FilteredList is empty, THE CratePicker SHALL take no action and remain open.

### Requirement 6: Cancellation

**User Story:** As a developer, I want to cancel the picker with Esc, so that I can return to the menu without executing any command.

#### Acceptance Criteria

1. WHEN the user presses Esc, THE CratePicker SHALL close and THE App SHALL return to `AppMode::Menu`.
2. WHEN the CratePicker closes via Esc, THE App SHALL not set any `pending_command`.

### Requirement 7: Rendering

**User Story:** As a developer, I want the crate picker to be visually clear and consistent with the rest of the TUI, so that I can use it without confusion.

#### Acceptance Criteria

1. THE CratePicker SHALL render as a centered overlay on top of the existing UI, consistent with the style used by the `Input` and `Confirm` overlays.
2. THE CratePicker SHALL display the filter input field above the package list within the overlay.
3. THE CratePicker SHALL highlight the currently selected item in the FilteredList using a distinct style (e.g. yellow foreground, bold).
4. THE CratePicker SHALL display a keybinding hint showing "Enter: Select   Esc: Cancel" within the overlay.
5. WHEN the FilteredList has more items than the visible area, THE CratePicker SHALL scroll the list so the selected item is always visible.

### Requirement 8: CommandAction Integration

**User Story:** As a developer, I want the existing command tree to route crate-name inputs through the picker, so that the feature works without duplicating command definitions.

#### Acceptance Criteria

1. THE `CommandAction` enum SHALL include a `PickCrate` variant that carries the `PendingAction` to execute after a crate is selected.
2. WHEN the App processes a `CommandAction::PickCrate` from the menu, THE App SHALL transition to `AppMode::CratePicker` with the associated `PendingAction`.
3. THE `remove <crate>` menu entry SHALL use `CommandAction::PickCrate` instead of `CommandAction::RequiresInput`.
4. THE `update <crate>` menu entry SHALL use `CommandAction::PickCrate` instead of `CommandAction::RequiresInput`.
