# Changelog

## 0.3.0 - 2026-11-24

### Added
- Add Left and Right cursor movement, Home and End, and forward Delete to command input fields.
- Keep cursor movement and deletion on UTF-8 character boundaries, including accented characters and emoji.

### Fixed
- Correct the Git installation command in the README to use the actual `Samarinara/cargotui` repository.

### Maintenance
- Update Ratatui to 0.30.2 and Crossterm to 0.29. Set the Tokio minimum version to 1.53.3.
- Refresh the dependency lockfile and resolve the RustSec advisories recorded in the previous release.
- Add GitHub Actions checks for formatting, Clippy, documentation, release builds, and tests on Linux, macOS, and Windows. Add a scheduled RustSec dependency audit.
