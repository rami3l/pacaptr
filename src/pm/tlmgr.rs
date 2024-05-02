//! The [TexLive Package Manager](https://www.tug.org/texlive/tlmgr.html).

use async_trait::async_trait;
use once_cell::sync::Lazy;
use tap::prelude::*;

use super::{DryRunStrategy, Pm, PmHelper, PmMode, Strategy};
use crate::{config::Config, error::Result, exec::Cmd};

/// The [TexLive Package Manager](https://www.tug.org/texlive/tlmgr.html).
#[derive(Debug)]
pub struct Tlmgr {
    cfg: Config,
}

static STRAT_CHECK_DRY: Lazy<Strategy> = Lazy::new(|| Strategy {
    dry_run: DryRunStrategy::with_flags(["--dry-run"]),
    ..Strategy::default()
});

impl Tlmgr {
    #[must_use]
    #[allow(missing_docs)]
    pub const fn new(cfg: Config) -> Self {
        Self { cfg }
    }
}

#[async_trait]
impl Pm for Tlmgr {
    /// Gets the name of the package manager.
    fn name(&self) -> &str {
        "tlmgr"
    }

    fn cfg(&self) -> &Config {
        &self.cfg
    }

    /// Q generates a list of installed packages.
    async fn q(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.qi(kws, flags).await
    }

    /// Qi displays local package information: name, version, description, etc.
    async fn qi(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(["tlmgr", "info", "--only-installed"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run(cmd))
            .await
    }

    /// Qk verifies one or more packages.
    async fn qk(&self, _kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(["tlmgr", "check", "files"]).flags(flags))
            .await
    }

    /// Ql displays files provided by local package.
    async fn ql(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(["tlmgr", "info", "--only-installed", "--list"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run(cmd))
            .await
    }

    /// R removes a single package, leaving all of its dependencies installed.
    async fn r(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(["tlmgr", "remove"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_CHECK_DRY))
            .await
    }

    /// S installs one or more packages by name.
    async fn s(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(["tlmgr", "install"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_CHECK_DRY))
            .await
    }

    /// Si displays remote package information: name, version, description, etc.
    async fn si(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(["tlmgr", "info"]).kws(kws).flags(flags))
            .await
    }

    /// Sl displays a list of all packages in all installation sources that are
    /// handled by the package management.
    async fn sl(&self, _kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(["tlmgr", "info"]).flags(flags)).await
    }

    /// Ss searches for package(s) by searching the expression in name,
    /// description, short description.
    async fn ss(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(["tlmgr", "search", "--global"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run(cmd))
            .await
    }

    /// Su updates outdated packages.
    async fn su(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(if kws.is_empty() {
            &["tlmgr", "update", "--self", "--all"][..]
        } else {
            &["tlmgr", "update", "--self"][..]
        })
        .kws(kws)
        .flags(flags)
        .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_CHECK_DRY))
        .await
    }

    /// Suy refreshes the local package database, then updates outdated
    /// packages.
    async fn suy(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.su(kws, flags).await
    }

    /// U upgrades or adds package(s) to the system and installs the required
    /// dependencies from sync repositories.
    async fn u(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(["tlmgr", "install", "--file"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_CHECK_DRY))
            .await
    }
}
