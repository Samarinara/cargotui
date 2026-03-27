pub mod metadata;
pub mod runner;
pub mod workspace;

use std::ffi::OsString;

pub struct CommandNode {
    pub name: &'static str,
    pub description: &'static str,
    pub action: CommandAction,
}

pub enum CommandAction {
    Submenu(Vec<CommandNode>),
    Execute(CargoCommand),
    RequiresInput(InputSpec, Box<CommandAction>),
    Confirm(Box<CommandAction>),
    BrowseDocs,
    PickCrate(Box<CommandAction>),
}

pub struct InputSpec {
    pub prompt: &'static str,
    pub required: bool,
    pub placeholder: &'static str,
}

#[derive(Debug, Clone)]
pub enum CargoCommand {
    Build {
        release: bool,
    },
    Check,
    Clean,
    Test {
        filter: Option<String>,
        doc: bool,
    },
    Bench,
    Run {
        bin: Option<String>,
        args: Option<String>,
    },
    Add {
        krate: String,
        version: Option<String>,
    },
    Remove {
        krate: String,
    },
    Update {
        krate: Option<String>,
    },
    Publish {
        dry_run: bool,
    },
    Package,
    Login {
        token: String,
    },
    Yank {
        krate: String,
        version: String,
    },
    Metadata,
    Doc {
        open: bool,
    },
    Fmt,
    Clippy,
    Fix,
    Tree,
}

impl CargoCommand {
    pub fn to_argv(&self) -> Vec<OsString> {
        let mut argv: Vec<OsString> = vec!["cargo".into()];
        match self {
            CargoCommand::Build { release } => {
                argv.push("build".into());
                if *release {
                    argv.push("--release".into());
                }
            }
            CargoCommand::Check => {
                argv.push("check".into());
            }
            CargoCommand::Clean => {
                argv.push("clean".into());
            }
            CargoCommand::Test { filter, doc } => {
                argv.push("test".into());
                if *doc {
                    argv.push("--doc".into());
                } else if let Some(f) = filter {
                    argv.push(f.into());
                }
            }
            CargoCommand::Bench => {
                argv.push("bench".into());
            }
            CargoCommand::Run { bin, args } => {
                argv.push("run".into());
                if let Some(b) = bin {
                    argv.push("--bin".into());
                    argv.push(b.into());
                }
                if let Some(a) = args {
                    argv.push("--".into());
                    argv.push(a.into());
                }
            }
            CargoCommand::Add { krate, version } => {
                argv.push("add".into());
                match version {
                    Some(v) => argv.push(format!("{}@{}", krate, v).into()),
                    None => argv.push(krate.into()),
                }
            }
            CargoCommand::Remove { krate } => {
                argv.push("remove".into());
                argv.push(krate.into());
            }
            CargoCommand::Update { krate } => {
                argv.push("update".into());
                if let Some(k) = krate {
                    argv.push(k.into());
                }
            }
            CargoCommand::Publish { dry_run } => {
                argv.push("publish".into());
                if *dry_run {
                    argv.push("--dry-run".into());
                }
            }
            CargoCommand::Package => {
                argv.push("package".into());
            }
            CargoCommand::Login { token } => {
                argv.push("login".into());
                argv.push(token.into());
            }
            CargoCommand::Yank { krate, version } => {
                argv.push("yank".into());
                argv.push("--version".into());
                argv.push(version.into());
                argv.push(krate.into());
            }
            CargoCommand::Metadata => {
                argv.push("metadata".into());
                argv.push("--format-version".into());
                argv.push("1".into());
            }
            CargoCommand::Doc { open } => {
                argv.push("doc".into());
                if *open {
                    argv.push("--open".into());
                }
            }
            CargoCommand::Fmt => {
                argv.push("fmt".into());
            }
            CargoCommand::Clippy => {
                argv.push("clippy".into());
            }
            CargoCommand::Fix => {
                argv.push("fix".into());
            }
            CargoCommand::Tree => {
                argv.push("tree".into());
            }
        }
        argv
    }
}

pub static COMMAND_TREE: std::sync::LazyLock<Vec<CommandNode>> = std::sync::LazyLock::new(|| {
    vec![
        CommandNode {
            name: "Build",
            description: "Compile and build commands",
            action: CommandAction::Submenu(vec![
                CommandNode {
                    name: "build",
                    description: "Compile the current package",
                    action: CommandAction::Execute(CargoCommand::Build { release: false }),
                },
                CommandNode {
                    name: "build --release",
                    description: "Compile with optimizations",
                    action: CommandAction::Execute(CargoCommand::Build { release: true }),
                },
                CommandNode {
                    name: "check",
                    description: "Check the package for errors without producing artifacts",
                    action: CommandAction::Execute(CargoCommand::Check),
                },
                CommandNode {
                    name: "clean",
                    description: "Remove the target directory",
                    action: CommandAction::Confirm(Box::new(CommandAction::Execute(
                        CargoCommand::Clean,
                    ))),
                },
            ]),
        },
        CommandNode {
            name: "Test",
            description: "Test and benchmark commands",
            action: CommandAction::Submenu(vec![
                CommandNode {
                    name: "test",
                    description: "Run the tests",
                    action: CommandAction::Execute(CargoCommand::Test {
                        filter: None,
                        doc: false,
                    }),
                },
                CommandNode {
                    name: "test <filter>",
                    description: "Run tests matching a filter",
                    action: CommandAction::RequiresInput(
                        InputSpec {
                            prompt: "Test filter",
                            required: true,
                            placeholder: "test_name",
                        },
                        Box::new(CommandAction::Execute(CargoCommand::Test {
                            filter: Some(String::new()),
                            doc: false,
                        })),
                    ),
                },
                CommandNode {
                    name: "test --doc",
                    description: "Run documentation tests",
                    action: CommandAction::Execute(CargoCommand::Test {
                        filter: None,
                        doc: true,
                    }),
                },
                CommandNode {
                    name: "bench",
                    description: "Run the benchmarks",
                    action: CommandAction::Execute(CargoCommand::Bench),
                },
                CommandNode {
                    name: "run",
                    description: "Run the binary",
                    action: CommandAction::Execute(CargoCommand::Run {
                        bin: None,
                        args: None,
                    }),
                },
                CommandNode {
                    name: "run --bin <name>",
                    description: "Run a specific binary",
                    action: CommandAction::RequiresInput(
                        InputSpec {
                            prompt: "Binary name",
                            required: true,
                            placeholder: "my_bin",
                        },
                        Box::new(CommandAction::Execute(CargoCommand::Run {
                            bin: Some(String::new()),
                            args: None,
                        })),
                    ),
                },
            ]),
        },
        CommandNode {
            name: "Dependencies",
            description: "Manage package dependencies",
            action: CommandAction::Submenu(vec![
                CommandNode {
                    name: "add <crate>",
                    description: "Add a dependency",
                    action: CommandAction::RequiresInput(
                        InputSpec {
                            prompt: "Crate name",
                            required: true,
                            placeholder: "serde",
                        },
                        Box::new(CommandAction::Execute(CargoCommand::Add {
                            krate: String::new(),
                            version: None,
                        })),
                    ),
                },
                CommandNode {
                    name: "add <crate@version>",
                    description: "Add a dependency at a specific version",
                    action: CommandAction::RequiresInput(
                        InputSpec {
                            prompt: "Crate name",
                            required: true,
                            placeholder: "serde",
                        },
                        Box::new(CommandAction::RequiresInput(
                            InputSpec {
                                prompt: "Version",
                                required: true,
                                placeholder: "1.0",
                            },
                            Box::new(CommandAction::Execute(CargoCommand::Add {
                                krate: String::new(),
                                version: Some(String::new()),
                            })),
                        )),
                    ),
                },
                CommandNode {
                    name: "remove <crate>",
                    description: "Remove a dependency",
                    action: CommandAction::PickCrate(Box::new(CommandAction::Execute(
                        CargoCommand::Remove {
                            krate: String::new(),
                        },
                    ))),
                },
                CommandNode {
                    name: "update",
                    description: "Update all dependencies",
                    action: CommandAction::Execute(CargoCommand::Update { krate: None }),
                },
                CommandNode {
                    name: "update <crate>",
                    description: "Update a specific dependency",
                    action: CommandAction::PickCrate(Box::new(CommandAction::Execute(
                        CargoCommand::Update {
                            krate: Some(String::new()),
                        },
                    ))),
                },
                CommandNode {
                    name: "Browse Docs",
                    description: "Browse dependencies and open documentation in browser",
                    action: CommandAction::BrowseDocs,
                },
            ]),
        },
        CommandNode {
            name: "Publish",
            description: "Package and publish commands",
            action: CommandAction::Submenu(vec![
                CommandNode {
                    name: "package",
                    description: "Assemble the local package into a distributable tarball",
                    action: CommandAction::Execute(CargoCommand::Package),
                },
                CommandNode {
                    name: "publish",
                    description: "Upload the package to the registry",
                    action: CommandAction::Confirm(Box::new(CommandAction::Execute(
                        CargoCommand::Publish { dry_run: false },
                    ))),
                },
                CommandNode {
                    name: "publish --dry-run",
                    description: "Perform all checks without uploading",
                    action: CommandAction::Execute(CargoCommand::Publish { dry_run: true }),
                },
                CommandNode {
                    name: "login",
                    description: "Log in to a registry",
                    action: CommandAction::RequiresInput(
                        InputSpec {
                            prompt: "API token",
                            required: true,
                            placeholder: "token",
                        },
                        Box::new(CommandAction::Execute(CargoCommand::Login {
                            token: String::new(),
                        })),
                    ),
                },
                CommandNode {
                    name: "yank",
                    description: "Remove a pushed crate from the index",
                    action: CommandAction::RequiresInput(
                        InputSpec {
                            prompt: "Crate name",
                            required: true,
                            placeholder: "my_crate",
                        },
                        Box::new(CommandAction::RequiresInput(
                            InputSpec {
                                prompt: "Version",
                                required: true,
                                placeholder: "1.0.0",
                            },
                            Box::new(CommandAction::Execute(CargoCommand::Yank {
                                krate: String::new(),
                                version: String::new(),
                            })),
                        )),
                    ),
                },
            ]),
        },
        CommandNode {
            name: "Toolchain",
            description: "Documentation and metadata",
            action: CommandAction::Submenu(vec![
                CommandNode {
                    name: "doc",
                    description: "Build the documentation",
                    action: CommandAction::Execute(CargoCommand::Doc { open: false }),
                },
                CommandNode {
                    name: "doc --open",
                    description: "Build and open the documentation in a browser",
                    action: CommandAction::Execute(CargoCommand::Doc { open: true }),
                },
                CommandNode {
                    name: "metadata",
                    description: "Output the resolved dependencies of the package in JSON",
                    action: CommandAction::Execute(CargoCommand::Metadata),
                },
            ]),
        },
        CommandNode {
            name: "Utilities",
            description: "Code quality and utility commands",
            action: CommandAction::Submenu(vec![
                CommandNode {
                    name: "fmt",
                    description: "Format all Rust files",
                    action: CommandAction::Execute(CargoCommand::Fmt),
                },
                CommandNode {
                    name: "clippy",
                    description: "Run the Clippy linter",
                    action: CommandAction::Execute(CargoCommand::Clippy),
                },
                CommandNode {
                    name: "fix",
                    description: "Automatically fix lint warnings",
                    action: CommandAction::Execute(CargoCommand::Fix),
                },
                CommandNode {
                    name: "tree",
                    description: "Display a tree visualization of dependencies",
                    action: CommandAction::Execute(CargoCommand::Tree),
                },
            ]),
        },
    ]
});

#[cfg(test)]
mod tests {
    use super::*;

    fn os(s: &str) -> OsString {
        OsString::from(s)
    }

    #[test]
    fn test_build_debug() {
        assert_eq!(
            CargoCommand::Build { release: false }.to_argv(),
            vec![os("cargo"), os("build")]
        );
    }

    #[test]
    fn test_build_release() {
        assert_eq!(
            CargoCommand::Build { release: true }.to_argv(),
            vec![os("cargo"), os("build"), os("--release")]
        );
    }

    #[test]
    fn test_check() {
        assert_eq!(
            CargoCommand::Check.to_argv(),
            vec![os("cargo"), os("check")]
        );
    }

    #[test]
    fn test_clean() {
        assert_eq!(
            CargoCommand::Clean.to_argv(),
            vec![os("cargo"), os("clean")]
        );
    }

    #[test]
    fn test_test_plain() {
        assert_eq!(
            CargoCommand::Test {
                filter: None,
                doc: false
            }
            .to_argv(),
            vec![os("cargo"), os("test")]
        );
    }

    #[test]
    fn test_test_filter() {
        assert_eq!(
            CargoCommand::Test {
                filter: Some("my_test".into()),
                doc: false
            }
            .to_argv(),
            vec![os("cargo"), os("test"), os("my_test")]
        );
    }

    #[test]
    fn test_test_doc() {
        assert_eq!(
            CargoCommand::Test {
                filter: None,
                doc: true
            }
            .to_argv(),
            vec![os("cargo"), os("test"), os("--doc")]
        );
    }

    #[test]
    fn test_bench() {
        assert_eq!(
            CargoCommand::Bench.to_argv(),
            vec![os("cargo"), os("bench")]
        );
    }

    #[test]
    fn test_run_plain() {
        assert_eq!(
            CargoCommand::Run {
                bin: None,
                args: None
            }
            .to_argv(),
            vec![os("cargo"), os("run")]
        );
    }

    #[test]
    fn test_run_bin() {
        assert_eq!(
            CargoCommand::Run {
                bin: Some("my_bin".into()),
                args: None
            }
            .to_argv(),
            vec![os("cargo"), os("run"), os("--bin"), os("my_bin")]
        );
    }

    #[test]
    fn test_run_args() {
        assert_eq!(
            CargoCommand::Run {
                bin: None,
                args: Some("--foo".into())
            }
            .to_argv(),
            vec![os("cargo"), os("run"), os("--"), os("--foo")]
        );
    }

    #[test]
    fn test_run_bin_and_args() {
        assert_eq!(
            CargoCommand::Run {
                bin: Some("b".into()),
                args: Some("a".into())
            }
            .to_argv(),
            vec![
                os("cargo"),
                os("run"),
                os("--bin"),
                os("b"),
                os("--"),
                os("a")
            ]
        );
    }

    #[test]
    fn test_add_no_version() {
        assert_eq!(
            CargoCommand::Add {
                krate: "serde".into(),
                version: None
            }
            .to_argv(),
            vec![os("cargo"), os("add"), os("serde")]
        );
    }

    #[test]
    fn test_add_with_version() {
        assert_eq!(
            CargoCommand::Add {
                krate: "serde".into(),
                version: Some("1.0".into())
            }
            .to_argv(),
            vec![os("cargo"), os("add"), os("serde@1.0")]
        );
    }

    #[test]
    fn test_remove() {
        assert_eq!(
            CargoCommand::Remove {
                krate: "serde".into()
            }
            .to_argv(),
            vec![os("cargo"), os("remove"), os("serde")]
        );
    }

    #[test]
    fn test_update_all() {
        assert_eq!(
            CargoCommand::Update { krate: None }.to_argv(),
            vec![os("cargo"), os("update")]
        );
    }

    #[test]
    fn test_update_specific() {
        assert_eq!(
            CargoCommand::Update {
                krate: Some("serde".into())
            }
            .to_argv(),
            vec![os("cargo"), os("update"), os("serde")]
        );
    }

    #[test]
    fn test_publish() {
        assert_eq!(
            CargoCommand::Publish { dry_run: false }.to_argv(),
            vec![os("cargo"), os("publish")]
        );
    }

    #[test]
    fn test_publish_dry_run() {
        assert_eq!(
            CargoCommand::Publish { dry_run: true }.to_argv(),
            vec![os("cargo"), os("publish"), os("--dry-run")]
        );
    }

    #[test]
    fn test_package() {
        assert_eq!(
            CargoCommand::Package.to_argv(),
            vec![os("cargo"), os("package")]
        );
    }

    #[test]
    fn test_login() {
        assert_eq!(
            CargoCommand::Login {
                token: "mytoken".into()
            }
            .to_argv(),
            vec![os("cargo"), os("login"), os("mytoken")]
        );
    }

    #[test]
    fn test_yank() {
        assert_eq!(
            CargoCommand::Yank {
                krate: "my_crate".into(),
                version: "1.0.0".into()
            }
            .to_argv(),
            vec![
                os("cargo"),
                os("yank"),
                os("--version"),
                os("1.0.0"),
                os("my_crate")
            ]
        );
    }

    #[test]
    fn test_metadata() {
        assert_eq!(
            CargoCommand::Metadata.to_argv(),
            vec![os("cargo"), os("metadata"), os("--format-version"), os("1")]
        );
    }

    #[test]
    fn test_doc() {
        assert_eq!(
            CargoCommand::Doc { open: false }.to_argv(),
            vec![os("cargo"), os("doc")]
        );
    }

    #[test]
    fn test_doc_open() {
        assert_eq!(
            CargoCommand::Doc { open: true }.to_argv(),
            vec![os("cargo"), os("doc"), os("--open")]
        );
    }

    #[test]
    fn test_fmt() {
        assert_eq!(CargoCommand::Fmt.to_argv(), vec![os("cargo"), os("fmt")]);
    }

    #[test]
    fn test_clippy() {
        assert_eq!(
            CargoCommand::Clippy.to_argv(),
            vec![os("cargo"), os("clippy")]
        );
    }

    #[test]
    fn test_fix() {
        assert_eq!(CargoCommand::Fix.to_argv(), vec![os("cargo"), os("fix")]);
    }

    #[test]
    fn test_tree() {
        assert_eq!(CargoCommand::Tree.to_argv(), vec![os("cargo"), os("tree")]);
    }

    // Feature: cargo-tui, Property 3: CargoCommand argv correctness
    #[test]
    fn prop_cargo_command_argv_correctness() {
        use proptest::prelude::*;
        use proptest::prop_oneof;

        let strategy = prop_oneof![
            proptest::bool::ANY.prop_map(|release| CargoCommand::Build { release }),
            Just(CargoCommand::Check),
            Just(CargoCommand::Clean),
            (
                proptest::option::of("[a-z][a-z0-9]{0,10}"),
                proptest::bool::ANY
            )
                .prop_map(|(filter, doc)| CargoCommand::Test { filter, doc }),
            Just(CargoCommand::Bench),
            (
                proptest::option::of("[a-z][a-z0-9]{0,10}"),
                proptest::option::of("[a-z][a-z0-9]{0,10}")
            )
                .prop_map(|(bin, args)| CargoCommand::Run { bin, args }),
            (
                "[a-z][a-z0-9]{0,10}",
                proptest::option::of("[a-z][a-z0-9]{0,10}")
            )
                .prop_map(|(krate, version)| CargoCommand::Add { krate, version }),
            "[a-z][a-z0-9]{0,10}".prop_map(|krate| CargoCommand::Remove { krate }),
            proptest::option::of("[a-z][a-z0-9]{0,10}")
                .prop_map(|krate| CargoCommand::Update { krate }),
            proptest::bool::ANY.prop_map(|dry_run| CargoCommand::Publish { dry_run }),
            Just(CargoCommand::Package),
            "[a-z][a-z0-9]{0,10}".prop_map(|token| CargoCommand::Login { token }),
            ("[a-z][a-z0-9]{0,10}", "[a-z][a-z0-9]{0,10}")
                .prop_map(|(krate, version)| CargoCommand::Yank { krate, version }),
            Just(CargoCommand::Metadata),
            proptest::bool::ANY.prop_map(|open| CargoCommand::Doc { open }),
            Just(CargoCommand::Fmt),
            Just(CargoCommand::Clippy),
            Just(CargoCommand::Fix),
            Just(CargoCommand::Tree),
        ];

        proptest::proptest!(|(cmd in strategy)| {
            let argv = cmd.to_argv();

            // argv[0] must always be "cargo"
            proptest::prop_assert_eq!(&argv[0], &OsString::from("cargo"));

            // must have at least "cargo" + subcommand
            proptest::prop_assert!(argv.len() >= 2);

            // second element must match the expected subcommand
            let expected_subcmd = match &cmd {
                CargoCommand::Build { .. } => "build",
                CargoCommand::Check => "check",
                CargoCommand::Clean => "clean",
                CargoCommand::Test { .. } => "test",
                CargoCommand::Bench => "bench",
                CargoCommand::Run { .. } => "run",
                CargoCommand::Add { .. } => "add",
                CargoCommand::Remove { .. } => "remove",
                CargoCommand::Update { .. } => "update",
                CargoCommand::Publish { .. } => "publish",
                CargoCommand::Package => "package",
                CargoCommand::Login { .. } => "login",
                CargoCommand::Yank { .. } => "yank",
                CargoCommand::Metadata => "metadata",
                CargoCommand::Doc { .. } => "doc",
                CargoCommand::Fmt => "fmt",
                CargoCommand::Clippy => "clippy",
                CargoCommand::Fix => "fix",
                CargoCommand::Tree => "tree",
            };
            proptest::prop_assert_eq!(&argv[1], &OsString::from(expected_subcmd));
        });
    }

    #[test]
    fn test_all_argv_start_with_cargo() {
        let commands = vec![
            CargoCommand::Build { release: false },
            CargoCommand::Check,
            CargoCommand::Clean,
            CargoCommand::Test {
                filter: None,
                doc: false,
            },
            CargoCommand::Bench,
            CargoCommand::Run {
                bin: None,
                args: None,
            },
            CargoCommand::Add {
                krate: "x".into(),
                version: None,
            },
            CargoCommand::Remove { krate: "x".into() },
            CargoCommand::Update { krate: None },
            CargoCommand::Publish { dry_run: false },
            CargoCommand::Package,
            CargoCommand::Login { token: "t".into() },
            CargoCommand::Yank {
                krate: "x".into(),
                version: "1.0".into(),
            },
            CargoCommand::Metadata,
            CargoCommand::Doc { open: false },
            CargoCommand::Fmt,
            CargoCommand::Clippy,
            CargoCommand::Fix,
            CargoCommand::Tree,
        ];
        for cmd in &commands {
            let argv = cmd.to_argv();
            assert_eq!(argv[0], os("cargo"), "argv[0] must be 'cargo'");
            assert!(argv.len() >= 2, "argv must have at least a subcommand");
        }
    }
}
