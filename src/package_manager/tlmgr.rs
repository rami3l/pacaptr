use super::{DryRunStrategy, PackageManager, Strategies};
use crate::dispatch::config::Config;
use crate::exec::Cmd;
use anyhow::Result;
use async_trait::async_trait;
use lazy_static::lazy_static;

pub struct Tlmgr {
    pub cfg: Config,
}

lazy_static! {
    static ref CHECK_DRY_STRAT: Strategies = Strategies {
        dry_run: DryRunStrategy::with_flags(&["--dry-run"]),
        ..Default::default()
    };
}

#[async_trait]
impl PackageManager for Tlmgr {
    /// Get the name of the package manager.
    fn name(&self) -> String {
        "tlmgr".into()
    }

    fn cfg(&self) -> Config {
        self.cfg.clone()
    }

    /// Q generates a list of installed packages.
    async fn q(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.qi(kws, flags).await
    }

    /// Qi displays local package information: name, version, description, etc.
    async fn qi(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run_default(
            Cmd::new(&["tlmgr", "info", "--only-installed"])
                .kws(kws)
                .flags(flags),
        )
        .await
    }

    /// Qk verifies one or more packages.
    async fn qk(&self, _kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run_default(Cmd::new(&["tlmgr", "check", "files"]).flags(flags))
            .await
    }

    /// Ql displays files provided by local package.
    async fn ql(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run_default(
            Cmd::new(&["tlmgr", "info", "--only-installed", "--list"])
                .kws(kws)
                .flags(flags),
        )
        .await
    }

    /// R removes a single package, leaving all of its dependencies installed.
    async fn r(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run(
            Cmd::new(&["tlmgr", "remove"]).kws(kws).flags(flags),
            Default::default(),
            CHECK_DRY_STRAT.clone(),
        )
        .await
    }

    /// S installs one or more packages by name.
    async fn s(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run(
            Cmd::new(&["tlmgr", "install"]).kws(kws).flags(flags),
            Default::default(),
            CHECK_DRY_STRAT.clone(),
        )
        .await
    }

    /// Si displays remote package information: name, version, description, etc.
    async fn si(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run_default(Cmd::new(&["tlmgr", "info"]).kws(kws).flags(flags))
            .await
    }

    /// Sl displays a list of all packages in all installation sources that are handled by the packages management.
    async fn sl(&self, _kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run_default(Cmd::new(&["tlmgr", "info"]).flags(flags))
            .await
    }

    /// Ss searches for package(s) by searching the expression in name, description, short description.
    async fn ss(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run_default(
            Cmd::new(&["tlmgr", "search", "--global"])
                .kws(kws)
                .flags(flags),
        )
        .await
    }

    /// Su updates outdated packages.
    async fn su(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
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
        .await
    }

    /// Suy refreshes the local package database, then updates outdated packages.
    async fn suy(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.su(kws, flags).await
    }

    /// U upgrades or adds package(s) to the system and installs the required dependencies from sync repositories.
    async fn u(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run(
            Cmd::new(&["tlmgr", "install", "--file"])
                .kws(kws)
                .flags(flags),
            Default::default(),
            CHECK_DRY_STRAT.clone(),
        )
        .await
    }
}
