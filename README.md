# cargotui

A terminal user interface for managing Rust projects via Cargo.

## Overview

cargotui provides an interactive TUI for running common Cargo commands without memorizing CLI flags. It auto-detects Cargo workspaces and displays a menu of commands.

## Features

- **Build commands**: build, build --release, check, clean
- **Test commands**: test, test <filter>, test --doc, bench, run, run --bin
- **Dependency management**: add, remove, update, browse documentation
- **Publish commands**: package, publish, publish --dry-run, login, yank
- **Toolchain**: doc, doc --open, metadata
- **Utilities**: fmt, clippy, fix, tree

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
