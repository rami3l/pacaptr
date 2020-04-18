use crate::error::Error;
use crate::packmanager::*;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "pacpat-ng",
    about = "A pacman-like wrapper for many package managers."
)]
/// The command line options to be collected
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

    // Other flags
    #[structopt(long = "dryrun", alias = "dry-run", help = "Perform a dry run")]
    dry_run: bool,

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

    // Keywords
    #[structopt(name = "KEYWORDS", help = "Package names, sometimes regex")]
    keywords: Vec<String>,
}

impl Opt {
    /// Check if an Opt object is malformed.
    pub fn check(&self) -> Result<(), Error> {
        match () {
            () if {
                let mut count = 0;
                for &v in &[self.query, self.remove, self.sync, self.update] {
                    if v {
                        count += 1;
                    }
                }
                count != 1
            } =>
            {
                Err("exactly 1 operation expected".into())
            }

            _ => Ok(()),
        }
    }

    /// Detect the PackManager implementation in question.
    // TODO: Implement this function.
    pub fn detect_pm(&self) -> impl PackManager {
        return Homebrew {
            dry_run: self.dry_run,
            force_cask: self.force_cask,
        };
    }

    pub fn dispatch_from(&self, pm: impl PackManager) -> Result<(), Error> {
        self.check()?;
        let kws: Vec<&str> = self.keywords.iter().map(|s| s.as_ref()).collect();

        match () {
            () if self.query => match () {
                () if self.c == 1 => pm.qc(&kws),
                () if self.c >= 2 => unimplemented!(),
                () if self.e => pm.qe(&kws),
                () if self.i == 1 => pm.qi(&kws),
                () if self.i >= 2 => unimplemented!(),
                () if self.k => pm.qk(&kws),
                () if self.l => pm.ql(&kws),
                () if self.m => pm.qm(&kws),
                () if self.o => pm.qo(&kws),
                () if self.p => pm.qp(&kws),
                () if self.s => pm.qs(&kws),
                () if self.u => pm.qu(&kws),
                _ => pm.q(&kws),
            },

            () if self.remove => match () {
                () if self.n && self.s => pm.rns(&kws),
                () if self.n => pm.rn(&kws),
                () if self.s => pm.rs(&kws),
                _ => pm.r(&kws),
            },

            () if self.sync => match () {
                () if self.c == 1 => pm.sc(&kws),
                () if self.c == 2 => pm.scc(&kws),
                () if self.c == 3 => pm.sccc(&kws),
                () if self.c >= 4 => unimplemented!(),
                () if self.g => pm.sg(&kws),
                () if self.i == 1 => pm.si(&kws),
                () if self.i == 2 => pm.sii(&kws),
                () if self.i >= 3 => unimplemented!(),
                () if self.l => pm.sl(&kws),
                () if self.s => pm.ss(&kws),
                () if self.u && self.y => pm.suy(&kws),
                () if self.u => pm.su(&kws),
                () if self.y => pm.sy(&kws),
                () if self.w => pm.sw(&kws),
                _ => pm.s(&kws),
            },

            () if self.update => pm.u(&kws),

            _ => Err("Invalid flag".into()),
        }
    }

    pub fn dispatch(&self) -> Result<(), Error> {
        self.dispatch_from(self.detect_pm())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! make_mock_pm {
        ($( $method:ident ), *) => {
            $(fn $method(&self, kws: &[&str]) -> Result<(), Error> {
                    panic!("should run: {} {:?}", stringify!($method), kws)
            })*
        };
    }

    struct MockPM {
        dry_run: bool,
        no_confirm: bool,
        force_cask: bool,
    }

    impl PackManager for MockPM {
        make_mock_pm!(
            q, qc, qe, qi, qk, ql, qm, qo, qp, qs, qu, r, rn, rns, rs, s, sc, scc, sccc, sg, si,
            sii, sl, ss, su, suy, sw, sy, u
        );
    }

    impl Opt {
        fn make_mock(&self) -> MockPM {
            MockPM {
                dry_run: self.dry_run,
                no_confirm: self.no_confirm,
                force_cask: self.force_cask,
            }
        }
    }

    #[test]
    #[should_panic(expected = "should run: suy")]
    fn simple_syu() {
        let opt = dbg!(Opt::from_iter(&["pacapt-ng", "-Syu"]));

        assert!(opt.keywords.is_empty());
        assert!(opt.sync);
        assert!(opt.y);
        assert!(opt.u);
        opt.dispatch_from(opt.make_mock()).unwrap();
    }

    #[test]
    #[should_panic(expected = "should run: suy")]
    fn long_syu() {
        let opt = dbg!(Opt::from_iter(&[
            "pacapt-ng",
            "--sync",
            "--refresh",
            "--sysupgrade"
        ]));

        assert!(opt.keywords.is_empty());
        assert!(opt.sync);
        assert!(opt.y);
        assert!(opt.u);
        opt.dispatch_from(opt.make_mock()).unwrap();
    }

    #[test]
    #[should_panic(expected = "should run: sw [\"curl\", \"wget\"]")]
    fn simple_si() {
        let opt = dbg!(Opt::from_iter(&["pacapt-ng", "-Sw", "curl", "wget"]));

        assert!(opt.sync);
        assert!(opt.w);
        opt.dispatch_from(opt.make_mock()).unwrap();
    }

    #[test]
    #[should_panic(expected = "should run: s [\"docker\"]")]
    fn additional_flags() {
        let opt = dbg!(Opt::from_iter(&["pacapt-ng", "-S", "docker", "--cask"]));

        assert!(opt.sync);
        assert!(opt.force_cask);
        opt.dispatch_from(opt.make_mock()).unwrap();
    }

    #[test]
    #[should_panic(expected = "exactly 1 operation expected")]
    fn too_many_ops() {
        let opt = dbg!(Opt::from_iter(&["pacapt-ng", "-SQns", "docker", "--cask"]));

        assert!(opt.sync);
        assert!(opt.query);
        assert!(opt.n);
        assert!(opt.s);
        opt.dispatch_from(opt.make_mock()).unwrap();
    }
}
