use exec::Cmd;

use super::{DryRunStrategy, PackageManager, Strategies};
use crate::dispatch::config::Config;
use crate::error::Error;
use crate::exec;

pub struct Tlmgr {
    pub cfg: Config,
}

lazy_static! {
    static ref CHECK_DRY_STRAT: Strategies = Strategies {
        dry_run: DryRunStrategy::with_flags(&["--dry-run"]),
        ..Default::default()
    };
}

impl PackageManager for Tlmgr {
    /// Get the name of the package manager.
    fn name(&self) -> String {
        "tlmgr".into()
    }

    fn cfg(&self) -> Config {
        self.cfg.clone()
    }

    /// Q generates a list of installed packages.
    fn q(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.qi(kws, flags)
    }

    /// Qi displays local package information: name, version, description, etc.
    fn qi(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run_default(
            Cmd::new(&["tlmgr", "info", "--only-installed"])
                .kws(kws)
                .flags(flags),
        )
    }

    /// Qk verifies one or more packages.
    fn qk(&self, _kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run_default(Cmd::new(&["tlmgr", "check", "files"]).flags(flags))
    }

    /// Ql displays files provided by local package.
    fn ql(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run_default(
            Cmd::new(&["tlmgr", "info", "--only-installed", "--list"])
                .kws(kws)
                .flags(flags),
        )
    }

    /// R removes a single package, leaving all of its dependencies installed.
    fn r(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run(
            Cmd::new(&["tlmgr", "remove"]).kws(kws).flags(flags),
            Default::default(),
            CHECK_DRY_STRAT.clone(),
        )
    }

    /// S installs one or more packages by name.
    fn s(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run(
            Cmd::new(&["tlmgr", "install"]).kws(kws).flags(flags),
            Default::default(),
            CHECK_DRY_STRAT.clone(),
        )
    }

    /// Si displays remote package information: name, version, description, etc.
    fn si(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run_default(Cmd::new(&["tlmgr", "info"]).kws(kws).flags(flags))
    }

    /// Sl displays a list of all packages in all installation sources that are handled by the packages management.
    fn sl(&self, _kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run_default(Cmd::new(&["tlmgr", "info"]).flags(flags))
    }

    /// Ss searches for package(s) by searching the expression in name, description, short description.
    fn ss(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run_default(
            Cmd::new(&["tlmgr", "search", "--global"])
                .kws(kws)
                .flags(flags),
        )
    }

    /// Su updates outdated packages.
    fn su(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        let cmd: &[&str] = if kws.is_empty() {
            &["tlmgr", "update", "--self", "--all"]
        } else {
            &["tlmgr", "update", "--self"]
        };
        self.just_run(
            Cmd::new(cmd).kws(kws).flags(flags),
            Default::default(),
            CHECK_DRY_STRAT.clone(),
        )
    }

    /// Suy refreshes the local package database, then updates outdated packages.
    fn suy(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.su(kws, flags)
    }

    /// U upgrades or adds package(s) to the system and installs the required dependencies from sync repositories.
    fn u(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run(
            Cmd::new(&["tlmgr", "install", "--file"])
                .kws(kws)
                .flags(flags),
            Default::default(),
            CHECK_DRY_STRAT.clone(),
        )
    }
}
