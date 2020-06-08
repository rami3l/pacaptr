use crate::error::Error;
use crate::exec::is_exe;
use crate::packmanager::*;
use structopt::{clap, StructOpt};

/// The command line options to be collected.
#[derive(Debug, StructOpt)]
#[structopt(
    about = clap::crate_description!(),
    version = clap::crate_version!(),
    author = clap::crate_authors!()
)]
pub struct Opt {
    // Operations include Q(uery), R(emove), and S(ync).
    #[structopt(short = "Q", long)]
    query: bool,

    #[structopt(short = "R", long)]
    remove: bool,

    #[structopt(short = "S", long)]
    sync: bool,

    #[structopt(short = "U", long)]
    update: bool,

    // Main flags and flagcounters
    // ! WARNING
    // ! Some long flag names are completely different for different operations,
    // ! but I think mose of us just use the shorthand form anyway...
    // see: https://www.archlinux.org/pacman/pacman.8.html
    #[structopt(short, long = "clean", help = "(-S) clean", parse(from_occurrences))]
    c: u32,

    #[structopt(short, long = "explicit", help = "(-Q) explicit")]
    e: bool,

    #[structopt(short, long = "groups", help = "(-Q/S) groups")]
    g: bool,

    #[structopt(short, long = "info", help = "(-Q/S) info", parse(from_occurrences))]
    i: u32,

    #[structopt(short, long = "check", help = "(-Q) check")]
    k: bool,

    #[structopt(short, long = "list", help = "(-Q) list")]
    l: bool,

    #[structopt(short, long = "foreign", help = "(-Q) foreign")]
    m: bool,

    #[structopt(short, long = "nosave", help = "(-R) nosave")]
    n: bool,

    #[structopt(short, long = "owns", help = "(-Q) owns")]
    o: bool,

    #[structopt(short, long = "print", help = "(-Q/R/S) print")]
    p: bool,

    #[structopt(
        short,
        long = "search",
        alias = "recursive",
        help = "(-S) search | (-R) recursive"
    )]
    s: bool,

    #[structopt(short, long = "sysupgrade", help = "(-S) sysupgrade")]
    u: bool,

    #[structopt(short, long = "downloadonly", help = "(-S) downloadonly")]
    w: bool,

    #[structopt(short, long = "refresh", help = "(-S) refresh")]
    y: bool,

    // Other Pacaptr flags
    #[structopt(
        long = "using",
        alias = "package-manager",
        alias = "pm",
        help = "Specify the package manager to be invoked"
    )]
    using: Option<String>,

    #[structopt(long = "dryrun", alias = "dry-run", help = "Perform a dry run")]
    dry_run: bool,

    #[structopt(long = "needed", help = "Prevent reinstalling installed packages")]
    needed: bool,

    #[structopt(
        long = "yes",
        alias = "noconfirm",
        alias = "no-confirm",
        help = "Answer yes to every question"
    )]
    no_confirm: bool,

    #[structopt(
        long = "cask",
        alias = "forcecask",
        alias = "force-cask",
        help = "Force the use of `brew cask` in some commands"
    )]
    force_cask: bool,

    #[structopt(
        long = "nocache",
        alias = "no-cache",
        help = "Remove cache after installation"
    )]
    no_cache: bool,

    // Keywords
    #[structopt(name = "KEYWORDS", help = "Package name or (sometimes) regex")]
    keywords: Vec<String>,

    // Additional Non-Pacaptr Flags
    #[structopt(
        last = true,
        name = "ADDITIONAL_FLAGS",
        help = "Additional Flags passed directly to backend"
    )]
    additional_flags: Vec<String>,
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
    #[cfg(target_os = "windows")]
    pub fn detect_pm<'s>() -> &'s str {
        match () {
            _ if is_exe("choco", "") => "choco",
            _ => "unknown",
        }
    }

    /// Automatically detect the name of the package manager in question.
    #[cfg(target_os = "macos")]
    pub fn detect_pm<'s>() -> &'s str {
        match () {
            _ if is_exe("brew", "/usr/local/bin/brew") => "brew",
            _ => "unknown",
        }
    }

    /// Automatically detect the name of the package manager in question.
    #[cfg(target_os = "linux")]
    pub fn detect_pm<'s>() -> &'s str {
        match () {
            _ if is_exe("apt-get", "/usr/bin/apt-get") => "apt",
            _ if is_exe("apk", "/sbin/apk") => "apk",
            _ if is_exe("dnf", "/usr/bin/dnf") => "dnf",
            _ => "unknown",
        }
    }

    /// Generate the PackManager instance according it's name.
    pub fn gen_pm(&self) -> Box<dyn PackManager> {
        let dry_run = self.dry_run;
        let needed = self.needed;
        let no_confirm = self.no_confirm;
        let force_cask = self.force_cask;
        let no_cache = self.no_cache;
        let pack_manager: &str = if let Some(pm) = &self.using {
            pm
        } else {
            Opt::detect_pm()
        };

        match pack_manager {
            // Chocolatey
            "choco" => Box::new(chocolatey::Chocolatey {
                dry_run,
                no_confirm,
            }),

            // Homebrew
            "brew" => Box::new(homebrew::Homebrew {
                dry_run,
                force_cask,
                no_confirm,
                needed,
                no_cache,
            }),

            // Apt/Dpkg for Debian/Ubuntu/Termux
            "dpkg" | "apt" => Box::new(apt::Apt {
                dry_run,
                no_confirm,
                no_cache,
            }),

            // Apk for Alpine
            "apk" => Box::new(apk::Apk {
                dry_run,
                no_confirm,
                no_cache,
            }),

            // Dnf for RedHat
            "dnf" => Box::new(dnf::Dnf {
                dry_run,
                no_confirm,
                no_cache,
            }),

            // Unknown package manager X
            x => Box::new(unknown::Unknown { name: x.into() }),
        }
    }

    /// Execute the job according to the flags received and the package manager detected.
    pub fn dispatch_from(&self, pm: Box<dyn PackManager>) -> Result<(), Error> {
        self.check()?;
        let kws: Vec<&str> = self.keywords.iter().map(|s| s.as_ref()).collect();
        let flags: Vec<&str> = self.additional_flags.iter().map(|s| s.as_ref()).collect();

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
                _ if self.s => pm.qs(&kws, &flags),
                _ if self.u => pm.qu(&kws, &flags),
                _ => pm.q(&kws, &flags),
            },

            _ if self.remove => match () {
                _ if self.n && self.s => pm.rns(&kws, &flags),
                _ if self.n => pm.rn(&kws, &flags),
                _ if self.s => pm.rs(&kws, &flags),
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
                _ if self.s => pm.ss(&kws, &flags),
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
        self.dispatch_from(self.gen_pm())
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

    impl PackManager for MockPM {
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
        let opt = dbg!(Opt::from_iter(&["pacaptr", "-Syu"]));

        assert!(opt.keywords.is_empty());
        assert!(opt.sync);
        assert!(opt.y);
        assert!(opt.u);
        opt.dispatch_from(Box::new(opt.make_mock())).unwrap();
    }

    #[test]
    #[should_panic(expected = "should run: suy")]
    fn long_syu() {
        let opt = dbg!(Opt::from_iter(&[
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
        let opt = dbg!(Opt::from_iter(&["pacaptr", "-Sw", "curl", "wget"]));

        assert!(opt.sync);
        assert!(opt.w);
        opt.dispatch_from(Box::new(opt.make_mock())).unwrap();
    }

    #[test]
    #[should_panic(expected = r#"should run: s ["docker"]"#)]
    fn other_flags() {
        let opt = dbg!(Opt::from_iter(&[
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
    fn additional_flags() {
        let opt = dbg!(Opt::from_iter(&[
            "pacaptr",
            "-S",
            "--yes",
            "docker",
            "--",
            "--proxy=localhost:1234"
        ]));

        assert!(opt.sync);
        assert!(opt.no_confirm);
        let mut flags = opt.additional_flags.iter();
        assert_eq!(flags.next(), Some(&String::from("--proxy=localhost:1234")));
        assert_eq!(flags.next(), None);
        opt.dispatch_from(Box::new(opt.make_mock())).unwrap();
    }

    #[test]
    #[should_panic(expected = "exactly 1 operation expected")]
    fn too_many_ops() {
        let opt = dbg!(Opt::from_iter(&["pacaptr", "-SQns", "docker", "--cask"]));

        assert!(opt.sync);
        assert!(opt.query);
        assert!(opt.n);
        assert!(opt.s);
        opt.dispatch_from(Box::new(opt.make_mock())).unwrap();
    }
}
