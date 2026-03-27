use std::time::Duration;

use crossterm::event::KeyEvent;
use tokio::sync::mpsc;

use crate::cargo::runner::OutputChunk;

pub enum Event {
    Key(KeyEvent),
    Resize(u16, u16),
    Output(OutputChunk),
    Tick,
}

pub struct EventHandler {
    pub rx: mpsc::Receiver<Event>,
}

impl EventHandler {
    /// Spawns a background task that multiplexes terminal events and optional
    /// subprocess output chunks into a single `Event` stream.
    ///
    /// `output_rx` is an optional receiver for `OutputChunk` values produced by
    /// a running cargo subprocess.  Pass `None` when no subprocess is active.
    pub fn new(output_rx: Option<mpsc::Receiver<OutputChunk>>) -> Self {
        let (tx, rx) = mpsc::channel::<Event>(256);
        spawn_event_task(tx, output_rx);
        Self { rx }
    }
}

fn spawn_event_task(tx: mpsc::Sender<Event>, mut output_rx: Option<mpsc::Receiver<OutputChunk>>) {
    tokio::spawn(async move {
        loop {
            // Branch A: poll crossterm in a blocking thread so we don't block
            // the async runtime.
            let tx_clone = tx.clone();
            let crossterm_fut =
                tokio::task::spawn_blocking(|| crossterm::event::poll(Duration::from_millis(250)));

            tokio::select! {
                // --- Terminal events ---
                poll_result = crossterm_fut => {
                    match poll_result {
                        Ok(Ok(true)) => {
                            // An event is ready – read it.
                            match crossterm::event::read() {
                                Ok(crossterm::event::Event::Key(key)) => {
                                    if tx_clone.send(Event::Key(key)).await.is_err() {
                                        break;
                                    }
                                }
                                Ok(crossterm::event::Event::Resize(w, h)) => {
                                    if tx_clone.send(Event::Resize(w, h)).await.is_err() {
                                        break;
                                    }
                                }
                                _ => {
                                    // Other crossterm events (mouse, focus, …) – ignore.
                                }
                            }
                        }
                        Ok(Ok(false)) => {
                            // poll timed out → send Tick
                            if tx_clone.send(Event::Tick).await.is_err() {
                                break;
                            }
                        }
                        _ => {
                            // poll error – send Tick and keep going
                            if tx_clone.send(Event::Tick).await.is_err() {
                                break;
                            }
                        }
                    }
                }

                // --- Subprocess output chunks ---
                chunk = async {
                    match output_rx.as_mut() {
                        Some(rx) => rx.recv().await,
                        None => std::future::pending().await,
                    }
                } => {
                    match chunk {
                        Some(c) => {
                            if tx.send(Event::Output(c)).await.is_err() {
                                break;
                            }
                        }
                        None => {
                            // Channel closed – subprocess finished; stop
                            // forwarding output chunks.
                            output_rx = None;
                        }
                    }
                }
            }
        }
    });
}
