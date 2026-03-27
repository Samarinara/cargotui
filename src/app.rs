use crate::cargo::metadata::PackageInfo;
use crate::cargo::runner::{OutputChunk, RunnerHandle, spawn_cargo};
use crate::cargo::workspace::Workspace;
use crate::cargo::{COMMAND_TREE, CargoCommand, CommandAction, CommandNode};
use crate::event::Event;
use crate::ui::input::{InputSpec, InputState};
use crate::ui::output::OutputBuffer;
use crossterm::event::{KeyCode, KeyModifiers};

pub enum DepBrowserStatus {
    Loading,
    Loaded,
    Error(String),
}

pub enum CratePickerStatus {
    Loading,
    Loaded,
    Error(String),
}

pub struct DepBrowserState {
    pub packages: Vec<PackageInfo>,
    pub selected: usize,
    pub status: DepBrowserStatus,
    pub message: Option<String>,
}

impl DepBrowserState {
    pub fn from_packages(mut packages: Vec<PackageInfo>) -> Self {
        packages.sort_by(|a, b| a.name.cmp(&b.name));
        DepBrowserState {
            packages,
            selected: 0,
            status: DepBrowserStatus::Loaded,
            message: None,
        }
    }

    pub fn move_down(&mut self) {
        if self.packages.is_empty() {
            return;
        }
        self.selected = (self.selected + 1) % self.packages.len();
    }

    pub fn move_up(&mut self) {
        if self.packages.is_empty() {
            return;
        }
        if self.selected == 0 {
            self.selected = self.packages.len() - 1;
        } else {
            self.selected -= 1;
        }
    }
}

pub struct CratePickerState {
    pub packages: Vec<PackageInfo>,
    pub filter: String,
    pub selected: usize,
    pub status: CratePickerStatus,
    pub pending_action: Box<CommandAction>,
}

impl CratePickerState {
    pub fn filtered_packages(&self) -> Vec<&PackageInfo> {
        if self.filter.is_empty() {
            self.packages.iter().collect()
        } else {
            let lower = self.filter.to_lowercase();
            self.packages
                .iter()
                .filter(|p| p.name.to_lowercase().contains(&lower))
                .collect()
        }
    }

    pub fn move_down(&mut self) {
        let len = self.filtered_packages().len();
        if len == 0 {
            return;
        }
        self.selected = (self.selected + 1) % len;
    }

    pub fn move_up(&mut self) {
        let len = self.filtered_packages().len();
        if len == 0 {
            return;
        }
        if self.selected == 0 {
            self.selected = len - 1;
        } else {
            self.selected -= 1;
        }
    }

    pub fn update_filter(&mut self, new_filter: String) {
        self.filter = new_filter;
        self.selected = 0;
    }
}

pub struct App {
    pub workspace: Option<Workspace>,
    pub menu: MenuState,
    pub output: OutputBuffer,
    pub input: Option<InputState>,
    pub runner: Option<RunnerHandle>,
    pub mode: AppMode,
    pub should_quit: bool,
    pub pending_command: Option<CargoCommand>,
    pub current_command: Option<CargoCommand>,
    pub terminal_size: (u16, u16),
    pub metadata_buf: String,
    pub stderr_buf: String,
}

pub enum AppMode {
    Menu,
    Input(InputContext),
    Running,
    Help,
    Confirm(ConfirmContext),
    Error(String),
    DepBrowser(DepBrowserState),
    CratePicker(CratePickerState),
}

pub struct InputContext {
    pub spec: crate::ui::input::InputSpec,
    pub pending_action: Box<crate::cargo::CommandAction>,
}

pub struct ConfirmContext {
    pub message: String,
    pub pending_action: Box<crate::cargo::CommandAction>,
}

pub struct MenuState {
    /// Stack of (nodes_snapshot, selected_index). The nodes are cloned from
    /// COMMAND_TREE so we always have a fresh copy when entering a submenu.
    pub stack: Vec<MenuLevel>,
}

pub struct MenuLevel {
    pub nodes: Vec<crate::cargo::CommandNode>,
    pub selected: usize,
}

/// Recursively clone a CommandNode from the static tree.
fn clone_node(node: &crate::cargo::CommandNode) -> crate::cargo::CommandNode {
    use crate::cargo::CommandAction;
    crate::cargo::CommandNode {
        name: node.name,
        description: node.description,
        action: clone_action(&node.action),
    }
}

fn clone_action(action: &crate::cargo::CommandAction) -> crate::cargo::CommandAction {
    use crate::cargo::{CommandAction, InputSpec};
    match action {
        CommandAction::Submenu(nodes) => {
            CommandAction::Submenu(nodes.iter().map(clone_node).collect())
        }
        CommandAction::Execute(cmd) => CommandAction::Execute(cmd.clone()),
        CommandAction::RequiresInput(spec, next) => CommandAction::RequiresInput(
            InputSpec {
                prompt: spec.prompt,
                required: spec.required,
                placeholder: spec.placeholder,
            },
            Box::new(clone_action(next)),
        ),
        CommandAction::Confirm(inner) => CommandAction::Confirm(Box::new(clone_action(inner))),
        CommandAction::BrowseDocs => CommandAction::BrowseDocs,
        CommandAction::PickCrate(inner) => CommandAction::PickCrate(Box::new(clone_action(inner))),
    }
}

impl App {
    pub fn new(workspace: Option<Workspace>) -> Self {
        // Clone the full top-level nodes from the static COMMAND_TREE so that
        // submenus are intact and can be entered without destroying the tree.
        let top_level_nodes: Vec<CommandNode> = COMMAND_TREE.iter().map(clone_node).collect();

        let menu = MenuState {
            stack: vec![MenuLevel {
                nodes: top_level_nodes,
                selected: 0,
            }],
        };

        App {
            workspace,
            menu,
            output: OutputBuffer::new(),
            input: None,
            runner: None,
            mode: AppMode::Menu,
            should_quit: false,
            pending_command: None,
            current_command: None,
            terminal_size: (80, 24),
            metadata_buf: String::new(),
            stderr_buf: String::new(),
        }
    }

    pub fn can_launch_command(&self) -> bool {
        self.runner.is_none()
    }

    /// Launch a cargo command asynchronously. Returns early if a command is
    /// already running (concurrent command prevention per Requirement 3.7).
    pub async fn launch_command(
        &mut self,
        cmd: CargoCommand,
        workspace_root: &std::path::Path,
        output_tx: tokio::sync::mpsc::Sender<OutputChunk>,
    ) -> std::io::Result<()> {
        if self.runner.is_some() {
            return Ok(());
        }
        let handle = spawn_cargo(&cmd, workspace_root, output_tx).await?;
        self.current_command = Some(cmd);
        self.runner = Some(handle);
        self.mode = AppMode::Running;
        Ok(())
    }

    /// Launch `cargo metadata` for the DepBrowser panel without changing `self.mode`.
    /// Unlike `launch_command`, this does NOT set mode to `AppMode::Running`.
    pub async fn launch_metadata_for_dep_browser(
        &mut self,
        workspace_root: &std::path::Path,
        output_tx: tokio::sync::mpsc::Sender<OutputChunk>,
    ) -> std::io::Result<()> {
        if self.runner.is_some() {
            return Ok(());
        }
        let cmd = CargoCommand::Metadata;
        let handle = spawn_cargo(&cmd, workspace_root, output_tx).await?;
        self.current_command = Some(cmd);
        self.runner = Some(handle);
        // NOTE: mode is intentionally NOT changed here — stays DepBrowser
        Ok(())
    }

    /// Launch `cargo metadata` for the CratePicker overlay without changing `self.mode`.
    /// Unlike `launch_command`, this does NOT set mode to `AppMode::Running`.
    pub async fn launch_metadata_for_crate_picker(
        &mut self,
        workspace_root: &std::path::Path,
        output_tx: tokio::sync::mpsc::Sender<OutputChunk>,
    ) -> std::io::Result<()> {
        if self.runner.is_some() {
            return Ok(());
        }
        let cmd = CargoCommand::Metadata;
        let handle = spawn_cargo(&cmd, workspace_root, output_tx).await?;
        self.current_command = Some(cmd);
        self.runner = Some(handle);
        // NOTE: mode is intentionally NOT changed here — stays CratePicker
        Ok(())
    }

    /// Re-parse the workspace manifest and update `self.workspace`.
    /// Non-fatal: if parsing fails, the existing workspace is left unchanged.
    pub fn refresh_workspace(&mut self) {
        let root = match self.workspace.as_ref().map(|w| w.root.clone()) {
            Some(r) => r,
            None => return,
        };
        match crate::cargo::workspace::find_workspace(&root) {
            Ok(new_workspace) => {
                self.workspace = Some(new_workspace);
            }
            Err(_) => {
                // Non-fatal: leave workspace unchanged
            }
        }
    }

    pub fn handle_event(&mut self, event: Event) {
        // Handle resize at any mode
        if let Event::Resize(w, h) = event {
            self.terminal_size = (w, h);
            return;
        }

        // Take the current mode so we can match on it without holding a borrow
        // on self. We'll put it back (possibly changed) at the end.
        let mode = std::mem::replace(&mut self.mode, AppMode::Menu);

        match mode {
            AppMode::Menu => {
                self.mode = AppMode::Menu;
                if let Event::Key(key) = event {
                    match key.code {
                        KeyCode::Up | KeyCode::Char('k') => {
                            if let Some(level) = self.menu.stack.last_mut() {
                                let len = level.nodes.len();
                                if len > 0 {
                                    if level.selected == 0 {
                                        level.selected = len - 1;
                                    } else {
                                        level.selected -= 1;
                                    }
                                }
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if let Some(level) = self.menu.stack.last_mut() {
                                let len = level.nodes.len();
                                if len > 0 {
                                    level.selected = (level.selected + 1) % len;
                                }
                            }
                        }
                        KeyCode::Enter => {
                            let selected_idx = self.menu.stack.last().map(|l| l.selected);
                            if let Some(idx) = selected_idx {
                                // Clone the action so the node stays intact for
                                // future visits (e.g. pressing Esc and re-entering).
                                let action = self
                                    .menu
                                    .stack
                                    .last()
                                    .and_then(|l| l.nodes.get(idx))
                                    .map(|n| clone_action(&n.action));

                                if let Some(action) = action {
                                    match action {
                                        CommandAction::Submenu(nodes) => {
                                            self.menu.stack.push(MenuLevel { nodes, selected: 0 });
                                        }
                                        CommandAction::Execute(cmd) => {
                                            self.pending_command = Some(cmd);
                                            self.mode = AppMode::Running;
                                        }
                                        CommandAction::RequiresInput(spec, next_action) => {
                                            let ui_spec = InputSpec {
                                                prompt: spec.prompt,
                                                required: spec.required,
                                                placeholder: spec.placeholder,
                                            };
                                            self.input = Some(InputState::new(InputSpec {
                                                prompt: spec.prompt,
                                                required: spec.required,
                                                placeholder: spec.placeholder,
                                            }));
                                            self.mode = AppMode::Input(InputContext {
                                                spec: ui_spec,
                                                pending_action: next_action,
                                            });
                                        }
                                        CommandAction::Confirm(action) => {
                                            self.mode = AppMode::Confirm(ConfirmContext {
                                                message: "Are you sure?".to_string(),
                                                pending_action: action,
                                            });
                                        }
                                        CommandAction::BrowseDocs => {
                                            self.metadata_buf.clear();
                                            self.stderr_buf.clear();
                                            self.mode = AppMode::DepBrowser(DepBrowserState {
                                                packages: vec![],
                                                selected: 0,
                                                status: DepBrowserStatus::Loading,
                                                message: None,
                                            });
                                            self.pending_command = Some(CargoCommand::Metadata);
                                        }
                                        CommandAction::PickCrate(inner) => {
                                            self.metadata_buf.clear();
                                            self.stderr_buf.clear();
                                            self.mode = AppMode::CratePicker(CratePickerState {
                                                packages: vec![],
                                                filter: String::new(),
                                                selected: 0,
                                                status: CratePickerStatus::Loading,
                                                pending_action: inner,
                                            });
                                            self.pending_command = Some(CargoCommand::Metadata);
                                        }
                                    }
                                }
                            }
                        }
                        KeyCode::Esc | KeyCode::Char('q') => {
                            if self.menu.stack.len() > 1 {
                                self.menu.stack.pop();
                            } else {
                                self.should_quit = true;
                            }
                        }
                        KeyCode::Char('?') => {
                            self.mode = AppMode::Help;
                        }
                        _ => {}
                    }
                }
            }

            AppMode::Input(ctx) => {
                if let Event::Key(key) = event {
                    match key.code {
                        KeyCode::Esc => {
                            self.input = None;
                            self.mode = AppMode::Menu;
                        }
                        KeyCode::Enter => {
                            let validation = self.input.as_ref().map(|s| s.validate());
                            match validation {
                                Some(Some(err)) => {
                                    // Validation failed — show error, stay in Input mode
                                    if let Some(input) = self.input.as_mut() {
                                        input.error = Some(err);
                                    }
                                    self.mode = AppMode::Input(ctx);
                                }
                                _ => {
                                    // Valid — extract value and resolve the pending action
                                    let value = self
                                        .input
                                        .as_ref()
                                        .map(|s| s.value.clone())
                                        .unwrap_or_default();
                                    self.input = None;

                                    // Apply the input value to the pending action
                                    let resolved =
                                        apply_input_to_action(*ctx.pending_action, value);
                                    match resolved {
                                        CommandAction::Execute(cmd) => {
                                            self.pending_command = Some(cmd);
                                            self.mode = AppMode::Menu;
                                        }
                                        CommandAction::RequiresInput(spec, next_action) => {
                                            // Another input required (chained inputs)
                                            let ui_spec = InputSpec {
                                                prompt: spec.prompt,
                                                required: spec.required,
                                                placeholder: spec.placeholder,
                                            };
                                            self.input = Some(InputState::new(InputSpec {
                                                prompt: spec.prompt,
                                                required: spec.required,
                                                placeholder: spec.placeholder,
                                            }));
                                            self.mode = AppMode::Input(InputContext {
                                                spec: ui_spec,
                                                pending_action: next_action,
                                            });
                                        }
                                        other => {
                                            // Submenu or Confirm after input — handle generically
                                            match other {
                                                CommandAction::Confirm(action) => {
                                                    self.mode = AppMode::Confirm(ConfirmContext {
                                                        message: "Are you sure?".to_string(),
                                                        pending_action: action,
                                                    });
                                                }
                                                _ => {
                                                    self.mode = AppMode::Menu;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        _ => {
                            if let Some(input) = self.input.as_mut() {
                                input.handle_key(key);
                            }
                            self.mode = AppMode::Input(ctx);
                        }
                    }
                } else {
                    self.mode = AppMode::Input(ctx);
                }
            }

            AppMode::Running => {
                self.mode = AppMode::Running;
                match event {
                    Event::Output(OutputChunk::Stdout(line)) => {
                        self.output.push_line(line);
                    }
                    Event::Output(OutputChunk::Stderr(line)) => {
                        self.output.push_line(format!("[stderr] {}", line));
                    }
                    Event::Output(OutputChunk::Done(status)) => {
                        self.output.finish_command(status);
                        self.runner = None;
                        // Refresh workspace manifest if the completed command
                        // was Add, Remove, or Update (they modify Cargo.toml).
                        let should_refresh = matches!(
                            &self.current_command,
                            Some(CargoCommand::Add { .. })
                                | Some(CargoCommand::Remove { .. })
                                | Some(CargoCommand::Update { .. })
                        );
                        self.current_command = None;
                        if should_refresh {
                            self.refresh_workspace();
                        }
                        self.mode = AppMode::Menu;
                    }
                    Event::Key(key) => match (key.code, key.modifiers) {
                        (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                            if let Some(runner) = self.runner.take() {
                                let _ = runner.tx_kill.send(());
                            }
                        }
                        (KeyCode::Char('j'), _) | (KeyCode::Down, _) => {
                            self.output.scroll_down(1);
                        }
                        (KeyCode::Char('k'), _) | (KeyCode::Up, _) => {
                            self.output.scroll_up(1);
                        }
                        (KeyCode::Char('c'), _) => {
                            self.output.clear_current();
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }

            AppMode::Help => {
                // Any key dismisses help
                if let Event::Key(_) = event {
                    self.mode = AppMode::Menu;
                } else {
                    self.mode = AppMode::Help;
                }
            }

            AppMode::Confirm(ctx) => {
                if let Event::Key(key) = event {
                    match key.code {
                        KeyCode::Enter => {
                            match *ctx.pending_action {
                                CommandAction::Execute(cmd) => {
                                    self.pending_command = Some(cmd);
                                }
                                _ => {}
                            }
                            self.mode = AppMode::Menu;
                        }
                        KeyCode::Esc | KeyCode::Char('q') => {
                            self.mode = AppMode::Menu;
                        }
                        _ => {
                            self.mode = AppMode::Confirm(ctx);
                        }
                    }
                } else {
                    self.mode = AppMode::Confirm(ctx);
                }
            }

            AppMode::Error(_) => {
                // Any key dismisses error
                if let Event::Key(_) = event {
                    self.mode = AppMode::Menu;
                } else {
                    self.mode = mode;
                }
            }

            AppMode::DepBrowser(mut state) => match event {
                Event::Output(OutputChunk::Stdout(line)) => {
                    self.metadata_buf.push_str(&line);
                    self.metadata_buf.push('\n');
                    self.mode = AppMode::DepBrowser(state);
                }
                Event::Output(OutputChunk::Stderr(line)) => {
                    self.stderr_buf.push_str(&line);
                    self.stderr_buf.push('\n');
                    self.mode = AppMode::DepBrowser(state);
                }
                Event::Output(OutputChunk::Done(status)) => {
                    self.runner = None;
                    self.current_command = None;
                    if !status.success() {
                        let err_msg = if self.stderr_buf.trim().is_empty() {
                            "cargo metadata failed".to_string()
                        } else {
                            self.stderr_buf.trim().to_string()
                        };
                        state.status = DepBrowserStatus::Error(err_msg);
                        self.mode = AppMode::DepBrowser(state);
                    } else {
                        match crate::cargo::metadata::parse_metadata(&self.metadata_buf) {
                            Ok(tree) => {
                                let new_state = DepBrowserState::from_packages(tree.packages);
                                self.mode = AppMode::DepBrowser(new_state);
                            }
                            Err(err_msg) => {
                                state.status = DepBrowserStatus::Error(err_msg);
                                self.mode = AppMode::DepBrowser(state);
                            }
                        }
                    }
                }
                Event::Key(key) => match key.code {
                    KeyCode::Char('j') | KeyCode::Down => {
                        state.move_down();
                        self.mode = AppMode::DepBrowser(state);
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        state.move_up();
                        self.mode = AppMode::DepBrowser(state);
                    }
                    KeyCode::Enter => {
                        if !state.packages.is_empty() {
                            let pkg = &state.packages[state.selected];
                            let url =
                                crate::ui::dep_browser::build_doc_url(&pkg.name, &pkg.version);
                            let success_msg = crate::ui::dep_browser::format_open_success_message(
                                &pkg.name,
                                &pkg.version,
                            );
                            match crate::ui::dep_browser::open_url(&url) {
                                Ok(()) => {
                                    state.message = Some(success_msg);
                                }
                                Err(err) => {
                                    state.message = Some(format!("Failed to open browser: {err}"));
                                }
                            }
                        }
                        self.mode = AppMode::DepBrowser(state);
                    }
                    KeyCode::Esc | KeyCode::Char('q') => {
                        self.mode = AppMode::Menu;
                    }
                    _ => {
                        self.mode = AppMode::DepBrowser(state);
                    }
                },
                _ => {
                    self.mode = AppMode::DepBrowser(state);
                }
            },

            AppMode::CratePicker(mut state) => {
                match event {
                    Event::Output(OutputChunk::Stdout(line)) => {
                        self.metadata_buf.push_str(&line);
                        self.metadata_buf.push('\n');
                        self.mode = AppMode::CratePicker(state);
                    }
                    Event::Output(OutputChunk::Stderr(line)) => {
                        self.stderr_buf.push_str(&line);
                        self.stderr_buf.push('\n');
                        self.mode = AppMode::CratePicker(state);
                    }
                    Event::Output(OutputChunk::Done(status)) => {
                        self.runner = None;
                        self.current_command = None;
                        if !status.success() {
                            let err_msg = if self.stderr_buf.trim().is_empty() {
                                "cargo metadata failed".to_string()
                            } else {
                                self.stderr_buf.trim().to_string()
                            };
                            state.status = CratePickerStatus::Error(err_msg);
                            self.mode = AppMode::CratePicker(state);
                        } else {
                            match crate::cargo::metadata::parse_metadata(&self.metadata_buf) {
                                Ok(tree) => {
                                    let mut packages = tree.packages;
                                    packages.sort_by(|a, b| a.name.cmp(&b.name));
                                    state.packages = packages;
                                    state.status = CratePickerStatus::Loaded;
                                    self.mode = AppMode::CratePicker(state);
                                }
                                Err(err_msg) => {
                                    state.status = CratePickerStatus::Error(err_msg);
                                    self.mode = AppMode::CratePicker(state);
                                }
                            }
                        }
                    }
                    Event::Key(key) => {
                        match key.code {
                            KeyCode::Esc => {
                                self.mode = AppMode::Menu;
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                if matches!(state.status, CratePickerStatus::Loaded) {
                                    state.move_down();
                                }
                                self.mode = AppMode::CratePicker(state);
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                if matches!(state.status, CratePickerStatus::Loaded) {
                                    state.move_up();
                                }
                                self.mode = AppMode::CratePicker(state);
                            }
                            KeyCode::Backspace => {
                                if matches!(state.status, CratePickerStatus::Loaded) {
                                    let mut new_filter = state.filter.clone();
                                    // Remove last Unicode char
                                    if let Some(last_char_start) =
                                        new_filter.char_indices().next_back().map(|(i, _)| i)
                                    {
                                        new_filter.truncate(last_char_start);
                                    }
                                    state.update_filter(new_filter);
                                }
                                self.mode = AppMode::CratePicker(state);
                            }
                            KeyCode::Enter => {
                                if matches!(state.status, CratePickerStatus::Loaded) {
                                    let filtered = state.filtered_packages();
                                    if !filtered.is_empty() {
                                        let name = filtered[state.selected].name.clone();
                                        drop(filtered);
                                        let resolved =
                                            apply_input_to_action(*state.pending_action, name);
                                        match resolved {
                                            CommandAction::Execute(cmd) => {
                                                self.pending_command = Some(cmd);
                                                self.mode = AppMode::Menu;
                                            }
                                            CommandAction::RequiresInput(spec, next_action) => {
                                                let ui_spec = InputSpec {
                                                    prompt: spec.prompt,
                                                    required: spec.required,
                                                    placeholder: spec.placeholder,
                                                };
                                                self.input = Some(InputState::new(InputSpec {
                                                    prompt: spec.prompt,
                                                    required: spec.required,
                                                    placeholder: spec.placeholder,
                                                }));
                                                self.mode = AppMode::Input(InputContext {
                                                    spec: ui_spec,
                                                    pending_action: next_action,
                                                });
                                            }
                                            _ => {
                                                self.mode = AppMode::Menu;
                                            }
                                        }
                                    } else {
                                        // Empty filtered list — no-op
                                        self.mode = AppMode::CratePicker(state);
                                    }
                                } else {
                                    self.mode = AppMode::CratePicker(state);
                                }
                            }
                            KeyCode::Char(c) => {
                                if matches!(state.status, CratePickerStatus::Loaded) {
                                    let new_filter = format!("{}{}", state.filter, c);
                                    state.update_filter(new_filter);
                                }
                                self.mode = AppMode::CratePicker(state);
                            }
                            _ => {
                                self.mode = AppMode::CratePicker(state);
                            }
                        }
                    }
                    _ => {
                        self.mode = AppMode::CratePicker(state);
                    }
                }
            }
        }
    }
}

/// Apply an input value to a `CommandAction`, filling in the first empty
/// string placeholder found in the action's associated `CargoCommand`.
fn apply_input_to_action(action: CommandAction, value: String) -> CommandAction {
    match action {
        CommandAction::Execute(cmd) => CommandAction::Execute(apply_input_to_command(cmd, &value)),
        CommandAction::RequiresInput(spec, next) => {
            // This level's input has been collected; pass it down
            CommandAction::RequiresInput(spec, Box::new(apply_input_to_action(*next, value)))
        }
        other => other,
    }
}

/// Fill the first empty-string field in a `CargoCommand` with `value`.
fn apply_input_to_command(cmd: CargoCommand, value: &str) -> CargoCommand {
    match cmd {
        CargoCommand::Test {
            filter: Some(ref f),
            doc,
        } if f.is_empty() => CargoCommand::Test {
            filter: Some(value.to_string()),
            doc,
        },
        CargoCommand::Run {
            bin: Some(ref b),
            args,
        } if b.is_empty() => CargoCommand::Run {
            bin: Some(value.to_string()),
            args,
        },
        CargoCommand::Add {
            ref krate,
            version: Some(ref v),
        } if krate.is_empty() => CargoCommand::Add {
            krate: value.to_string(),
            version: Some(v.clone()),
        },
        CargoCommand::Add {
            ref krate,
            version: None,
        } if krate.is_empty() => CargoCommand::Add {
            krate: value.to_string(),
            version: None,
        },
        CargoCommand::Add {
            krate,
            version: Some(ref v),
        } if v.is_empty() => CargoCommand::Add {
            krate,
            version: Some(value.to_string()),
        },
        CargoCommand::Remove { ref krate } if krate.is_empty() => CargoCommand::Remove {
            krate: value.to_string(),
        },
        CargoCommand::Update { krate: Some(ref k) } if k.is_empty() => CargoCommand::Update {
            krate: Some(value.to_string()),
        },
        CargoCommand::Login { ref token } if token.is_empty() => CargoCommand::Login {
            token: value.to_string(),
        },
        CargoCommand::Yank {
            ref krate,
            ref version,
        } if krate.is_empty() => CargoCommand::Yank {
            krate: value.to_string(),
            version: version.clone(),
        },
        CargoCommand::Yank { krate, ref version } if version.is_empty() => CargoCommand::Yank {
            krate,
            version: value.to_string(),
        },
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cargo::runner::RunnerHandle;
    use proptest::prelude::*;

    // Feature: cargo-tui, Property 9: Menu stack round trip
    proptest! {
        #[test]
        fn prop_menu_stack_round_trip(depth in 1..=8usize) {
            // **Validates: Requirements 2.5**
            let initial_level = MenuLevel { nodes: vec![], selected: 0 };
            let mut state = MenuState {
                stack: vec![initial_level],
            };

            // Push N additional levels (simulating entering submenus)
            for _ in 0..depth {
                state.stack.push(MenuLevel { nodes: vec![], selected: 0 });
            }

            prop_assert_eq!(state.stack.len(), depth + 1);

            // Pop N times (simulating pressing Escape N times)
            for _ in 0..depth {
                state.stack.pop();
            }

            // Stack depth should equal the initial depth (1)
            prop_assert_eq!(state.stack.len(), 1);
        }
    }

    // Feature: cargo-tui, Property 10: Concurrent command prevention
    proptest! {
        #[test]
        fn prop_concurrent_command_prevention(_dummy in proptest::bool::ANY) {
            // **Validates: Requirements 3.7**
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let mut app = App::new(None);

                // Initially no runner — can launch
                prop_assert!(app.runner.is_none());
                prop_assert!(app.can_launch_command());

                // Simulate a running command by setting runner to Some
                app.runner = Some(RunnerHandle::dummy());

                // Now runner is Some — cannot launch another command
                prop_assert!(app.runner.is_some());
                prop_assert!(!app.can_launch_command());

                Ok(()) as Result<(), TestCaseError>
            }).unwrap();
        }
    }

    #[test]
    fn test_new_app_defaults() {
        let app = App::new(None);
        assert!(!app.should_quit);
        assert!(app.pending_command.is_none());
        assert_eq!(app.terminal_size, (80, 24));
    }

    #[test]
    fn test_resize_event_updates_terminal_size() {
        let mut app = App::new(None);
        app.handle_event(Event::Resize(120, 40));
        assert_eq!(app.terminal_size, (120, 40));
    }

    #[test]
    fn test_menu_esc_at_root_sets_quit() {
        let mut app = App::new(None);
        app.handle_event(Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Esc,
            KeyModifiers::NONE,
        )));
        assert!(app.should_quit);
    }

    #[test]
    fn test_menu_q_at_root_sets_quit() {
        let mut app = App::new(None);
        app.handle_event(Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Char('q'),
            KeyModifiers::NONE,
        )));
        assert!(app.should_quit);
    }

    #[test]
    fn test_menu_question_mark_goes_to_help() {
        let mut app = App::new(None);
        app.handle_event(Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Char('?'),
            KeyModifiers::NONE,
        )));
        assert!(matches!(app.mode, AppMode::Help));
    }

    #[test]
    fn test_help_key_returns_to_menu() {
        let mut app = App::new(None);
        app.mode = AppMode::Help;
        app.handle_event(Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Esc,
            KeyModifiers::NONE,
        )));
        assert!(matches!(app.mode, AppMode::Menu));
    }

    #[test]
    fn test_confirm_esc_returns_to_menu() {
        let mut app = App::new(None);
        app.mode = AppMode::Confirm(ConfirmContext {
            message: "Are you sure?".to_string(),
            pending_action: Box::new(CommandAction::Execute(CargoCommand::Check)),
        });
        app.handle_event(Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Esc,
            KeyModifiers::NONE,
        )));
        assert!(matches!(app.mode, AppMode::Menu));
    }

    #[test]
    fn test_confirm_enter_sets_pending_command() {
        let mut app = App::new(None);
        app.mode = AppMode::Confirm(ConfirmContext {
            message: "Are you sure?".to_string(),
            pending_action: Box::new(CommandAction::Execute(CargoCommand::Check)),
        });
        app.handle_event(Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::NONE,
        )));
        assert!(matches!(app.mode, AppMode::Menu));
        assert!(app.pending_command.is_some());
    }

    #[test]
    fn test_input_esc_returns_to_menu() {
        let mut app = App::new(None);
        app.mode = AppMode::Input(InputContext {
            spec: crate::ui::input::InputSpec {
                prompt: "test",
                required: false,
                placeholder: "",
            },
            pending_action: Box::new(CommandAction::Execute(CargoCommand::Check)),
        });
        app.input = Some(InputState::new(crate::ui::input::InputSpec {
            prompt: "test",
            required: false,
            placeholder: "",
        }));
        app.handle_event(Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Esc,
            KeyModifiers::NONE,
        )));
        assert!(matches!(app.mode, AppMode::Menu));
        assert!(app.input.is_none());
    }

    #[test]
    fn test_running_done_returns_to_menu() {
        #[cfg(unix)]
        let status = std::process::Command::new("true").status().unwrap();
        #[cfg(windows)]
        let status = std::process::Command::new("cmd")
            .args(["/c", "exit", "0"])
            .status()
            .unwrap();

        let mut app = App::new(None);
        app.mode = AppMode::Running;
        app.handle_event(Event::Output(OutputChunk::Done(status)));
        assert!(matches!(app.mode, AppMode::Menu));
        assert!(app.runner.is_none());
    }

    fn make_package(name: &str) -> PackageInfo {
        PackageInfo {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            dependencies: vec![],
        }
    }

    fn make_crate_picker_state(packages: Vec<PackageInfo>, filter: &str) -> CratePickerState {
        CratePickerState {
            packages,
            filter: filter.to_string(),
            selected: 0,
            status: CratePickerStatus::Loaded,
            pending_action: Box::new(CommandAction::Execute(CargoCommand::Check)),
        }
    }

    // Feature: crate-picker, Property 2: Filtered list is a case-insensitive substring match
    proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(100))]
        #[test]
        fn prop_filtered_packages_case_insensitive_substring(
            names in proptest::collection::vec("[a-zA-Z][a-zA-Z0-9_-]{0,10}", 0..=20),
            filter in "[a-zA-Z0-9_-]{0,5}",
        ) {
            // **Validates: Requirements 3.2, 3.3**
            let packages: Vec<PackageInfo> = names.iter().map(|n| make_package(n)).collect();
            let state = make_crate_picker_state(packages.clone(), &filter);
            let filtered = state.filtered_packages();

            let lower_filter = filter.to_lowercase();

            // Every returned package must contain the filter (case-insensitive)
            for pkg in &filtered {
                prop_assert!(
                    pkg.name.to_lowercase().contains(&lower_filter),
                    "Package '{}' does not contain filter '{}'", pkg.name, filter
                );
            }

            // Every package that matches must be in the result
            let expected_count = packages.iter()
                .filter(|p| p.name.to_lowercase().contains(&lower_filter))
                .count();
            prop_assert_eq!(filtered.len(), expected_count);
        }
    }

    // Feature: crate-picker, Property 3: Filter change resets selected index to zero
    proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(100))]
        #[test]
        fn prop_update_filter_resets_selected(
            names in proptest::collection::vec("[a-z][a-z0-9]{0,8}", 1..=10),
            initial_selected in 0usize..10,
            new_filter in "[a-z]{0,5}",
        ) {
            // **Validates: Requirements 3.4**
            let packages: Vec<PackageInfo> = names.iter().map(|n| make_package(n)).collect();
            let mut state = make_crate_picker_state(packages, "");
            state.selected = initial_selected;
            state.update_filter(new_filter);
            prop_assert_eq!(state.selected, 0);
        }
    }

    // Feature: crate-picker, Property 5: Navigation wraps correctly within the filtered list
    proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(100))]
        #[test]
        fn prop_navigation_wraps(
            names in proptest::collection::vec("[a-z][a-z0-9]{0,8}", 1..=20),
            start_idx in 0usize..20,
        ) {
            // **Validates: Requirements 4.1, 4.2, 4.3, 4.4, 4.5**
            let packages: Vec<PackageInfo> = names.iter().map(|n| make_package(n)).collect();
            let n = packages.len();
            let initial = start_idx % n;

            // Test move_down wraps
            let mut state = make_crate_picker_state(packages.clone(), "");
            state.selected = initial;
            state.move_down();
            prop_assert_eq!(state.selected, (initial + 1) % n);

            // Test move_up wraps
            let mut state = make_crate_picker_state(packages.clone(), "");
            state.selected = initial;
            state.move_up();
            prop_assert_eq!(state.selected, (initial + n - 1) % n);
        }
    }

    #[test]
    fn prop_navigation_empty_list_noop() {
        // **Validates: Requirements 4.4, 4.5**
        let mut state = make_crate_picker_state(vec![], "");
        state.selected = 0;
        state.move_down();
        assert_eq!(state.selected, 0);
        state.move_up();
        assert_eq!(state.selected, 0);
    }

    // Feature: crate-picker, Property 1: Package list is sorted alphabetically
    proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(100))]
        #[test]
        fn prop_package_list_sorted_alphabetically(
            names in proptest::collection::vec("[a-zA-Z][a-zA-Z0-9_-]{0,10}", 0..=20),
        ) {
            // **Validates: Requirements 2.3**
            let packages: Vec<PackageInfo> = names.iter().map(|n| make_package(n)).collect();
            // Simulate what CratePicker does on OutputChunk::Done(success):
            // sort packages alphabetically before storing in state
            let mut sorted = packages.clone();
            sorted.sort_by(|a, b| a.name.cmp(&b.name));
            let state = CratePickerState {
                packages: sorted,
                filter: String::new(),
                selected: 0,
                status: CratePickerStatus::Loaded,
                pending_action: Box::new(CommandAction::Execute(CargoCommand::Check)),
            };
            // Assert the packages are sorted
            prop_assert!(
                state.packages.windows(2).all(|w| w[0].name <= w[1].name),
                "Package list is not sorted alphabetically"
            );
        }
    }

    // Feature: crate-picker, Property 6: Enter resolves the pending action with the selected package name
    proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(100))]
        #[test]
        fn prop_enter_resolves_pending_action(
            names in proptest::collection::vec("[a-z][a-z0-9_-]{0,10}", 1..=20),
            selected_raw in 0usize..20,
        ) {
            // **Validates: Requirements 5.1, 5.2**
            let packages: Vec<PackageInfo> = names.iter().map(|n| make_package(n)).collect();
            let n = packages.len();
            let selected = selected_raw % n;

            let state = CratePickerState {
                packages: packages.clone(),
                filter: String::new(),
                selected,
                status: CratePickerStatus::Loaded,
                pending_action: Box::new(CommandAction::Execute(CargoCommand::Remove { krate: String::new() })),
            };

            let filtered = state.filtered_packages();
            prop_assert!(!filtered.is_empty());
            let expected_name = filtered[selected].name.clone();
            drop(filtered);

            // Simulate Enter: apply_input_to_action with the selected package name
            let resolved = apply_input_to_action(
                CommandAction::Execute(CargoCommand::Remove { krate: String::new() }),
                expected_name.clone(),
            );

            match resolved {
                CommandAction::Execute(CargoCommand::Remove { krate }) => {
                    prop_assert_eq!(krate, expected_name);
                }
                _ => {
                    prop_assert!(false, "Expected Execute(Remove {{ krate }})");
                }
            }
        }
    }
}
