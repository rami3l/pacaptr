use super::config::Config;
use crate::error::Error;
use crate::exec::is_exe;
use crate::package_manager::*;
use clap::{self, Clap};
// use structopt::{clap, StructOpt};

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
    #[clap(short = "Q", long)]
    query: bool,

    #[clap(short = "R", long)]
    remove: bool,

    #[clap(short = "S", long)]
    sync: bool,

    #[clap(short = "U", long)]
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
        long = "cask",
        alias = "forcecask",
        alias = "force-cask",
        about = "Force the use of `brew cask` in some commands"
    )]
    force_cask: bool,

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
    pub fn check(&self) -> Result<(), Error> {
        let count = [self.query, self.remove, self.sync, self.update]
            .iter()
            .filter(|&&x| x)
            .count();
        if count != 1 {
            Err("exactly 1 operation expected".into())
        } else {
            Ok(())
        }
    }

    /// Automatically detect the name of the package manager in question.
    pub fn detect_pm<'s>() -> &'s str {
        #[cfg(target_os = "windows")]
        match () {
            _ if is_exe("choco", "") => "choco",
            _ => "unknown",
        }

        #[cfg(target_os = "macos")]
        match () {
            _ if is_exe("brew", "/usr/local/bin/brew") => "brew",
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
    pub fn gen_pm(&self, cfg: Config) -> Box<dyn PackageManager> {
        let cfg = {
            macro_rules! make_actual_cfg {
                ($other:ident, ($( $field:ident ), *)) => {{
                    Config {
                        $($field: self.$field || $other.$field,)*
                    }
                }};
            }
            make_actual_cfg! {
                cfg,
                (
                    dry_run,
                    needed,
                    no_confirm,
                    force_cask,
                    no_cache
                )
            }
        };

        let package_manager: &str = self.using.as_deref().unwrap_or_else(Opt::detect_pm);

        match package_manager {
            // Chocolatey
            "choco" => Box::new(chocolatey::Chocolatey { cfg }),

            // Homebrew
            "brew" if cfg!(target_os = "macos") => Box::new(homebrew::Homebrew { cfg }),

            // Linuxbrew
            "brew" => Box::new(linuxbrew::Linuxbrew { cfg }),

            // Apk for Alpine
            "apk" => Box::new(apk::Apk { cfg }),

            // Apt for Debian/Ubuntu/Termux (new versions)
            "apt" => Box::new(apt::Apt { cfg }),

            // Apt-Get/Dpkg for Debian/Ubuntu/Termux
            "apt-get" => Box::new(aptget::AptGet { cfg }),

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

            // Unknown package manager X
            x => Box::new(unknown::Unknown { name: x.into() }),
        }
    }

    /// Execute the job according to the flags received and the package manager detected.
    pub fn dispatch_from(&self, pm: Box<dyn PackageManager>) -> Result<(), Error> {
        self.check()?;
        let kws: Vec<&str> = self.keywords.iter().map(|s| s.as_ref()).collect();
        let flags: Vec<&str> = self.extra_flags.iter().map(|s| s.as_ref()).collect();

        match () {
            _ if self.query => match () {
                _ if self.c == 1 => pm.qc(&kws, &flags),
                _ if self.c >= 2 => unimplemented!(),
                _ if self.e => pm.qe(&kws, &flags),
                _ if self.i == 1 => pm.qi(&kws, &flags),
                _ if self.i >= 2 => unimplemented!(),
                _ if self.k => pm.qk(&kws, &flags),
                _ if self.l => pm.ql(&kws, &flags),
                _ if self.m => pm.qm(&kws, &flags),
                _ if self.o => pm.qo(&kws, &flags),
                _ if self.p => pm.qp(&kws, &flags),
                _ if self.s == 1 => pm.qs(&kws, &flags),
                _ if self.s >= 2 => unimplemented!(),
                _ if self.u => pm.qu(&kws, &flags),
                _ => pm.q(&kws, &flags),
            },

            _ if self.remove => match () {
                _ if self.n && (self.s == 1) => pm.rns(&kws, &flags),
                _ if self.n => pm.rn(&kws, &flags),
                _ if self.s == 1 => pm.rs(&kws, &flags),
                _ if self.s == 2 => pm.rss(&kws, &flags),
                _ if self.s >= 3 => unimplemented!(),
                _ => pm.r(&kws, &flags),
            },

            _ if self.sync => match () {
                _ if self.c == 1 => pm.sc(&kws, &flags),
                _ if self.c == 2 => pm.scc(&kws, &flags),
                _ if self.c == 3 => pm.sccc(&kws, &flags),
                _ if self.c >= 4 => unimplemented!(),
                _ if self.g => pm.sg(&kws, &flags),
                _ if self.i == 1 => pm.si(&kws, &flags),
                _ if self.i == 2 => pm.sii(&kws, &flags),
                _ if self.i >= 3 => unimplemented!(),
                _ if self.l => pm.sl(&kws, &flags),
                _ if self.s == 1 => pm.ss(&kws, &flags),
                _ if self.s >= 2 => unimplemented!(),
                _ if self.u && self.y => pm.suy(&kws, &flags),
                _ if self.u => pm.su(&kws, &flags),
                _ if self.y => pm.sy(&kws, &flags),
                _ if self.w => pm.sw(&kws, &flags),
                _ => pm.s(&kws, &flags),
            },

            _ if self.update => pm.u(&kws, &flags),

            _ => Err("Invalid flag".into()),
        }
    }

    pub fn dispatch(&self) -> Result<(), Error> {
        self.dispatch_from(self.gen_pm(Config::load()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! make_mock_pm {
        ($( $method:ident ), *) => {
            $(fn $method(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
                    let kws: Vec<_> = kws.iter().chain(flags).collect();
                    panic!("should run: {} {:?}", stringify!($method), &kws)
            })*
        };
    }

    struct MockPM {}

    impl PackageManager for MockPM {
        /// Get the name of the package manager.
        fn name(&self) -> String {
            "mockpm".into()
        }

        make_mock_pm!(
            q, qc, qe, qi, qk, ql, qm, qo, qp, qs, qu, r, rn, rns, rs, s, sc, scc, sccc, sg, si,
            sii, sl, ss, su, suy, sw, sy, u
        );
    }

    impl Opt {
        fn make_mock(&self) -> MockPM {
            MockPM {}
        }
    }

    #[test]
    #[should_panic(expected = "should run: suy")]
    fn simple_syu() {
        let opt = dbg!(Opt::parse_from(&["pacaptr", "-Syu"]));

        assert!(opt.keywords.is_empty());
        assert!(opt.sync);
        assert!(opt.y);
        assert!(opt.u);
        opt.dispatch_from(Box::new(opt.make_mock())).unwrap();
    }

    #[test]
    #[should_panic(expected = "should run: suy")]
    fn long_syu() {
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
        opt.dispatch_from(Box::new(opt.make_mock())).unwrap();
    }

    #[test]
    #[should_panic(expected = r#"should run: sw ["curl", "wget"]"#)]
    fn simple_si() {
        let opt = dbg!(Opt::parse_from(&["pacaptr", "-Sw", "curl", "wget"]));

        assert!(opt.sync);
        assert!(opt.w);
        opt.dispatch_from(Box::new(opt.make_mock())).unwrap();
    }

    #[test]
    #[should_panic(expected = r#"should run: s ["docker"]"#)]
    fn other_flags() {
        let opt = dbg!(Opt::parse_from(&[
            "pacaptr", "-S", "--dryrun", "--yes", "docker", "--cask"
        ]));

        assert!(opt.sync);
        assert!(opt.dry_run);
        assert!(opt.no_confirm);
        assert!(opt.force_cask);
        opt.dispatch_from(Box::new(opt.make_mock())).unwrap();
    }

    #[test]
    #[should_panic(expected = r#"should run: s ["docker", "--proxy=localhost:1234"]"#)]
    fn extra_flags() {
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
        opt.dispatch_from(Box::new(opt.make_mock())).unwrap();
    }

    #[test]
    #[should_panic(expected = "exactly 1 operation expected")]
    fn too_many_ops() {
        let opt = dbg!(Opt::parse_from(&["pacaptr", "-SQns", "docker", "--cask"]));

        assert!(opt.sync);
        assert!(opt.query);
        assert!(opt.n);
        assert_eq!(opt.s, 1);
        opt.dispatch_from(Box::new(opt.make_mock())).unwrap();
    }
}
