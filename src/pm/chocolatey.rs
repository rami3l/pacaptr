use super::{DryRunStrategy, Pm, PmHelper, PromptStrategy, Strategy};
use crate::exec::Cmd;
use crate::{dispatch::config::Config, error::Result};
use async_trait::async_trait;
use once_cell::sync::Lazy;
use tap::prelude::*;

pub struct Chocolatey {
    pub cfg: Config,
}

static STRAT_PROMPT: Lazy<Strategy> = Lazy::new(|| Strategy {
    prompt: PromptStrategy::native_no_confirm(&["--yes"]),
    dry_run: DryRunStrategy::with_flags(&["--what-if"]),
    ..Default::default()
});

static STRAT_CHECK_DRY: Lazy<Strategy> = Lazy::new(|| Strategy {
    dry_run: DryRunStrategy::with_flags(&["--what-if"]),
    ..Default::default()
});

impl Chocolatey {
    async fn check_dry(&self, cmd: Cmd) -> Result<()> {
        self.run_with(cmd, Default::default(), &STRAT_CHECK_DRY)
            .await
    }
}

// Windows is so special! It's better not to "sudo" automatically.
#[async_trait]
impl Pm for Chocolatey {
    /// Gets the name of the package manager.
    fn name(&self) -> String {
        "choco".into()
    }

    fn cfg(&self) -> &Config {
        &self.cfg
    }

    /// Q generates a list of installed packages.
    async fn q(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(&["choco", "list", "--localonly"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.check_dry(cmd))
            .await
    }

    /// Qi displays local package information: name, version, description, etc.
    async fn qi(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.si(kws, flags).await
    }

    /// Qu lists packages which have an update available.
    async fn qu(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.check_dry(Cmd::new(&["choco", "outdated"]).kws(kws).flags(flags))
            .await
    }

    /// R removes a single package, leaving all of its dependencies installed.
    async fn r(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(&["choco", "uninstall"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, Default::default(), &STRAT_PROMPT))
            .await
    }

    /// Rss removes a package and its dependencies which are not required by any other installed package.
    async fn rss(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(&["choco", "uninstall", "--removedependencies"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, Default::default(), &STRAT_PROMPT))
            .await
    }

    /// S installs one or more packages by name.
    async fn s(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(if self.cfg.needed {
            &["choco", "install"]
        } else {
            &["choco", "install", "--force"]
        })
        .kws(kws)
        .flags(flags)
        .pipe(|cmd| self.run_with(cmd, Default::default(), &STRAT_PROMPT))
        .await
    }

    /// Si displays remote package information: name, version, description, etc.
    async fn si(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.check_dry(Cmd::new(&["choco", "info"]).kws(kws).flags(flags))
            .await
    }

    /// Ss searches for package(s) by searching the expression in name, description, short description.
    async fn ss(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.check_dry(Cmd::new(&["choco", "search"]).kws(kws).flags(flags))
            .await
    }

    /// Su updates outdated packages.
    async fn su(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(if kws.is_empty() {
            &["choco", "upgrade", "all"]
        } else {
            &["choco", "upgrade"]
        })
        .kws(kws)
        .flags(flags)
        .pipe(|cmd| self.run_with(cmd, Default::default(), &STRAT_PROMPT))
        .await
    }

    /// Suy refreshes the local package database, then updates outdated packages.
    async fn suy(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.su(kws, flags).await
    }
}
