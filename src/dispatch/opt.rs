use super::config::Config;
use crate::error::{Error, Result};
use crate::exec::{is_exe, StatusCode};
use crate::package_manager::*;
use clap::{self, Clap};
use std::iter::FromIterator;

/// The command line options to be collected.
#[derive(Debug, Clap)]
#[clap(
    about = clap::crate_description!(),
    version = clap::crate_version!(),
    author = clap::crate_authors!(),
    setting = clap::AppSettings::ColoredHelp,
    setting = clap::AppSettings::ArgRequiredElseHelp,
)]
pub struct Opt {
    // Operations include Query, Remove, Sync, etc.
    #[clap(short = 'Q', long)]
    query: bool,

    #[clap(short = 'R', long)]
    remove: bool,

    #[clap(short = 'S', long)]
    sync: bool,

    #[clap(short = 'U', long)]
    update: bool,

    // Main flags and flagcounters
    // ! WARNING
    // ! Some long flag names are completely different for different operations,
    // ! but I think mose of us just use the shorthand form anyway...
    // see: https://www.archlinux.org/pacman/pacman.8.html
    #[clap(short, long = "clean", about = "(-S) clean", parse(from_occurrences))]
    c: u32,

    #[clap(short, long = "explicit", about = "(-Q) explicit")]
    e: bool,

    #[clap(short, long = "groups", about = "(-Q/S) groups")]
    g: bool,

    #[clap(short, long = "info", about = "(-Q/S) info", parse(from_occurrences))]
    i: u32,

    #[clap(short, long = "check", about = "(-Q) check")]
    k: bool,

    #[clap(short, long = "list", about = "(-Q) list")]
    l: bool,

    #[clap(short, long = "foreign", about = "(-Q) foreign")]
    m: bool,

    #[clap(short, long = "nosave", about = "(-R) nosave")]
    n: bool,

    #[clap(short, long = "owns", about = "(-Q) owns")]
    o: bool,

    #[clap(short, long = "print", about = "(-Q/R/S) print")]
    p: bool,

    #[clap(
        short,
        long = "search",
        alias = "recursive",
        about = "(-S) search | (-R) recursive",
        parse(from_occurrences)
    )]
    s: u32,

    #[clap(short, long = "sysupgrade", about = "(-S) sysupgrade")]
    u: bool,

    #[clap(short, long = "downloadonly", about = "(-S) downloadonly")]
    w: bool,

    #[clap(short, long = "refresh", about = "(-S) refresh")]
    y: bool,

    // Other Pacaptr flags
    #[clap(
        long = "using",
        alias = "package-manager",
        alias = "pm",
        value_name = "pm",
        about = "Specify the package manager to be invoked"
    )]
    using: Option<String>,

    #[clap(long = "dryrun", alias = "dry-run", about = "Perform a dry run")]
    dry_run: bool,

    #[clap(long = "needed", about = "Prevent reinstalling installed packages")]
    needed: bool,

    #[clap(
        long = "yes",
        alias = "noconfirm",
        alias = "no-confirm",
        about = "Answer yes to every question"
    )]
    no_confirm: bool,

    #[clap(
        long = "nocache",
        alias = "no-cache",
        about = "Remove cache after installation"
    )]
    no_cache: bool,

    // Keywords
    #[clap(name = "KEYWORDS", about = "Package name or (sometimes) regex")]
    keywords: Vec<String>,

    // Extra Non-Pacaptr Flags
    #[clap(
        last = true,
        name = "EXTRA_FLAGS",
        about = "Extra Flags passed directly to backend"
    )]
    extra_flags: Vec<String>,
}

impl Opt {
    /// Check if an Opt object is malformed.
    fn check(&self) -> Result<()> {
        let count = [self.query, self.remove, self.sync, self.update]
            .iter()
            .filter(|&&x| x)
            .count();
        if count != 1 {
            Err(Error::ArgParseError {
                msg: "exactly 1 operation expected".into(),
            })
        } else {
            Ok(())
        }
    }

    /// Automatically detect the name of the package manager in question.
    pub fn detect_pm_str<'s>() -> &'s str {
        #[cfg(target_os = "windows")]
        match () {
            _ if is_exe("choco", "") => "choco",
            _ => "unknown",
        }

        #[cfg(target_os = "macos")]
        match () {
            _ if is_exe("brew", "/usr/local/bin/brew") => "brew",
            _ if is_exe("port", "/opt/local/bin/port") => "port",
            _ => "unknown",
        }

        #[cfg(target_os = "linux")]
        match () {
            _ if is_exe("apk", "/sbin/apk") => "apk",
            _ if is_exe("apt", "/usr/bin/apt") => "apt",
            _ if is_exe("apt-get", "/usr/bin/apt-get") => "apt-get",
            _ if is_exe("dnf", "/usr/bin/dnf") => "dnf",
            _ if is_exe("zypper", "/usr/bin/zypper") => "zypper",

            _ => "unknown",
        }
    }

    /// Generate the PackageManager instance according it's name.
    pub fn make_pm(&self, cfg: Config) -> Box<dyn PackageManager> {
        let cfg = {
            macro_rules! make_actual_cfg {
                (
                    $other: ident,
                    bool: ($( $bool_field:ident ), *),
                    retain: ($( $retain_field:ident ), *),
                ) => {
                    Config {
                        $($bool_field: self.$bool_field || $other.$bool_field,)*
                        $($retain_field: $other.$retain_field,)*
                    }
                };
            }
            make_actual_cfg! {
                cfg,
                bool: (
                    dry_run,
                    needed,
                    no_confirm,
                    no_cache
                ),
                retain: (
                    default_pm
                ),
            }
        };

        let pm_str: &str = self
            .using
            .as_deref()
            .or_else(|| cfg.default_pm.as_deref())
            .unwrap_or_else(Opt::detect_pm_str);

        #[allow(clippy::match_single_binding)]
        match pm_str {
            // Chocolatey
            "choco" => Box::new(chocolatey::Chocolatey { cfg }),

            // Homebrew/Linuxbrew
            "brew" => Box::new(homebrew::Homebrew { cfg }),

            // Macports
            "port" if cfg!(target_os = "macos") => Box::new(macports::Macports { cfg }),

            // Apk for Alpine
            "apk" => Box::new(apk::Apk { cfg }),

            // Apt for Debian/Ubuntu/Termux (new versions)
            "apt" => Box::new(apt::Apt { cfg }),

            // Dnf for RedHat
            "dnf" => Box::new(dnf::Dnf { cfg }),

            // Zypper for SUSE
            "zypper" => Box::new(zypper::Zypper { cfg }),

            // * External Package Managers *

            // Conda
            "conda" => Box::new(conda::Conda { cfg }),

            // Pip
            "pip" => Box::new(pip::Pip {
                cmd: "pip".into(),
                cfg,
            }),
            "pip3" => Box::new(pip::Pip {
                cmd: "pip3".into(),
                cfg,
            }),

            // Tlmgr
            "tlmgr" => Box::new(tlmgr::Tlmgr { cfg }),

            // Unknown package manager X
            x => Box::new(unknown::Unknown { name: x.into() }),
        }
    }

    /// Execute the job according to the flags received and the package manager detected.
    pub async fn dispatch_from(&self, pm: Box<dyn PackageManager>) -> Result<StatusCode> {
        self.check()?;
        let kws: Vec<&str> = self.keywords.iter().map(|s| s.as_ref()).collect();
        let flags: Vec<&str> = self.extra_flags.iter().map(|s| s.as_ref()).collect();

        let mut options = "".to_owned();

        macro_rules! collect_options {(
                ops: [$( $op:ident ), *],
                flags: [$( $flag:ident ), *],
                counters: [$($counter: ident), *]
            ) => {
                $(if self.$op {
                    options.push_str(&stringify!($op)[0..1].to_uppercase());
                })*
                $(if self.$flag {
                    options.push_str(stringify!($flag));
                })*
                $(for _ in 0..self.$counter {
                    options.push_str(stringify!($counter));
                })*
            };
        }

        collect_options! {
            ops: [query, remove, sync, update],
            flags: [e, g, k, l, m, n, o, p, u, w, y],
            counters: [c, i, s]
        };

        let mut chars: Vec<char> = options.chars().collect();
        chars.sort_unstable();
        options = String::from_iter(chars);

        macro_rules! dispatch_match {
            ($( $method:ident ), *) => {
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
            si, sii, sl, ss, su, suy, sw, sy, u
        ]?;

        Ok(pm.code().await)
    }

    pub async fn dispatch(&self) -> Result<StatusCode> {
        self.dispatch_from(self.make_pm(Config::load()?)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use tokio::test;

    /*
    macro_rules! make_mock_pm {
        ($( $method:ident ), *) => {
            $(fn $method(&self, kws: &[&str], flags: &[&str]) -> futures::future::BoxFuture<'_,crate::error::Result<()>> {
                    let kws: Vec<_> = kws.iter().chain(flags).collect();
                    panic!("should run: {} {:?}", stringify!($method), &kws)
            })*
        };
    }
    */

    macro_rules! make_mock_op_body {
        ( $self:ident, $kws:ident, $flags:ident, $method:ident ) => {{
            let kws: Vec<_> = $kws.iter().chain($flags).collect();
            panic!("should run: {} {:?}", stringify!($method), &kws)
        }};
    }

    struct MockPM {}

    #[async_trait]
    impl PackageManager for MockPM {
        /// Get the name of the package manager.
        fn name(&self) -> String {
            "mockpm".into()
        }

        fn cfg(&self) -> Config {
            Config::default()
        }

        // ! WARNING!
        // ! Dirty copy-paste!

        /// Q generates a list of installed packages.
        async fn q(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
            make_mock_op_body!(self, _kws, _flags, q)
        }

        /// Qc shows the changelog of a package.
        async fn qc(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
            make_mock_op_body!(self, _kws, _flags, qc)
        }

        /// Qe lists packages installed explicitly (not as dependencies).
        async fn qe(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
            make_mock_op_body!(self, _kws, _flags, qe)
        }

        /// Qi displays local package information: name, version, description, etc.
        async fn qi(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
            make_mock_op_body!(self, _kws, _flags, qi)
        }

        /// Qk verifies one or more packages.
        async fn qk(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
            make_mock_op_body!(self, _kws, _flags, qk)
        }

        /// Ql displays files provided by local package.
        async fn ql(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
            make_mock_op_body!(self, _kws, _flags, ql)
        }

        /// Qm lists packages that are installed but are not available in any installation source (anymore).
        async fn qm(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
            make_mock_op_body!(self, _kws, _flags, qm)
        }

        /// Qo queries the package which provides FILE.
        async fn qo(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
            make_mock_op_body!(self, _kws, _flags, qo)
        }

        /// Qp queries a package supplied on the command line rather than an entry in the package management database.
        async fn qp(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
            make_mock_op_body!(self, _kws, _flags, qp)
        }

        /// Qs searches locally installed package for names or descriptions.
        async fn qs(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
            make_mock_op_body!(self, _kws, _flags, qs)
        }

        /// Qu lists packages which have an update available.
        async fn qu(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
            make_mock_op_body!(self, _kws, _flags, qu)
        }

        /// R removes a single package, leaving all of its dependencies installed.
        async fn r(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
            make_mock_op_body!(self, _kws, _flags, r)
        }

        /// Rn removes a package and skips the generation of configuration backup files.
        async fn rn(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
            make_mock_op_body!(self, _kws, _flags, rn)
        }

        /// Rns removes a package and its dependencies which are not required by any other installed package,
        /// and skips the generation of configuration backup files.
        async fn rns(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
            make_mock_op_body!(self, _kws, _flags, rns)
        }

        /// Rs removes a package and its dependencies which are not required by any other installed package,
        /// and not explicitly installed by the user.
        async fn rs(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
            make_mock_op_body!(self, _kws, _flags, rs)
        }

        /// Rss removes a package and its dependencies which are not required by any other installed package.
        async fn rss(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
            make_mock_op_body!(self, _kws, _flags, rss)
        }

        /// S installs one or more packages by name.
        async fn s(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
            make_mock_op_body!(self, _kws, _flags, s)
        }

        /// Sc removes all the cached packages that are not currently installed, and the unused sync database.
        async fn sc(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
            make_mock_op_body!(self, _kws, _flags, sc)
        }

        /// Scc removes all files from the cache.
        async fn scc(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
            make_mock_op_body!(self, _kws, _flags, scc)
        }

        /// Sccc ...
        /// What is this?
        async fn sccc(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
            make_mock_op_body!(self, _kws, _flags, sccc)
        }

        /// Sg lists all packages belonging to the GROUP.
        async fn sg(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
            make_mock_op_body!(self, _kws, _flags, sg)
        }

        /// Si displays remote package information: name, version, description, etc.
        async fn si(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
            make_mock_op_body!(self, _kws, _flags, si)
        }

        /// Sii displays packages which require X to be installed, aka reverse dependencies.
        async fn sii(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
            make_mock_op_body!(self, _kws, _flags, sii)
        }

        /// Sl displays a list of all packages in all installation sources that are handled by the packages management.
        async fn sl(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
            make_mock_op_body!(self, _kws, _flags, sl)
        }

        /// Ss searches for package(s) by searching the expression in name, description, short description.
        async fn ss(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
            make_mock_op_body!(self, _kws, _flags, ss)
        }

        /// Su updates outdated packages.
        async fn su(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
            make_mock_op_body!(self, _kws, _flags, su)
        }

        /// Suy refreshes the local package database, then updates outdated packages.
        async fn suy(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
            make_mock_op_body!(self, _kws, _flags, suy)
        }

        /// Sw retrieves all packages from the server, but does not install/upgrade anything.
        async fn sw(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
            make_mock_op_body!(self, _kws, _flags, sw)
        }

        /// Sy refreshes the local package database.
        async fn sy(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
            make_mock_op_body!(self, _kws, _flags, sy)
        }

        /// U upgrades or adds package(s) to the system and installs the required dependencies from sync repositories.
        async fn u(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
            make_mock_op_body!(self, _kws, _flags, u)
        }
    }

    impl Opt {
        fn make_mock(&self) -> MockPM {
            MockPM {}
        }
    }

    #[test]
    #[should_panic(expected = "should run: suy")]
    async fn simple_syu() {
        let opt = dbg!(Opt::parse_from(&["pacaptr", "-Syu"]));

        assert!(opt.keywords.is_empty());
        assert!(opt.sync);
        assert!(opt.y);
        assert!(opt.u);
        opt.dispatch_from(Box::new(opt.make_mock())).await.unwrap();
    }

    #[test]
    #[should_panic(expected = "should run: suy")]
    async fn long_syu() {
        let opt = dbg!(Opt::parse_from(&[
            "pacaptr",
            "--sync",
            "--refresh",
            "--sysupgrade"
        ]));

        assert!(opt.keywords.is_empty());
        assert!(opt.sync);
        assert!(opt.y);
        assert!(opt.u);
        opt.dispatch_from(Box::new(opt.make_mock())).await.unwrap();
    }

    #[test]
    #[should_panic(expected = r#"should run: sw ["curl", "wget"]"#)]
    async fn simple_si() {
        let opt = dbg!(Opt::parse_from(&["pacaptr", "-Sw", "curl", "wget"]));

        assert!(opt.sync);
        assert!(opt.w);
        opt.dispatch_from(Box::new(opt.make_mock())).await.unwrap();
    }

    #[test]
    #[should_panic(expected = r#"should run: s ["docker"]"#)]
    async fn other_flags() {
        let opt = dbg!(Opt::parse_from(&[
            "pacaptr", "-S", "--dryrun", "--yes", "docker"
        ]));

        assert!(opt.sync);
        assert!(opt.dry_run);
        assert!(opt.no_confirm);
        opt.dispatch_from(Box::new(opt.make_mock())).await.unwrap();
    }

    #[test]
    #[should_panic(expected = r#"should run: s ["docker", "--proxy=localhost:1234"]"#)]
    async fn extra_flags() {
        let opt = dbg!(Opt::parse_from(&[
            "pacaptr",
            "-S",
            "--yes",
            "docker",
            "--",
            "--proxy=localhost:1234"
        ]));

        assert!(opt.sync);
        assert!(opt.no_confirm);
        let mut flags = opt.extra_flags.iter();
        assert_eq!(flags.next(), Some(&String::from("--proxy=localhost:1234")));
        assert_eq!(flags.next(), None);
        opt.dispatch_from(Box::new(opt.make_mock())).await.unwrap();
    }

    #[test]
    #[should_panic(expected = "exactly 1 operation expected")]
    async fn too_many_ops() {
        let opt = dbg!(Opt::parse_from(&["pacaptr", "-SQns", "docker"]));

        assert!(opt.sync);
        assert!(opt.query);
        assert!(opt.n);
        assert_eq!(opt.s, 1);
        opt.dispatch_from(Box::new(opt.make_mock())).await.unwrap();
    }
}
