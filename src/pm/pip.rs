#![doc = docs_self!()]

use async_trait::async_trait;
use indoc::indoc;
use once_cell::sync::Lazy;
use tap::prelude::*;

use super::{Pm, PmHelper, PmMode, PromptStrategy, Strategy};
use crate::{
    dispatch::Config,
    error::{Error, Result},
    exec::Cmd,
};

macro_rules! docs_self {
    () => {
        indoc! {"
            The [Python Package Installer](https://pip.pypa.io/).
        "}
    };
}

#[doc = docs_self!()]
#[derive(Debug)]
pub(crate) struct Pip {
    cfg: Config,
}

static STRAT_PROMPT: Lazy<Strategy> = Lazy::new(|| Strategy {
    prompt: PromptStrategy::CustomPrompt,
    ..Strategy::default()
});

static STRAT_UNINSTALL: Lazy<Strategy> = Lazy::new(|| Strategy {
    prompt: PromptStrategy::native_no_confirm(["-y"]),
    ..Strategy::default()
});

impl Pip {
    /// Returns the command used to invoke [`Pip`], eg. `pip`, `pip3`.
    #[must_use]
    fn cmd(&self) -> &str {
        self.cfg
            .default_pm
            .as_deref()
            .expect("default package manager should have been assigned before initialization")
    }
}

impl Pip {
    #[must_use]
    #[allow(missing_docs)]
    pub(crate) fn new(cfg: Config) -> Self {
        Pip { cfg }
    }
}

#[async_trait]
impl Pm for Pip {
    /// Gets the name of the package manager.
    fn name(&self) -> &str {
        "pip"
    }

    fn cfg(&self) -> &Config {
        &self.cfg
    }

    /// Q generates a list of installed packages.
    async fn q(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        if kws.is_empty() {
            self.run(Cmd::new([self.cmd(), "list"]).flags(flags)).await
        } else {
            self.qs(kws, flags).await
        }
    }

    /// Qi displays local package information: name, version, description, etc.
    async fn qi(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new([self.cmd(), "show"]).kws(kws).flags(flags))
            .await
    }

    /// Qs searches locally installed package for names or descriptions.
    // According to https://www.archlinux.org/pacman/pacman.8.html#_query_options_apply_to_em_q_em_a_id_qo_a,
    // when including multiple search terms, only packages with descriptions
    // matching ALL of those terms are returned.
    async fn qs(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.search_regex(Cmd::new([self.cmd(), "list"]).flags(flags), kws)
            .await
    }

    /// Qu lists packages which have an update available.
    async fn qu(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new([self.cmd(), "list", "--outdated"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run(cmd))
            .await
    }

    /// R removes a single package, leaving all of its dependencies installed.
    async fn r(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new([self.cmd(), "uninstall"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_UNINSTALL))
            .await
    }

    /// S installs one or more packages by name.
    async fn s(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new([self.cmd(), "install"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_PROMPT))
            .await
    }

    /// Sc removes all the cached packages that are not currently installed, and
    /// the unused sync database.
    async fn sc(&self, _kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new([self.cmd(), "cache", "purge"]).flags(flags))
            .await
    }

    /// Su updates outdated packages.
    async fn su(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        if kws.is_empty() {
            return Err(Error::OperationUnimplementedError {
                op: "su".into(),
                pm: self.name().into(),
            });
        }
        Cmd::new([self.cmd(), "install", "--upgrade"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run(cmd))
            .await
    }

    /// Sw retrieves all packages from the server, but does not install/upgrade
    /// anything.
    async fn sw(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new([self.cmd(), "download"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run(cmd))
            .await
    }
}
