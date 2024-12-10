//! Definitions for command line argument mapping and dispatching.
//!
//! An overall introduction of how this module works:
//!
//! 1. [`clap`] handles command line arguments and generate a [`Pacaptr`]
//!    instance holding all the flags and options.
//!
//! 2. [`Config`] reads the configuration file (if it exists) and then merges it
//!    with the current command line arguments using [`Pacaptr::merge_cfg`].
//!
//! 3. The correct package manager to be used will be indicated by the user
//!    (through command line arguments or config file), or, if this is not the
//!    case, automatically detected by [`detect_pm_str`].
//!
//! 4. [`Pacaptr::dispatch`] will call the corresponding trait method, eg.
//!    `.suy()`, according to the combination of flags and options obtained
//!    above.

use clap::{self, ArgAction, Parser};
use figment::Figment;
use itertools::Itertools;
use pacaptr::{
    config::Config,
    error::{Error, Result},
    methods,
    pm::BoxPm,
    print::{println, prompt},
};
use tap::prelude::*;
use tokio::task;
use tt_call::tt_call;

use crate::_built::GIT_VERSION;

fn version() -> &'static str {
    GIT_VERSION.unwrap_or(clap::crate_version!())
}

/// The command line options to be collected.
#[derive(Debug, Parser)]
#[command(
    version = version(),
    author = clap::crate_authors!(),
    about = clap::crate_description!(),
    before_help = format!("{} {}", clap::crate_name!(), version()),
    subcommand_required = true,
    arg_required_else_help = true,
)]
#[allow(clippy::struct_excessive_bools)]
pub struct Pacaptr {
    #[command(subcommand)]
    ops: Operations,

    /// Specify the package manager to be invoked.
    #[arg(
        global = true,
        number_of_values = 1,
        long = "using",
        alias = "package-manager",
        visible_alias = "pm",
        value_name = "pm"
    )]
    using: Option<String>,

    /// Perform a dry run.
    #[arg(global = true, long, visible_alias = "dryrun")]
    dry_run: bool,

    /// Prevent reinstalling previously installed packages.
    #[arg(global = true, long = "needed")]
    needed: bool,

    /// Answer yes to every question.
    #[arg(
        global = true,
        long,
        visible_alias = "noconfirm",
        visible_alias = "yes"
    )]
    no_confirm: bool,

    /// Remove cache after installation.
    #[arg(global = true, long, visible_alias = "nocache")]
    no_cache: bool,

    /// Suppress log output.
    #[arg(global = true, long, conflicts_with = "dry_run")]
    quiet: bool,

    /// Package name or (sometimes) regex.
    #[arg(global = true, name = "KEYWORDS")]
    keywords: Vec<String>,

    /// Extra Flags passed directly to backend.
    #[arg(last = true, global = true, name = "EXTRA_FLAGS")]
    extra_flags: Vec<String>,
}

// For details on operations, flags and flagcounters, see: https://www.archlinux.org/pacman/pacman.8.html
#[derive(Debug, Parser)]
#[command(about = clap::crate_description!())]
enum Operations {
    /// Query the package database.
    #[command(short_flag = 'Q', long_flag = "query")]
    Query {
        /// View the changelog of a package if it exists.
        #[arg(short, long = "changelog")]
        c: bool,

        /// Restrict or filter output to explicitly installed packages.
        #[arg(short, long = "explicit")]
        e: bool,

        /// Display information on a given package.
        #[arg(short, long = "info", action(ArgAction::Count))]
        i: u8,

        /// Check that all files owned by the given package(s) are present on
        /// the system.
        #[arg(short, long = "check")]
        k: bool,

        /// List all files owned by a given package.
        #[arg(short, long = "list")]
        l: bool,

        /// Restrict or filter output to packages that were not found in the
        /// sync database(s).
        #[arg(short, long = "foreign")]
        m: bool,

        /// Search for packages that own the specified file(s).
        #[arg(short, long = "owns")]
        o: bool,

        /// Signifies that the package supplied on the command line is a file
        /// and not an entry in the database.
        #[arg(short, long = "file")]
        p: bool,

        /// Search each locally-installed package for names or descriptions that
        /// match regexp.
        #[arg(short, long = "search")]
        s: bool,

        /// Restrict or filter output to packages that are out-of-date on the
        /// local system.
        #[arg(short, long = "upgrades")]
        u: bool,
    },

    /// Remove package(s) from the system.
    #[command(short_flag = 'R', long_flag = "remove")]
    Remove {
        /// Ignore file backup designations.
        #[arg(short, long = "nosave")]
        n: bool,

        /// Only print the targets instead of performing the actual operation.
        #[arg(short, long = "print")]
        p: bool,

        /// Remove package(s) with all their dependencies that are no longer
        /// required.
        #[arg(short, long = "recursive", action(ArgAction::Count))]
        s: u8,
    },

    /// Synchronize packages.
    #[command(short_flag = 'S', long_flag = "sync")]
    Sync {
        /// Remove packages that are no longer installed from the cache as well
        /// as currently unused sync databases to free up disk space.
        #[arg(short, long = "clean", action(ArgAction::Count))]
        c: u8,

        /// Display all the members for each package group specified.
        #[arg(short, long = "groups")]
        g: bool,

        /// Display information on a given sync database package.
        #[arg(short, long = "info", action(ArgAction::Count))]
        i: u8,

        /// List all packages in the specified repositories.
        #[arg(short, long = "list")]
        l: bool,

        /// Only print the targets instead of performing the actual operation.
        #[arg(short, long = "print")]
        p: bool,

        /// Search each package in the sync databases for names or descriptions
        /// that match regexp.
        #[arg(short, long = "search")]
        s: bool,

        /// Upgrade all packages that are out-of-date Each currently-installed
        /// package will be examined and upgraded if a newer package exists.
        #[arg(short, long = "sysupgrade")]
        u: bool,

        /// Retrieve all packages from the server, but do not install/upgrade
        /// anything.
        #[arg(short, long = "downloadonly")]
        w: bool,

        /// Download a fresh copy of the master package database from the
        /// server.
        #[arg(short, long = "refresh")]
        y: bool,
    },

    /// Upgrade or add package(s) to the system and install the required
    /// dependencies from sync repositories.
    #[command(short_flag = 'U', long_flag = "update")]
    Update {
        /// Only print the targets instead of performing the actual operation.
        #[arg(short, long = "print")]
        p: bool,
    },
}

impl Pacaptr {
    /// Generates the current [`Config`] according to current command line
    /// arguments.
    fn cfg(&self) -> Config {
        Config {
            dry_run: self.dry_run,
            needed: self.needed,
            no_confirm: self.no_confirm,
            no_cache: self.no_cache,
            quiet: self.quiet.then_some(true),
            default_pm: self.using.clone(),
        }
    }

    /// Executes the job according to the flags received and the package manager
    /// detected.
    ///
    /// # Errors
    /// See [`Error`](crate::error::Error) for a list of possible errors.
    #[allow(trivial_numeric_casts)]
    async fn dispatch_from(&self, mut cfg: Config) -> Result<()> {
        /// Collect options as a `String`, eg. `-S -y -u => "Suy"`.
        ///
        /// # Hack
        /// In `Pm` we ensure the Pacman methods are all named with flags in
        /// ASCII order, eg. `Suy` instead of `Syu`. Then, in order to
        /// stay coherent with Rust coding style the method name should be
        /// `suy`.
        macro_rules! collect_options {(
            $( $op:ident {
                $( mappings: [$( $key:ident -> $val:ident ), *], )?
                $( flags: [$( $flag:ident ), *], )?
            }, )*
        ) => {{
            let mut options = String::new();
            match self.ops {
                $( Operations::$op {
                    $( $( $key, )* )?
                    $( $( $flag, )* )?
                } => {
                    options.push_str(&stringify!($op)[0..1]);
                    $( $(if $key {
                        cfg.$val = true;
                    })* )?
                    $( $(for _ in 0..(u8::from($flag)) {
                        options.push_str(stringify!($flag));
                    })* )?
                } )*
            }
            options.chars().sorted_unstable().pipe(String::from_iter)
        }};}

        // Ensure that the cursor is not hidden when `Ctrl-C` is used.
        // See: https://github.com/console-rs/dialoguer/issues/77#issuecomment-669986406
        _ = ctrlc::set_handler(move || {
            let term = console::Term::stdout();
            _ = term.show_cursor();
        })
        .tap_err(|e| println(&*prompt::INFO, e));

        let options = collect_options! {
            Query {
                flags: [c, e, i, k, l, m, o, p, s, u],
            },
            Remove {
                mappings: [p -> dry_run],
                flags: [n, s],
            },
            Sync {
                mappings: [p -> dry_run],
                flags: [c, g, i, l, s, u, w, y],
            },
            Update {
                mappings: [p -> dry_run],
            },
        };

        let pm = cfg.conv::<BoxPm>();

        let kws = self.keywords.iter().map(AsRef::as_ref).collect_vec();
        let flags = self.extra_flags.iter().map(AsRef::as_ref).collect_vec();

        /// Call the method indicated by `options` on `pm`. That is:
        ///
        /// ```rust
        /// match options.to_lowercase().as_ref() {
        ///     "q" => pm.q(&kws, &flags).await,
        ///     ..
        /// }
        /// ```
        macro_rules! dispatch_match {(
            methods = [{ $(
                $( #[$meta:meta] )*
                async fn $method:ident;
            )* }]
        ) => {
            match options.to_lowercase().as_ref() {
                $(stringify!($method) => pm.$method(&kws, &flags).await,)*
                _ => Err(Error::ArgParseError {
                    msg: format!("invalid flag combination `-{options}`"),
                }),
            }
        };}

        // Send `methods!()` to `dispatch_match`. That is,
        // `dispatch_match!( methods = [{ q qc qe .. }] )`.
        tt_call! {
            macro = [{ methods }]
            ~~> dispatch_match
        }
    }

    /// Runs [`dispatch_from`](Pacaptr::dispatch_from) with automatically
    /// detected [`Config`].
    ///
    /// The [`Config`] precedence is defined in the following order:
    /// - CLI flags;
    /// - Environment variables;
    /// - The config file.
    ///
    /// # Errors
    /// See [`Error`](crate::error::Error) for a list of possible errors.
    pub async fn dispatch(&self) -> Result<()> {
        let cfg = self.cfg().join(task::block_in_place(|| {
            Figment::new()
                .join(Config::env_provider())
                .join(Config::file_provider())
                .extract::<Config>()
        })?);
        self.dispatch_from(cfg).await
    }
}

#[cfg(all(test, feature = "test"))]
mod tests {
    #![allow(clippy::dbg_macro)]

    use std::sync::LazyLock;

    use tokio::test;

    use super::*;

    static MOCK_CFG: LazyLock<Config> = LazyLock::new(|| Config {
        default_pm: Some("mockpm".into()),
        ..Config::default()
    });

    #[test]
    #[should_panic(expected = "should run: suy")]
    #[allow(clippy::semicolon_if_nothing_returned)]
    async fn simple_syu() {
        let opt = dbg!(Pacaptr::parse_from(["pacaptr", "-Syu"]));
        let subcmd = &opt.ops;

        assert!(matches!(subcmd, &Operations::Sync{ u, y, .. } if y && u));
        assert!(opt.keywords.is_empty());

        opt.dispatch_from(MOCK_CFG.clone()).await.unwrap();
    }

    #[test]
    #[should_panic(expected = "should run: suy")]
    #[allow(clippy::semicolon_if_nothing_returned)]
    async fn long_syu() {
        let opt = dbg!(Pacaptr::parse_from([
            "pacaptr",
            "--sync",
            "--refresh",
            "--sysupgrade"
        ]));
        let subcmd = &opt.ops;

        assert!(matches!(subcmd, &Operations::Sync { u, y, .. } if y && u));
        assert!(opt.keywords.is_empty());

        opt.dispatch_from(MOCK_CFG.clone()).await.unwrap();
    }

    #[test]
    #[should_panic(expected = r#"should run: sw ["curl", "wget"]"#)]
    #[allow(clippy::semicolon_if_nothing_returned)]
    async fn simple_sw() {
        let opt = dbg!(Pacaptr::parse_from(["pacaptr", "-Sw", "curl", "wget"]));
        let subcmd = &opt.ops;

        assert!(matches!(subcmd, &Operations::Sync { w, .. } if w));
        assert_eq!(opt.keywords, &["curl", "wget"]);

        opt.dispatch_from(MOCK_CFG.clone()).await.unwrap();
    }

    #[test]
    #[should_panic(expected = r#"should run: s ["docker"]"#)]
    #[allow(clippy::semicolon_if_nothing_returned)]
    async fn other_flags() {
        let opt = dbg!(Pacaptr::parse_from([
            "pacaptr", "-S", "--dryrun", "--yes", "docker"
        ]));
        let subcmd = &opt.ops;

        assert!(opt.dry_run);
        assert!(opt.no_confirm);
        assert!(matches!(subcmd, &Operations::Sync { .. }));
        assert_eq!(opt.keywords, &["docker"]);

        opt.dispatch_from(MOCK_CFG.clone()).await.unwrap();
    }

    #[test]
    #[should_panic(expected = r#"should run: s ["docker", "--proxy=localhost:1234"]"#)]
    #[allow(clippy::semicolon_if_nothing_returned)]
    async fn extra_flags() {
        let opt = dbg!(Pacaptr::parse_from([
            "pacaptr",
            "-S",
            "--yes",
            "docker",
            "--",
            "--proxy=localhost:1234"
        ]));
        let subcmd = &opt.ops;

        assert!(opt.no_confirm);
        assert!(matches!(subcmd, &Operations::Sync { .. }));
        assert_eq!(opt.keywords, &["docker"]);
        assert_eq!(opt.extra_flags, &["--proxy=localhost:1234"]);

        opt.dispatch_from(MOCK_CFG.clone()).await.unwrap();
    }

    #[test]
    #[should_panic(expected = r#"should run: si ["docker", "--proxy=localhost:1234"]"#)]
    #[allow(clippy::semicolon_if_nothing_returned)]
    async fn using() {
        let opt = dbg!(Pacaptr::parse_from([
            "pacaptr",
            "--pm",
            "mockpm",
            "-Si",
            "--yes",
            "docker",
            "--",
            "--proxy=localhost:1234"
        ]));
        let subcmd = &opt.ops;

        assert!(opt.no_confirm);
        assert!(matches!(subcmd, &Operations::Sync { i, .. } if i == 1));
        assert_eq!(opt.keywords, &["docker"]);
        assert_eq!(opt.extra_flags, &["--proxy=localhost:1234"]);

        opt.dispatch_from(MOCK_CFG.clone()).await.unwrap();
    }
}
