use crate::cargo::CargoCommand;
use std::path::Path;
use std::process::ExitStatus;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

pub enum OutputChunk {
    Stdout(String),
    Stderr(String),
    Done(ExitStatus),
}

pub struct RunnerHandle {
    pub tx_kill: oneshot::Sender<()>,
    pub task: JoinHandle<ExitStatus>,
}

pub async fn spawn_cargo(
    cmd: &CargoCommand,
    workspace_root: &Path,
    output_tx: mpsc::Sender<OutputChunk>,
) -> std::io::Result<RunnerHandle> {
    let argv = cmd.to_argv();
    let program = &argv[0];
    let args = &argv[1..];

    let mut child = Command::new(program)
        .args(args)
        .current_dir(workspace_root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().expect("stdout was piped");
    let stderr = child.stderr.take().expect("stderr was piped");

    let (tx_kill, rx_kill) = oneshot::channel::<()>();

    let task = tokio::spawn(async move {
        let mut stdout_lines = BufReader::new(stdout).lines();
        let mut stderr_lines = BufReader::new(stderr).lines();
        let mut rx_kill = rx_kill;
        let mut stdout_done = false;
        let mut stderr_done = false;

        loop {
            if stdout_done && stderr_done {
                break;
            }
            tokio::select! {
                line = stdout_lines.next_line(), if !stdout_done => {
                    match line {
                        Ok(Some(l)) => { let _ = output_tx.send(OutputChunk::Stdout(l)).await; }
                        _ => { stdout_done = true; }
                    }
                }
                line = stderr_lines.next_line(), if !stderr_done => {
                    match line {
                        Ok(Some(l)) => { let _ = output_tx.send(OutputChunk::Stderr(l)).await; }
                        _ => { stderr_done = true; }
                    }
                }
                _ = &mut rx_kill => {
                    let _ = child.kill().await;
                    break;
                }
            }
        }

        let status = child.wait().await.expect("failed to wait on child");
        let _ = output_tx.send(OutputChunk::Done(status)).await;
        status
    });

    Ok(RunnerHandle { tx_kill, task })
}

#[cfg(test)]
impl RunnerHandle {
    pub fn dummy() -> Self {
        let (tx_kill, _rx) = tokio::sync::oneshot::channel::<()>();
        let task = tokio::spawn(async {
            // Return a real ExitStatus via a no-op process
            #[cfg(unix)]
            let status = std::process::Command::new("true").status().unwrap();
            #[cfg(windows)]
            let status = std::process::Command::new("cmd")
                .args(["/c", "exit", "0"])
                .status()
                .unwrap();
            status
        });
        RunnerHandle { tx_kill, task }
    }
}
