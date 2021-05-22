use crate::{
    dispatch::Config,
    error::{Error, Result},
    exec::StatusCode,
    pm::Pm,
};
use clap::{self, AppSettings, Clap};
use itertools::Itertools;
use std::iter::FromIterator;
use tap::prelude::*;
use tokio::task;

/// The command line options to be collected.
#[derive(Debug, Clap)]
#[clap(
    about = clap::crate_description!(),
    version = clap::crate_version!(),
    author = clap::crate_authors!(),
    global_setting = AppSettings::ColoredHelp,
    setting = AppSettings::ArgRequiredElseHelp,
)]
pub struct Opts {
    // Main operations, flags and flagcounters.
    // see: https://www.archlinux.org/pacman/pacman.8.html
    #[clap(subcommand)]
    operations: Operations,

    // Other Pacaptr flags.
    #[clap(
        long = "using",
        alias = "package-manager",
        alias = "pm",
        value_name = "pm",
        about = "Specify the package manager to be invoked"
    )]
    using: Option<String>,

    #[clap(
        global = true,
        long = "dryrun",
        alias = "dry-run",
        about = "Perform a dry run"
    )]
    dry_run: bool,

    #[clap(
        global = true,
        long = "needed",
        about = "Prevent reinstalling packages already installed"
    )]
    needed: bool,

    #[clap(
        global = true,
        long = "no-confirm",
        alias = "noconfirm",
        alias = "yes",
        about = "Answer yes to every question"
    )]
    no_confirm: bool,

    #[clap(
        global = true,
        long = "no-cache",
        alias = "nocache",
        about = "Remove cache after installation"
    )]
    no_cache: bool,

    // Keywords
    #[clap(
        global = true,
        name = "KEYWORDS",
        about = "Package name or (sometimes) regex"
    )]
    keywords: Vec<String>,

    // Extra Non-Pacaptr Flags
    #[clap(
        last = true,
        global = true,
        name = "EXTRA_FLAGS",
        about = "Extra Flags passed directly to backend"
    )]
    extra_flags: Vec<String>,
}

/// The command line options to be collected.
#[derive(Debug, Clap)]
pub enum Operations {
    #[clap(short_flag = 'Q', long_flag = "query")]
    Query {
        #[clap(short, long = "changelog")]
        c: bool,

        #[clap(short, long = "explicit")]
        e: bool,

        #[clap(short, long = "info", parse(from_occurrences))]
        i: u32,

        #[clap(short, long = "check")]
        k: bool,

        #[clap(short, long = "list")]
        l: bool,

        #[clap(short, long = "foreign")]
        m: bool,

        #[clap(short, long = "owns")]
        o: bool,

        #[clap(short, long = "file")]
        p: bool,

        #[clap(short, long = "search")]
        s: bool,

        #[clap(short, long = "upgrades")]
        u: bool,
    },

    #[clap(short_flag = 'R', long_flag = "remove")]
    Remove {
        #[clap(short, long = "nosave")]
        n: bool,

        #[clap(short, long = "print")]
        p: bool,

        #[clap(short, long = "recursive", parse(from_occurrences))]
        s: u32,
    },

    #[clap(short_flag = 'S', long_flag = "sync")]
    Sync {
        #[clap(short, long = "clean", parse(from_occurrences))]
        c: u32,

        #[clap(short, long = "groups")]
        g: bool,

        #[clap(short, long = "info", parse(from_occurrences))]
        i: u32,

        #[clap(short, long = "list")]
        l: bool,

        #[clap(short, long = "print")]
        p: bool,

        #[clap(short, long = "search")]
        s: bool,

        #[clap(short, long = "sysupgrade")]
        u: bool,

        #[clap(short, long = "downloadonly")]
        w: bool,

        #[clap(short, long = "refresh")]
        y: bool,
    },

    #[clap(short_flag = 'U', long_flag = "update")]
    Update {
        #[clap(short, long = "print")]
        p: bool,
    },
}

impl Opts {
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
        let options = {
            // ! HACK: In `Pm` we ensure the Pacman methods are all named with flags in ASCII order,
            // ! eg. `Suy` instead of `Syu`.
            // ! Then, in order to stay coherent with Rust coding style the method name should be `suy`.

            let mut options = "".to_owned();
            macro_rules! collect_options {(
                op: $op:ident,
                $( mappings: [$( $key:ident -> $val:ident ), *], )?
                $( flags: [$( $flag:ident ), *], )?
                $( counters: [$( $counter:ident ), *], )?
            ) => {{
                options.push_str(&stringify!($op)[0..1]);
                $( $(if $key {
                    cfg.$val = true;
                })* )?
                $( $(if $flag {
                    options.push_str(stringify!($flag));
                })* )?
                $( $(for _ in 0..$counter {
                    options.push_str(stringify!($counter));
                })* )?
            }};}

            match self.operations {
                Operations::Query {
                    c,
                    e,
                    i,
                    k,
                    l,
                    m,
                    o,
                    p,
                    s,
                    u,
                } => collect_options! {
                    op: Query,
                    flags: [c, e, k, l, m, o, p, s, u],
                    counters: [i],
                },

                Operations::Remove { n, p, s } => collect_options! {
                    op: Remove,
                    mappings: [p -> dry_run],
                    flags: [n],
                    counters: [s],
                },

                Operations::Sync {
                    c,
                    g,
                    i,
                    l,
                    p,
                    s,
                    u,
                    w,
                    y,
                } => collect_options! {
                    op: Sync,
                    mappings: [p -> dry_run],
                    flags: [g, l, s, u, w, y],
                    counters: [c, i],
                },

                Operations::Update { p } => collect_options! {
                    op: Update,
                    mappings: [p -> dry_run],
                },
            }

            options
                .chars()
                .collect_vec()
                .tap_mut(|chars| chars.sort_unstable())
                .pipe(String::from_iter)
        };

        let pm: Box<dyn Pm> = cfg.into();

        let kws = self.keywords.iter().map(|s| s.as_ref()).collect_vec();
        let flags = self.extra_flags.iter().map(|s| s.as_ref()).collect_vec();

        macro_rules! dispatch_match {
            ($( $method:ident ), * $(,)?) => {
                match options.to_lowercase().as_ref() {
                    $(stringify!($method) => pm.$method(&kws, &flags).await,)*
                    _ => Err(Error::ArgParseError {
                        msg: "Invalid flag".into()
                    }),
                }
            };
        }

        dispatch_match![
            q, qc, qe, qi, qk, ql, qm, qo, qp, qs, qu, r, rn, rns, rs, rss, s, sc, scc, sccc, sg,
            si, sii, sl, ss, su, suy, sw, sy, u,
        ]?;

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
        $(
            $( #[$meta:meta] )*
            async fn $method:ident;
        )*
    ) => {
        #[async_trait]
        impl Pm for MockPm {
            /// Gets the name of the package manager.
            fn name(&self) -> String {
                "mockpm".into()
            }

            fn cfg(&self) -> &Config {
                &self.cfg
            }

            // * Automatically generated methods below... *
            $(
                $( #[$meta] )*
                async fn $method(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
                    make_mock_op_body!(self, kws, flags, $method)
                }
            )*
        }
    };}

    impl_pm_mock! {
       /// Q generates a list of installed packages.
       async fn q;

       /// Qc shows the changelog of a package.
       async fn qc;

       /// Qe lists packages installed explicitly (not as dependencies).
       async fn qe;

       /// Qi displays local package information: name, version, description, etc.
       async fn qi;

       /// Qk verifies one or more packages.
       async fn qk;

       /// Ql displays files provided by local package.
       async fn ql;

       /// Qm lists packages that are installed but are not available in any installation source (anymore).
       async fn qm;

       /// Qo queries the package which provides FILE.
       async fn qo;

       /// Qp queries a package supplied through a file supplied on the command line rather than an entry in the package management database.
       async fn qp;

       /// Qs searches locally installed package for names or descriptions.
       async fn qs;

       /// Qu lists packages which have an update available.
       async fn qu;

       /// R removes a single package, leaving all of its dependencies installed.
       async fn r;

       /// Rn removes a package and skips the generation of configuration backup files.
       async fn rn;

       /// Rns removes a package and its dependencies which are not required by any other installed package,
       /// and skips the generation of configuration backup files.
       async fn rns;

       /// Rs removes a package and its dependencies which are not required by any other installed package,
       /// and not explicitly installed by the user.
       async fn rs;

       /// Rss removes a package and its dependencies which are not required by any other installed package.
       async fn rss;

       /// S installs one or more packages by name.
       async fn s;

       /// Sc removes all the cached packages that are not currently installed, and the unused sync database.
       async fn sc;

       /// Scc removes all files from the cache.
       async fn scc;

       /// Sccc ...
       /// What is this?
       async fn sccc;

       /// Sg lists all packages belonging to the GROUP.
       async fn sg;

       /// Si displays remote package information: name, version, description, etc.
       async fn si;

       /// Sii displays packages which require X to be installed, aka reverse dependencies.
       async fn sii;

       /// Sl displays a list of all packages in all installation sources that are handled by the packages management.
       async fn sl;

       /// Ss searches for package(s) by searching the expression in name, description, short description.
       async fn ss;

       /// Su updates outdated packages.
       async fn su;

       /// Suy refreshes the local package database, then updates outdated packages.
       async fn suy;

       /// Sw retrieves all packages from the server, but does not install/upgrade anything.
       async fn sw;

       /// Sy refreshes the local package database.
       async fn sy;

       /// U upgrades or adds package(s) to the system and installs the required dependencies from sync repositories.
       async fn u;
    }

    static MOCK_CFG: Lazy<Config> = Lazy::new(|| Config {
        default_pm: Some("mockpm".into()),
        ..Default::default()
    });

    #[test]
    #[should_panic(expected = "should run: suy")]
    async fn simple_syu() {
        let opt = dbg!(Opts::parse_from(&["pacaptr", "-Syu"]));
        let subcmd = &opt.operations;

        assert!(matches!(subcmd, &Operations::Sync{
            u, y, ..
        } if y && u));
        assert!(opt.keywords.is_empty());

        opt.dispatch_from(MOCK_CFG.clone()).await.unwrap();
    }

    #[test]
    #[should_panic(expected = "should run: suy")]
    async fn long_syu() {
        let opt = dbg!(Opts::parse_from(&[
            "pacaptr",
            "--sync",
            "--refresh",
            "--sysupgrade"
        ]));
        let subcmd = &opt.operations;

        assert!(matches!(subcmd, &Operations::Sync { u, y, .. } if y && u));
        assert!(opt.keywords.is_empty());

        opt.dispatch_from(MOCK_CFG.clone()).await.unwrap();
    }

    #[test]
    #[should_panic(expected = r#"should run: sw ["curl", "wget"]"#)]
    async fn simple_si() {
        let opt = dbg!(Opts::parse_from(&["pacaptr", "-Sw", "curl", "wget"]));
        let subcmd = &opt.operations;

        assert!(matches!(subcmd, &Operations::Sync { w, .. } if w));
        assert_eq!(opt.keywords, &["curl", "wget"]);

        opt.dispatch_from(MOCK_CFG.clone()).await.unwrap();
    }

    #[test]
    #[should_panic(expected = r#"should run: s ["docker"]"#)]
    async fn other_flags() {
        let opt = dbg!(Opts::parse_from(&[
            "pacaptr", "-S", "--dryrun", "--yes", "docker"
        ]));
        let subcmd = &opt.operations;

        assert!(opt.dry_run);
        assert!(opt.no_confirm);
        assert!(matches!(subcmd, &Operations::Sync { .. }));
        assert_eq!(opt.keywords, &["docker"]);

        opt.dispatch_from(MOCK_CFG.clone()).await.unwrap();
    }

    #[test]
    #[should_panic(expected = r#"should run: s ["docker", "--proxy=localhost:1234"]"#)]
    async fn extra_flags() {
        let opt = dbg!(Opts::parse_from(&[
            "pacaptr",
            "-S",
            "--yes",
            "docker",
            "--",
            "--proxy=localhost:1234"
        ]));
        let subcmd = &opt.operations;

        assert!(opt.no_confirm);
        assert!(matches!(subcmd, &Operations::Sync { .. }));
        assert_eq!(opt.keywords, &["docker"]);

        let mut flags = opt.extra_flags.iter();
        assert_eq!(flags.next(), Some(&String::from("--proxy=localhost:1234")));
        assert_eq!(flags.next(), None);
        opt.dispatch_from(MOCK_CFG.clone()).await.unwrap();
    }
}
