mod app;
mod cargo;
mod event;
mod ui;

use std::io::{self, Stdout};

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::app::App;
use crate::cargo::workspace::Workspace;
use crate::event::EventHandler;

struct TerminalGuard(Terminal<CrosstermBackend<Stdout>>);

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.0.backend_mut(), LeaveAlternateScreen);
    }
}

fn setup_terminal() -> io::Result<TerminalGuard> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(TerminalGuard(terminal))
}

async fn run(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    workspace: Workspace,
) -> io::Result<()> {
    let mut app = App::new(Some(workspace));
    let mut event_handler = EventHandler::new(None);

    loop {
        // Render current state
        terminal.draw(|frame| ui::render(&mut app, frame))?;

        // Wait for next event
        let event = event_handler.rx.recv().await;
        match event {
            Some(event) => app.handle_event(event),
            None => break,
        }

        // Check if we should quit
        if app.should_quit {
            break;
        }

        // Check if there's a pending command to launch
        if let Some(cmd) = app.pending_command.take() {
            if let Some(workspace) = &app.workspace {
                let root = workspace.root.clone();
                let (output_tx, output_rx) = tokio::sync::mpsc::channel(256);
                // Recreate event handler with the new output receiver
                event_handler = EventHandler::new(Some(output_rx));
                if matches!(app.mode, crate::app::AppMode::DepBrowser(_)) {
                    // Launching metadata for DepBrowser — don't change mode or touch output panel
                    let _ = app.launch_metadata_for_dep_browser(&root, output_tx).await;
                } else {
                    app.output.start_command(format!("{:?}", cmd));
                    let _ = app.launch_command(cmd, &root, output_tx).await;
                }
            }
        }
    }

    Ok(())
}

fn main() {
    // Detect workspace — must be done before entering alternate screen so
    // error messages are visible in the normal terminal.
    let workspace = match crate::cargo::workspace::find_workspace(&std::env::current_dir().unwrap()) {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    };

    let mut guard = match setup_terminal() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("Failed to initialize terminal: {e}");
            std::process::exit(1);
        }
    };

    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("Failed to create tokio runtime: {e}");
            std::process::exit(1);
        }
    };

    if let Err(e) = runtime.block_on(run(&mut guard.0, workspace)) {
        eprintln!("Application error: {e}");
        std::process::exit(1);
    }
}
