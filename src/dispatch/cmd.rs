use crate::{
    dispatch::Config,
    error::{Error, Result},
    exec::StatusCode,
    methods,
    pm::Pm,
};
use clap::{self, AppSettings, Clap};
use itertools::Itertools;
use std::iter::FromIterator;
use tap::prelude::*;
use tokio::task;
use tt_call::tt_call;

/// The command line options to be collected.
#[derive(Debug, Clap)]
#[clap(
    version = clap::crate_version!(),
    author = clap::crate_authors!(),
    global_setting = AppSettings::ColoredHelp,
    setting = AppSettings::SubcommandRequiredElseHelp,
)]
pub struct Pacaptr {
    /// Main operations, flags and flagcounters.
    ///
    /// See: https://www.archlinux.org/pacman/pacman.8.html
    #[clap(subcommand)]
    ops: Operations,

    /// Specify the package manager to be invoked.
    #[clap(
        global = true,
        number_of_values = 1,
        long = "using",
        alias = "package-manager",
        visible_alias = "pm",
        value_name = "pm"
    )]
    using: Option<String>,

    /// Perform a dry run.
    #[clap(global = true, long = "dry-run", visible_alias = "dryrun")]
    dry_run: bool,

    /// Prevent reinstalling packages already installed.
    #[clap(global = true, long = "needed")]
    needed: bool,

    /// Answer yes to every question.
    #[clap(
        global = true,
        long = "no-confirm",
        visible_alias = "noconfirm",
        visible_alias = "yes"
    )]
    no_confirm: bool,

    /// Remove cache after installation.
    #[clap(global = true, long = "no-cache", visible_alias = "nocache")]
    no_cache: bool,

    /// Package name or (sometimes) regex.
    #[clap(global = true, name = "KEYWORDS")]
    keywords: Vec<String>,

    /// Extra Flags passed directly to backend.
    #[clap(last = true, global = true, name = "EXTRA_FLAGS")]
    extra_flags: Vec<String>,
}

/// Main operations, flags and flagcounters.
///
/// See: https://www.archlinux.org/pacman/pacman.8.html
#[derive(Debug, Clap)]
#[clap(about = clap::crate_description!())]
pub enum Operations {
    /// Query the package database.
    #[clap(short_flag = 'Q', long_flag = "query")]
    Query {
        /// View the ChangeLog of a package if it exists.
        #[clap(short, long = "changelog")]
        c: bool,

        /// Restrict or filter output to explicitly installed packages.
        #[clap(short, long = "explicit")]
        e: bool,

        /// Display information on a given package.
        #[clap(short, long = "info", parse(from_occurrences))]
        i: u32,

        /// Check that all files owned by the given package(s) are present on the system.
        #[clap(short, long = "check")]
        k: bool,

        /// List all files owned by a given package.
        #[clap(short, long = "list")]
        l: bool,

        /// Restrict or filter output to packages that were not found in the sync database(s).
        #[clap(short, long = "foreign")]
        m: bool,

        /// Search for packages that own the specified file(s).
        #[clap(short, long = "owns")]
        o: bool,

        /// Signifies that the package supplied on the command line is a file and not an entry in the database.
        #[clap(short, long = "file")]
        p: bool,

        /// Search each locally-installed package for names or descriptions that match regexp.
        #[clap(short, long = "search")]
        s: bool,

        /// Restrict or filter output to packages that are out-of-date on the local system.
        #[clap(short, long = "upgrades")]
        u: bool,
    },

    /// Remove package(s) from the system.
    #[clap(short_flag = 'R', long_flag = "remove")]
    Remove {
        /// Ignore file backup designations.
        #[clap(short, long = "nosave")]
        n: bool,

        /// Only print the targets instead of performing the actual operation.
        #[clap(short, long = "print")]
        p: bool,

        /// Remove package(s) with all their dependencies that are no longer required.
        #[clap(short, long = "recursive", parse(from_occurrences))]
        s: u32,
    },

    /// Synchronize packages.
    #[clap(short_flag = 'S', long_flag = "sync")]
    Sync {
        /// Remove packages that are no longer installed from the cache as well as currently unused sync databases to free up disk space.
        #[clap(short, long = "clean", parse(from_occurrences))]
        c: u32,

        /// Display all the members for each package group specified.
        #[clap(short, long = "groups")]
        g: bool,

        /// Display information on a given sync database package.
        #[clap(short, long = "info", parse(from_occurrences))]
        i: u32,

        /// List all packages in the specified repositories.
        #[clap(short, long = "list")]
        l: bool,

        /// Only print the targets instead of performing the actual operation.
        #[clap(short, long = "print")]
        p: bool,

        /// Search each package in the sync databases for names or descriptions that match regexp.
        #[clap(short, long = "search")]
        s: bool,

        /// Upgrade all packages that are out-of-date Each currently-installed package will be examined and upgraded if a newer package exists.
        #[clap(short, long = "sysupgrade")]
        u: bool,

        /// Retrieve all packages from the server, but do not install/upgrade anything.
        #[clap(short, long = "downloadonly")]
        w: bool,

        /// Download a fresh copy of the master package database from the server.
        #[clap(short, long = "refresh")]
        y: bool,
    },

    /// Upgrade or add package(s) to the system and install the required dependencies from sync repositories.
    #[clap(short_flag = 'U', long_flag = "update")]
    Update {
        /// Only print the targets instead of performing the actual operation.
        #[clap(short, long = "print")]
        p: bool,
    },
}

impl Pacaptr {
    /// Generates current config by merging current CLI flags with the dotfile.
    /// The precedence of the CLI flags is highter than the dotfile.
    fn merge_cfg(&self, dotfile: Config) -> Config {
        Config {
            dry_run: self.dry_run || dotfile.dry_run,
            needed: self.needed || dotfile.dry_run,
            no_confirm: self.no_confirm || dotfile.no_confirm,
            no_cache: self.no_cache || dotfile.no_cache,
            default_pm: self.using.clone().or(dotfile.default_pm),
        }
    }

    /// Executes the job according to the flags received and the package manager detected.
    pub async fn dispatch_from(&self, mut cfg: Config) -> Result<StatusCode> {
        // Collect options as a `String`, eg. `-S -y -u => "Suy"`.
        // ! HACK: In `Pm` we ensure the Pacman methods are all named with flags in ASCII order,
        // ! eg. `Suy` instead of `Syu`.
        // ! Then, in order to stay coherent with Rust coding style the method name should be `suy`.
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
                    $( $(for _ in 0..($flag as u32) {
                        options.push_str(stringify!($flag));
                    })* )?
                } )*
            }
            options.chars().sorted_unstable().pipe(String::from_iter)
        }};}

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

        let pm = cfg.conv::<Box<dyn Pm>>();

        let kws = self.keywords.iter().map(|s| s.as_ref()).collect_vec();
        let flags = self.extra_flags.iter().map(|s| s.as_ref()).collect_vec();

        macro_rules! dispatch_match {(
            methods = [{ $(
                $( #[$meta:meta] )*
                async fn $method:ident;
            )* }]
        ) => {
            match options.to_lowercase().as_ref() {
                $(stringify!($method) => pm.$method(&kws, &flags).await,)*
                _ => Err(Error::ArgParseError {
                    msg: format!("Invalid flag combination `-{}`", &options),
                }),
            }
        };}

        // Send `methods!()` to `dispatch_match`, that is,
        // `dispatch_match!( methods = [{ q qc qe .. }] )`.
        (tt_call! {
            macro = [{ methods }]
            ~~> dispatch_match
        })?;

        Ok(pm.code().await)
    }

    pub async fn dispatch(&self) -> Result<StatusCode> {
        let dotfile = task::block_in_place(Config::load);
        let cfg = self.merge_cfg(dotfile?);
        self.dispatch_from(cfg).await
    }
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;
    use async_trait::async_trait;
    use once_cell::sync::Lazy;
    use tokio::test;
    use tt_call::tt_call;

    pub struct MockPm {
        pub cfg: Config,
    }

    macro_rules! make_mock_op_body {
        ( $self:ident, $kws:ident, $flags:ident, $method:ident ) => {{
            let kws: Vec<_> = $kws.iter().chain($flags).collect();
            panic!("should run: {} {:?}", stringify!($method), &kws)
        }};
    }

    macro_rules! impl_pm_mock {(
        methods = [{ $(
            $( #[$meta:meta] )*
            async fn $method:ident;
        )* }]
    ) => {
        #[async_trait]
        impl Pm for MockPm {
            /// Gets the name of the package manager.
            fn name(&self) -> &str {
                "mockpm"
            }

            fn cfg(&self) -> &Config {
                &self.cfg
            }

            // * Automatically generated methods below... *
            $( async fn $method(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
                    make_mock_op_body!(self, kws, flags, $method)
            } )*
        }
    };}

    tt_call! {
        macro = [{ methods }]
        ~~> impl_pm_mock
    }

    static MOCK_CFG: Lazy<Config> = Lazy::new(|| Config {
        default_pm: Some("mockpm".into()),
        ..Default::default()
    });

    #[test]
    #[should_panic(expected = "should run: suy")]
    async fn simple_syu() {
        let opt = dbg!(Pacaptr::parse_from(&["pacaptr", "-Syu"]));
        let subcmd = &opt.ops;

        assert!(matches!(subcmd, &Operations::Sync{ u, y, .. } if y && u));
        assert!(opt.keywords.is_empty());

        opt.dispatch_from(MOCK_CFG.clone()).await.unwrap();
    }

    #[test]
    #[should_panic(expected = "should run: suy")]
    async fn long_syu() {
        let opt = dbg!(Pacaptr::parse_from(&[
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
    async fn simple_sw() {
        let opt = dbg!(Pacaptr::parse_from(&["pacaptr", "-Sw", "curl", "wget"]));
        let subcmd = &opt.ops;

        assert!(matches!(subcmd, &Operations::Sync { w, .. } if w));
        assert_eq!(opt.keywords, &["curl", "wget"]);

        opt.dispatch_from(MOCK_CFG.clone()).await.unwrap();
    }

    #[test]
    #[should_panic(expected = r#"should run: s ["docker"]"#)]
    async fn other_flags() {
        let opt = dbg!(Pacaptr::parse_from(&[
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
    async fn extra_flags() {
        let opt = dbg!(Pacaptr::parse_from(&[
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
    async fn using() {
        let opt = dbg!(Pacaptr::parse_from(&[
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
