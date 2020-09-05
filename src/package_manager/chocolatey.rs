use super::{DryRunStrategy, PackageManager, PromptStrategy, Strategies};
use crate::dispatch::config::Config;
use crate::error::Error;
use crate::exec::Cmd;

pub struct Chocolatey {
    pub cfg: Config,
}

lazy_static! {
    static ref PROMPT_STRAT: Strategies = Strategies {
        prompt: PromptStrategy::native_prompt(&["--yes"]),
        ..Default::default()
    };
    static ref CHECK_DRY_STRAT: Strategies = Strategies {
        dry_run: DryRunStrategy::with_flags(&["--what-if"]),
        ..Default::default()
    };
}

// Windows is so special! It's better not to "sudo" automatically.
impl PackageManager for Chocolatey {
    /// Get the name of the package manager.
    fn name(&self) -> String {
        "choco".into()
    }

    fn cfg(&self) -> Config {
        self.cfg.clone()
    }

    // Method override.
    fn just_run_default(&self, cmd: Cmd) -> Result<(), Error> {
        self.just_run(cmd, Default::default(), CHECK_DRY_STRAT.clone())
    }

    /// Q generates a list of installed packages.
    fn q(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run_default(
            Cmd::new(&["choco", "list", "--localonly"])
                .kws(kws)
                .flags(flags),
        )
    }

    /// Qi displays local package information: name, version, description, etc.
    fn qi(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.si(kws, flags)
    }

    /// Qu lists packages which have an update available.
    fn qu(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run_default(Cmd::new(&["choco", "outdated"]).kws(kws).flags(flags))
    }

    /// R removes a single package, leaving all of its dependencies installed.
    fn r(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run(
            Cmd::new(&["choco", "uninstall"]).kws(kws).flags(flags),
            Default::default(),
            PROMPT_STRAT.clone(),
        )
    }

    /// Rss removes a package and its dependencies which are not required by any other installed package.
    fn rss(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run(
            Cmd::new(&["choco", "uninstall", "--removedependencies"])
                .kws(kws)
                .flags(flags),
            Default::default(),
            PROMPT_STRAT.clone(),
        )
    }

    /// S installs one or more packages by name.
    fn s(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        let cmd: &[&str] = if self.cfg.needed {
            &["choco", "install"]
        } else {
            &["choco", "install", "--force"]
        };
        self.just_run(
            Cmd::new(cmd).kws(kws).flags(flags),
            Default::default(),
            PROMPT_STRAT.clone(),
        )
    }

    /// Si displays remote package information: name, version, description, etc.
    fn si(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run_default(Cmd::new(&["choco", "info"]).kws(kws).flags(flags))
    }

    /// Ss searches for package(s) by searching the expression in name, description, short description.
    fn ss(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run_default(Cmd::new(&["choco", "search"]).kws(kws).flags(flags))
    }

    /// Su updates outdated packages.
    fn su(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        let cmd: &[&str] = if kws.is_empty() {
            &["choco", "upgrade", "all"]
        } else {
            &["tlmgr", "upgrade"]
        };
        self.just_run(
            Cmd::new(cmd).kws(kws).flags(flags),
            Default::default(),
            PROMPT_STRAT.clone(),
        )
    }

    /// Suy refreshes the local package database, then updates outdated packages.
    fn suy(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.su(kws, flags)
    }
}
