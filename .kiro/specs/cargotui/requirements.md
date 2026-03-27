space path, active command, and key bindings
- **Manifest**: The `Cargo.toml` file describing the Rust package or workspace

---

## Requirements

### Requirement 1: Project Workspace Detection

**User Story:** As a developer, I want the App to detect and display the current Rust workspace, so that I know which project I am managing.

#### Acceptance Criteria

1. WHEN the App starts, THE App SHALL search for a `Cargo.toml` file in the current working directory and its parent directories up to the filesystem root.
2. WHEN a `Cargo.toml` is found, THE App SHALL display the workspace root path in the Status_Bar.
3. IF no `Cargo.toml` is found in the current directory or any parent directory, THEN THE App SHALL display an error message indicating no Rust workspace was detected and exit with a non-zero status code.
4. WHEN the App starts in a Cargo workspace with multiple members, THE App SHALL detect and list all workspace member crates.

---

### Requirement 2: Command Menu Navigation

**User Story:** As a developer, I want to browse all available Cargo commands through a navigable menu, so that I do not need to memorize command names or flags.

#### Acceptance Criteria

1. THE Command_Menu SHALL display all top-level Cargo commands grouped by category (Build, Test, Dependencies, Publish, Toolchain, Utilities).
2. WHEN the user navigates to a command in the Command_Menu, THE Command_Menu SHALL display a short description of the selected command.
3. WHEN the user selects a command that accepts subcommands or flags, THE Command_Menu SHALL present a secondary menu listing the available options.
4. THE App SHALL support keyboard navigation of the Command_Menu using arrow keys, `j`/`k` for vim-style movement, and `Enter` to select.
5. THE App SHALL support pressing `Escape` or `q` to go back to the previous menu level or exit the App when at the top level.
6. THE Status_Bar SHALL display the key bindings available in the current context at all times.

---

### Requirement 3: Build Commands

**User Story:** As a developer, I want to run Cargo build commands from the TUI, so that I can compile my project without leaving the terminal interface.

#### Acceptance Criteria

1. WHEN the user selects the build command, THE Command_Runner SHALL execute `cargo build` in the detected Workspace root.
2. WHEN the user selects the release build option, THE Command_Runner SHALL execute `cargo build --release` in the detected Workspace root.
3. WHEN the user selects the check command, THE Command_Runner SHALL execute `cargo check` in the detected Workspace root.
4. WHEN the user selects the clean command, THE Command_Runner SHALL execute `cargo clean` in the detected Workspace root.
5. WHEN a build command is executing, THE Output_Panel SHALL stream stdout and stderr output in real time.
6. WHEN a build command completes, THE Output_Panel SHALL display the exit status and indicate success or failure.
7. WHILE a command is executing, THE App SHALL prevent launching a second concurrent Cargo command.

---

### Requirement 4: Test Commands

**User Story:** As a developer, I want to run Cargo test commands from the TUI, so that I can execute and review test results interactively.

#### Acceptance Criteria

1. WHEN the user selects the test command, THE Command_Runner SHALL execute `cargo test` in the detected Workspace root.
2. WHEN the user selects a specific test filter option, THE App SHALL present an input field for the test name filter string, and THE Command_Runner SHALL execute `cargo test <filter>` with the provided string.
3. WHEN the user selects the doc-test option, THE Command_Runner SHALL execute `cargo test --doc` in the detected Workspace root.
4. WHEN the user selects the benchmark command, THE Command_Runner SHALL execute `cargo bench` in the detected Workspace root.
5. WHEN a test command is executing, THE Output_Panel SHALL stream test output in real time.
6. WHEN a test command completes, THE Output_Panel SHALL display a summary of passed, failed, and ignored test counts parsed from Cargo's output.

---

### Requirement 5: Run Command

**User Story:** As a developer, I want to run my Rust binary from the TUI, so that I can execute my project without switching to a separate terminal.

#### Acceptance Criteria

1. WHEN the user selects the run command, THE Command_Runner SHALL execute `cargo run` in the detected Workspace root.
2. WHEN the Manifest defines multiple binary targets, THE App SHALL present a selection list of available binaries, and THE Command_Runner SHALL execute `cargo run --bin <name>` for the selected binary.
3. WHEN the user provides additional arguments for the binary, THE App SHALL present an input field, and THE Command_Runner SHALL pass the arguments after `--` to the cargo run invocation.
4. WHEN a run command is executing, THE Output_Panel SHALL stream stdout and stderr in real time.

---

### Requirement 6: Dependency Management

**User Story:** As a developer, I want to add, remove, and update dependencies through the TUI, so that I can manage my project's dependencies without editing `Cargo.toml` manually.

#### Acceptance Criteria

1. WHEN the user selects the add dependency option, THE App SHALL present an input field for the crate name, and THE Dependency_Manager SHALL execute `cargo add <crate>` with the provided name.
2. WHEN the user selects the add dependency option and specifies a version, THE Dependency_Manager SHALL execute `cargo add <crate>@<version>`.
3. WHEN the user selects the remove dependency option, THE App SHALL present a list of current dependencies parsed from the Manifest, and THE Dependency_Manager SHALL execute `cargo remove <crate>` for the selected dependency.
4. WHEN the user selects the update command, THE Dependency_Manager SHALL execute `cargo update` in the detected Workspace root.
5. WHEN the user selects the update command for a specific crate, THE App SHALL present a list of current dependencies, and THE Dependency_Manager SHALL execute `cargo update <crate>` for the selected dependency.
6. WHEN a dependency operation completes, THE App SHALL refresh the dependency list displayed in the TUI to reflect the current state of the Manifest.
7. THE Dependency_Manager SHALL parse the `[dependencies]`, `[dev-dependencies]`, and `[build-dependencies]` sections of the Manifest and display them grouped by section.

---

### Requirement 7: Publish and Registry Commands

**User Story:** As a developer, I want to publish my crate and manage registry interactions from the TUI, so that I can release my crate without memorizing publish flags.

#### Acceptance Criteria

1. WHEN the user selects the publish command, THE App SHALL display a confirmation prompt before executing.
2. WHEN the user confirms the publish action, THE Command_Runner SHALL execute `cargo publish` in the detected Workspace root.
3. WHEN the user selects the dry-run publish option, THE Command_Runner SHALL execute `cargo publish --dry-run` in the detected Workspace root.
4. WHEN the user selects the package command, THE Command_Runner SHALL execute `cargo package` to create a `.crate` archive without publishing.
5. WHEN the user selects the login command, THE App SHALL present an input field for the registry token, and THE Command_Runner SHALL execute `cargo login` with the provided token.
6. WHEN the user selects the yank command, THE App SHALL present input fields for the crate name and version, and THE Command_Runner SHALL execute `cargo yank --version <version> <crate>`.

---

### Requirement 8: Toolchain and Metadata Commands

**User Story:** As a developer, I want to access Cargo metadata and toolchain commands from the TUI, so that I can inspect and manage my project configuration.

#### Acceptance Criteria

1. WHEN the user selects the metadata command, THE Command_Runner SHALL execute `cargo metadata --format-version 1` and THE App SHALL display the parsed JSON output in a readable tree structure.
2. WHEN the user selects the doc command, THE Command_Runner SHALL execute `cargo doc` in the detected Workspace root.
3. WHEN the user selects the doc command with the open option, THE Command_Runner SHALL execute `cargo doc --open`.
4. WHEN the user selects the fmt command, THE Command_Runner SHALL execute `cargo fmt` in the detected Workspace root.
5. WHEN the user selects the clippy command, THE Command_Runner SHALL execute `cargo clippy` in the detected Workspace root.
6. WHEN the user selects the fix command, THE Command_Runner SHALL execute `cargo fix` in the detected Workspace root.
7. WHEN the user selects the tree command, THE Command_Runner SHALL execute `cargo tree` and THE Output_Panel SHALL display the dependency tree output.

---

### Requirement 9: Real-Time Output Display

**User Story:** As a developer, I want to see live output from running Cargo commands, so that I can monitor progress and diagnose issues without waiting for the command to finish.

#### Acceptance Criteria

1. WHILE a Cargo command is executing, THE Output_Panel SHALL update with new output lines within 100ms of the Command_Runner receiving them from the subprocess.
2. THE Output_Panel SHALL support scrolling through output history using arrow keys or `j`/`k`.
3. WHEN the Output_Panel content exceeds the visible area, THE Output_Panel SHALL automatically scroll to the latest output line unless the user has manually scrolled up.
4. WHEN the user manually scrolls up in the Output_Panel, THE App SHALL pause auto-scroll and display a visual indicator that auto-scroll is paused.
5. THE Output_Panel SHALL retain the output of the last 10 completed commands, accessible via a command history selector.
6. WHEN the user presses `c` while the Output_Panel is focused, THE App SHALL clear the current output.

---

### Requirement 10: Input Fields and Forms

**User Story:** As a developer, I want contextual input fields when a command requires arguments, so that I can provide parameters without leaving the TUI.

#### Acceptance Criteria

1. WHEN a Cargo command requires a text argument, THE App SHALL display an inline input field within the TUI without spawning an external editor.
2. WHEN an input field is active, THE App SHALL support standard text editing keys: `Backspace` to delete, `Ctrl+A` to move to start, `Ctrl+E` to move to end, and `Ctrl+U` to clear the field.
3. WHEN the user presses `Enter` in an input field, THE App SHALL submit the value and proceed with command execution.
4. WHEN the user presses `Escape` in an input field, THE App SHALL cancel the input and return to the Command_Menu without executing the command.
5. IF the user submits an empty value for a required argument, THEN THE App SHALL display an inline validation error and keep the input field active.

---

### Requirement 11: Error Handling

**User Story:** As a developer, I want the App to handle errors gracefully, so that failures in Cargo commands or the TUI itself do not cause data loss or a broken terminal state.

#### Acceptance Criteria

1. WHEN a Cargo command exits with a non-zero status code, THE App SHALL display the exit code and the full stderr output in the Output_Panel.
2. WHEN the App encounters an unrecoverable internal error, THE App SHALL restore the terminal to its original state before exiting.
3. IF the terminal is resized while the App is running, THEN THE App SHALL re-render the TUI layout to fit the new terminal dimensions within 50ms.
4. WHEN the App exits for any reason, THE App SHALL restore the raw terminal mode and alternate screen state so the user's shell is not left in a broken state.

---

### Requirement 12: Keyboard-Driven Interaction

**User Story:** As a developer, I want to control the entire App using only the keyboard, so that I can stay in a flow state without reaching for the mouse.

#### Acceptance Criteria

1. THE App SHALL be fully operable using only keyboard input with no mouse interaction required.
2. THE App SHALL support a global `?` key that opens a help overlay listing all key bindings for the current context.
3. WHEN the help overlay is open, THE App SHALL close it when the user presses `?`, `Escape`, or `q`.
4. THE App SHALL support a global `Ctrl+C` key binding that cancels the currently running Cargo command without exiting the App.
5. WHEN no Cargo command is running and the user presses `Ctrl+C`, THE App SHALL exit cleanly.
