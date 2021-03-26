use super::{DryRunStrategy, Pm, PmHelper, PromptStrategy, Strategies};
use crate::exec::Cmd;
use crate::{dispatch::config::Config, error::Result};
use async_trait::async_trait;
use lazy_static::lazy_static;

pub struct Chocolatey {
    pub cfg: Config,
}

lazy_static! {
    static ref PROMPT_STRAT: Strategies = Strategies {
        prompt: PromptStrategy::native_prompt(&["--yes"]),
        dry_run: DryRunStrategy::with_flags(&["--what-if"]),
        ..Default::default()
    };
    static ref CHECK_DRY_STRAT: Strategies = Strategies {
        dry_run: DryRunStrategy::with_flags(&["--what-if"]),
        ..Default::default()
    };
}

impl Chocolatey {
    async fn check_dry_run(&self, cmd: Cmd) -> Result<()> {
        self.just_run(cmd, Default::default(), &CHECK_DRY_STRAT)
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
        self.check_dry_run(
            Cmd::new(&["choco", "list", "--localonly"])
                .kws(kws)
                .flags(flags),
        )
        .await
    }

    /// Qi displays local package information: name, version, description, etc.
    async fn qi(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.si(kws, flags).await
    }

    /// Qu lists packages which have an update available.
    async fn qu(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.check_dry_run(Cmd::new(&["choco", "outdated"]).kws(kws).flags(flags))
            .await
    }

    /// R removes a single package, leaving all of its dependencies installed.
    async fn r(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run(
            Cmd::new(&["choco", "uninstall"]).kws(kws).flags(flags),
            Default::default(),
            &PROMPT_STRAT,
        )
        .await
    }

    /// Rss removes a package and its dependencies which are not required by any other installed package.
    async fn rss(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run(
            Cmd::new(&["choco", "uninstall", "--removedependencies"])
                .kws(kws)
                .flags(flags),
            Default::default(),
            &PROMPT_STRAT,
        )
        .await
    }

    /// S installs one or more packages by name.
    async fn s(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        let cmd: &[&str] = if self.cfg.needed {
            &["choco", "install"]
        } else {
            &["choco", "install", "--force"]
        };
        self.just_run(
            Cmd::new(cmd).kws(kws).flags(flags),
            Default::default(),
            &PROMPT_STRAT,
        )
        .await
    }

    /// Si displays remote package information: name, version, description, etc.
    async fn si(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.check_dry_run(Cmd::new(&["choco", "info"]).kws(kws).flags(flags))
            .await
    }

    /// Ss searches for package(s) by searching the expression in name, description, short description.
    async fn ss(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.check_dry_run(Cmd::new(&["choco", "search"]).kws(kws).flags(flags))
            .await
    }

    /// Su updates outdated packages.
    async fn su(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        let cmd: &[&str] = if kws.is_empty() {
            &["choco", "upgrade", "all"]
        } else {
            &["choco", "upgrade"]
        };
        self.just_run(
            Cmd::new(cmd).kws(kws).flags(flags),
            Default::default(),
            &PROMPT_STRAT,
        )
        .await
    }

    /// Suy refreshes the local package database, then updates outdated packages.
    async fn suy(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.su(kws, flags).await
    }
}
