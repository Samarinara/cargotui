# cargotui

A terminal user interface for managing Rust projects via Cargo.

## Overview

cargotui provides an interactive TUI for running common Cargo commands without memorizing CLI flags. It auto-detects Cargo workspaces and displays a menu of commands.

<img width="1920" height="1080" alt="image" src="https://github.com/user-attachments/assets/834ad4b2-8e43-4d29-94c7-01204bb523de" />

## Features

- **Build commands**: build, build --release, build --features, build --target, build --no-default-features, check, clean

<img width="1920" height="1080" alt="image" src="https://github.com/user-attachments/assets/0c9e150d-5517-4a0b-af85-b6a6a20bfa3a" />

- **Test commands**: test, test <filter>, test --doc, test --no-run, test --ignored, bench, run, run --bin, run --features

<img width="1920" height="1080" alt="image" src="https://github.com/user-attachments/assets/88bc55a6-2f80-4d0e-8577-eea97889c757" />

- **Dependency management**: add, remove, update, browse documentation

<img width="1920" height="1080" alt="image" src="https://github.com/user-attachments/assets/82c07f0a-1969-4bcc-a9ae-0dc5ae6dd949" />

- **Publish commands**: package, publish, publish --dry-run, login, logout, yank

<img width="1920" height="1080" alt="image" src="https://github.com/user-attachments/assets/4bfb04d5-e490-4f49-af3f-8999fc025c5a" />

- **Toolchain**: doc, doc --open, metadata, rustc, rustdoc

<img width="1920" height="1080" alt="image" src="https://github.com/user-attachments/assets/d252ec75-3783-474e-8c7a-1217af1cd055" />

- **Utilities**: fmt, clippy, clippy -- -D warnings, clippy --fix, fix, tree

<img width="1920" height="1080" alt="image" src="https://github.com/user-attachments/assets/6b7a12cc-b703-47ed-b7fd-ba022ce0c8bb" />

- **Advanced**: search, vendor, generate-lockfile, locate-project, verify-project, report future-incompatibilities

<img width="1920" height="1080" alt="image" src="https://github.com/user-attachments/assets/aaca4d5f-8bbc-4f2f-b181-a91f92a3f662" />


## Installation

### From latest cargo release

```
cargo install cargotui
```

### From latest git version

```
cargo install --git https://github.com/yourusername/cargotui
```

## Usage

Run from any directory containing a Cargo.toml (or a subdirectory of one):

```
cargotui
```

## Keybindings

| Key | Action |
|-----|--------|
| `j` / `Down` | Navigate down |
| `k` / `Up` | Navigate up |
| `Enter` | Execute selected command |
| `Esc` | Go back / Quit |
| `Tab` | Switch focus between panels |
| `?` | Show help |

## Requirements

- Rust toolchain (stable)
- A Cargo.toml in the current directory or an ancestor
