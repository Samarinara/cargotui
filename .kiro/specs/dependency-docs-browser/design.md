# Design Document: Dependency Docs Browser

## Overview

The dependency docs browser adds a new interactive TUI panel (`DepBrowser`) to cargo-tui that lets developers browse all packages in the current Cargo workspace and open their docs.rs documentation in the system browser with a single keypress.

The feature integrates into the existing `AppMode` state machine as a new `DepBrowser` variant, reuses the existing `cargo::metadata` module for JSON parsing, and follows the same keyboard conventions already established in the app (j/k navigation, Esc to go back, Enter to confirm).

The browser-open side effect is dispatched as a non-blocking `tokio::task::spawn_blocking` call so the TUI event loop is never stalled.

---

## Architecture

The feature touches four layers of the existing architecture:

```
┌─────────────────────────────────────────────────────────┐
│  src/cargo/mod.rs          COMMAND_TREE                  │
│    Dependencies submenu  ← add "Browse Docs" entry       │
└────────────────────────────┬────────────────────────────┘
                             │ CommandAction::BrowseDocs
                             ▼
┌─────────────────────────────────────────────────────────┐
│  src/app.rs                App::handle_event             │
│    AppMode::DepBrowser(DepBrowserState)                  │
│    • on enter  → spawn cargo metadata task               │
│    • j/k       → move selection                          │
│    • Enter     → open browser (spawn_blocking)           │
│    • Esc/q     → return to AppMode::Menu                 │
└────────────────────────────┬────────────────────────────┘
                             │
          ┌──────────────────┴──────────────────┐
          ▼                                     ▼
┌──────────────────────┐           ┌────────────────────────┐
│  src/ui/dep_browser.rs│           │  src/ui/status_bar.rs  │
│  render_dep_browser() │           │  mode_hints() extended  │
└──────────────────────┘           └────────────────────────┘
```

The metadata fetch reuses `CargoCommand::Metadata` / `spawn_cargo` from the existing runner infrastructure. Output chunks are collected into a `String` buffer; when `OutputChunk::Done` arrives the buffer is parsed with `cargo::metadata::parse_metadata`.

---

## Components and Interfaces

### `DepBrowserState` (new, in `src/app.rs`)

```rust
pub struct DepBrowserState {
    /// Packages sorted alphabetically by name, populated after metadata loads.
    pub packages: Vec<PackageInfo>,
    /// Index of the currently highlighted package (0 when list is empty).
    pub selected: usize,
    /// Current loading/display state.
    pub status: DepBrowserStatus,
    /// Transient status bar message (set after open-browser attempt).
    pub message: Option<String>,
}

pub enum DepBrowserStatus {
    Loading,
    Loaded,
    Error(String),
}
```

`DepBrowserState` is owned by `AppMode::DepBrowser(DepBrowserState)`.

### `AppMode` extension (in `src/app.rs`)

```rust
pub enum AppMode {
    // … existing variants …
    DepBrowser(DepBrowserState),
}
```

### `CommandAction` extension (in `src/cargo/mod.rs`)

```rust
pub enum CommandAction {
    // … existing variants …
    BrowseDocs,
}
```

`BrowseDocs` is a leaf action — no cargo subprocess is launched immediately; instead `App::handle_event` transitions to `AppMode::DepBrowser` and kicks off the metadata fetch.

### `render_dep_browser` (new file `src/ui/dep_browser.rs`)

```rust
pub fn render_dep_browser(state: &DepBrowserState, frame: &mut Frame, area: Rect)
```

Renders a bordered list of `{name} v{version}` rows with the selected row highlighted, or a centred message when the list is empty or in an error state.

### `open_url` helper (in `src/ui/dep_browser.rs` or a small `src/platform.rs`)

```rust
pub fn open_url(url: &str) -> std::io::Result<()>
```

Wraps the platform open command:
- Linux: `xdg-open <url>`
- macOS: `open <url>`
- Windows: `cmd /c start <url>`

Called inside `tokio::task::spawn_blocking` so it never blocks the async runtime.

---

## Data Models

### `PackageInfo` (already exists in `src/cargo/metadata.rs`)

```rust
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub dependencies: Vec<String>,
}
```

No changes needed. `DepBrowserState` stores `Vec<PackageInfo>` sorted by `name`.

### Doc URL construction

```
Doc_URL = format!("https://docs.rs/{}/{}", package.name, package.version)
```

This is a pure function with no additional path segments. Names with hyphens or underscores are passed through unchanged.

### Metadata fetch flow

1. `AppMode::DepBrowser` is entered → `DepBrowserStatus::Loading`.
2. A `tokio::sync::mpsc` channel is created; `spawn_cargo(CargoCommand::Metadata, …)` is called.
3. `EventHandler` is recreated with the new output receiver (same pattern as existing command launch in `main.rs`).
4. `OutputChunk::Stdout` lines are accumulated in a `String` buffer stored on `DepBrowserState` (or a temporary field on `App`).
5. `OutputChunk::Done(status)`:
   - Non-zero exit → `DepBrowserStatus::Error(accumulated_stderr)`.
   - Zero exit → `parse_metadata(&buffer)` → sort packages → `DepBrowserStatus::Loaded`.

To avoid adding a raw-buffer field to `DepBrowserState`, the accumulation buffer lives as `App::metadata_buf: String` (cleared on entry to `DepBrowser` mode).

---

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system — essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Doc URL construction correctness

*For any* package name and version string (including names containing hyphens and underscores), the constructed Doc_URL must have scheme `https`, host `docs.rs`, and path `/{name}/{version}` with no additional segments.

**Validates: Requirements 3.1, 6.1, 6.2, 6.3**

### Property 2: Package row display format

*For any* `PackageInfo` with a non-empty name and version, the rendered row string must equal `"{name} v{version}"`.

**Validates: Requirements 1.2**

### Property 3: Dependency list is sorted alphabetically

*For any* list of `PackageInfo` values loaded into `DepBrowserState`, the `packages` field must be sorted in ascending lexicographic order by `name`.

**Validates: Requirements 1.5**

### Property 4: Selection index invariant

*For any* `DepBrowserState` with a non-empty package list, the `selected` index must satisfy `selected < packages.len()` after every navigation event.

**Validates: Requirements 2.1**

### Property 5: Navigation round trip

*For any* non-empty `DepBrowserState` and any starting `selected` index, pressing Down then Up (or Up then Down) must return `selected` to its original value.

**Validates: Requirements 2.2, 2.3**

### Property 6: Success status message format

*For any* `PackageInfo`, after a successful browser-open attempt the status message must equal `"Opening docs for {name} v{version}…"`.

**Validates: Requirements 3.3**

---

## Error Handling

| Scenario | Behaviour |
|---|---|
| `cargo metadata` exits non-zero | `DepBrowserStatus::Error(stderr_text)` — error text shown in panel; panel stays open |
| `cargo metadata` returns empty package list | `DepBrowserStatus::Loaded` with empty `packages`; panel shows "No packages found" |
| Platform `open` command fails | `DepBrowserState::message` set to `"Failed to open browser: {error}"`; panel stays open |
| Navigation on empty list | `selected` stays 0; no panic (guarded by `if packages.is_empty()` check) |
| JSON parse error from metadata output | Treated as `DepBrowserStatus::Error(parse_error_message)` |

All errors are surfaced in-panel or in the status bar; none cause a panic or crash the application.

---

## Testing Strategy

### Unit tests

Focus on specific examples, edge cases, and error conditions:

- `test_doc_url_construction` — verifies the URL for a known name/version.
- `test_doc_url_hyphen_underscore` — verifies names with `-` and `_` are preserved.
- `test_empty_package_list_message` — verifies "No packages found" is rendered.
- `test_metadata_error_stays_open` — verifies error state keeps the panel open.
- `test_esc_returns_to_menu` — verifies mode transition on Esc.
- `test_browse_docs_in_command_tree` — verifies the menu entry exists with the correct description.
- `test_status_bar_loading` — verifies "Loading dependencies…" hint.
- `test_status_bar_loaded` — verifies "↑/↓ navigate  Enter open docs  Esc back" hint.
- `test_open_browser_error_message` — verifies "Failed to open browser: {error}" format.

### Property-based tests

Use `proptest` (already a dev-dependency in this project). Each test runs a minimum of 100 iterations.

**Property 1 — Doc URL construction correctness**
```
// Feature: dependency-docs-browser, Property 1: Doc URL construction correctness
proptest! {
    fn prop_doc_url_construction(name in "[a-zA-Z][a-zA-Z0-9_\\-]{0,30}", version in "[0-9]+\\.[0-9]+\\.[0-9]+") {
        let url = build_doc_url(&name, &version);
        prop_assert!(url.starts_with("https://docs.rs/"));
        prop_assert!(url.ends_with(&format!("{}/{}", name, version)));
        prop_assert_eq!(url, format!("https://docs.rs/{}/{}", name, version));
    }
}
```

**Property 2 — Package row display format**
```
// Feature: dependency-docs-browser, Property 2: Package row display format
proptest! {
    fn prop_package_row_format(name in "[a-z][a-z0-9_\\-]{0,20}", version in "[0-9]+\\.[0-9]+\\.[0-9]+") {
        let pkg = PackageInfo { name: name.clone(), version: version.clone(), dependencies: vec![] };
        let row = format_package_row(&pkg);
        prop_assert_eq!(row, format!("{} v{}", name, version));
    }
}
```

**Property 3 — Dependency list is sorted alphabetically**
```
// Feature: dependency-docs-browser, Property 3: Dependency list is sorted alphabetically
proptest! {
    fn prop_packages_sorted(names in prop::collection::vec("[a-z][a-z0-9]{0,10}", 0..=20)) {
        let packages: Vec<PackageInfo> = names.iter().map(|n| PackageInfo { name: n.clone(), version: "1.0.0".into(), dependencies: vec![] }).collect();
        let state = DepBrowserState::from_packages(packages);
        let sorted_names: Vec<&str> = state.packages.iter().map(|p| p.name.as_str()).collect();
        let mut expected = sorted_names.clone();
        expected.sort();
        prop_assert_eq!(sorted_names, expected);
    }
}
```

**Property 4 — Selection index invariant**
```
// Feature: dependency-docs-browser, Property 4: Selection index invariant
proptest! {
    fn prop_selection_invariant(names in prop::collection::vec("[a-z][a-z0-9]{0,10}", 1..=20), presses in prop::collection::vec(proptest::bool::ANY, 0..=50)) {
        let packages = /* build from names */;
        let mut state = DepBrowserState::from_packages(packages);
        for down in presses {
            if down { state.move_down(); } else { state.move_up(); }
            prop_assert!(state.selected < state.packages.len());
        }
    }
}
```

**Property 5 — Navigation round trip**
```
// Feature: dependency-docs-browser, Property 5: Navigation round trip
proptest! {
    fn prop_navigation_round_trip(names in prop::collection::vec("[a-z][a-z0-9]{0,10}", 1..=20), start in 0usize..20) {
        let mut state = DepBrowserState::from_packages(/* packages */);
        state.selected = start % state.packages.len();
        let original = state.selected;
        state.move_down();
        state.move_up();
        prop_assert_eq!(state.selected, original);
    }
}
```

**Property 6 — Success status message format**
```
// Feature: dependency-docs-browser, Property 6: Success status message format
proptest! {
    fn prop_success_status_message(name in "[a-z][a-z0-9_\\-]{0,20}", version in "[0-9]+\\.[0-9]+\\.[0-9]+") {
        let msg = format_open_success_message(&name, &version);
        prop_assert_eq!(msg, format!("Opening docs for {} v{}…", name, version));
    }
}
```

Both unit and property tests live in the same `#[cfg(test)]` modules as the code they test, consistent with the existing project style.
