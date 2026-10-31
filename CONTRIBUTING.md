# Contributing

Use the stable Rust toolchain with rustfmt and Clippy:

```sh
rustup component add rustfmt clippy
```

Run these checks before opening a pull request:

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-targets --all-features
cargo build --locked --workspace --all-features --release
RUSTDOCFLAGS="-D warnings" cargo doc --locked --workspace --all-features --no-deps
```

The documentation command above uses POSIX shell syntax. In PowerShell, set
`$env:RUSTDOCFLAGS = "-D warnings"` before running `cargo doc`.

To fix formatting, run `cargo fmt --all` and review the diff.

GitHub Actions runs the quality checks on pushes and pull requests, and runs the
test suite on Linux, macOS, and Windows. The commands use the committed
`Cargo.lock` so dependency resolution cannot silently change during CI.

To check dependencies against RustSec advisories locally:

```sh
cargo install cargo-audit --locked
cargo audit
```

The dependency audit runs when manifests, the lockfile, or its workflow change.
It also runs every Monday and can be started manually. Scheduled runs begin after
the workflow reaches the default branch. Review audit warnings as well as errors.
Do not add advisory exceptions without documenting why the project is unaffected.
