use async_trait::async_trait;
use once_cell::sync::Lazy;
use tap::prelude::*;
use tokio::sync::Mutex;

use super::{NoCacheStrategy, Pm, PmHelper, PmMode, PromptStrategy, Strategy};
use crate::{
    dispatch::config::Config,
    error::Result,
    exec::{Cmd, StatusCode},
};

#[derive(Debug)]
pub struct Port {
    cfg: Config,
    code: Mutex<StatusCode>,
}

static STRAT_PROMPT: Lazy<Strategy> = Lazy::new(|| Strategy {
    prompt: PromptStrategy::CustomPrompt,
    ..Strategy::default()
});

static STRAT_INSTALL: Lazy<Strategy> = Lazy::new(|| Strategy {
    prompt: PromptStrategy::CustomPrompt,
    no_cache: NoCacheStrategy::Scc,
    ..Strategy::default()
});

impl Port {
    #[must_use]
    #[allow(missing_docs)]
    pub fn new(cfg: Config) -> Self {
        Port {
            cfg,
            code: Mutex::default(),
        }
    }
}

#[async_trait]
impl Pm for Port {
    /// Gets the name of the package manager.
    fn name(&self) -> &str {
        "port"
    }

    fn cfg(&self) -> &Config {
        &self.cfg
    }

    async fn code(&self) -> StatusCode {
        *self.code.lock().await
    }

    async fn set_code(&self, to: StatusCode) {
        *self.code.lock().await = to;
    }

    /// Q generates a list of installed packages.
    async fn q(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(&["port", "installed"]).kws(kws).flags(flags))
            .await
    }

    /// Qc shows the changelog of a package.
    async fn qc(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(&["port", "log"]).kws(kws).flags(flags))
            .await
    }

    /// Qi displays local package information: name, version, description, etc.
    async fn qi(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.si(kws, flags).await
    }

    /// Ql displays files provided by local package.
    async fn ql(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(&["port", "contents"]).kws(kws).flags(flags))
            .await
    }

    /// Qo queries the package which provides FILE.
    async fn qo(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(&["port", "provides"]).kws(kws).flags(flags))
            .await
    }

    /// Qs searches locally installed package for names or descriptions.
    // According to https://www.archlinux.org/pacman/pacman.8.html#_query_options_apply_to_em_q_em_a_id_qo_a,
    // when including multiple search terms, only packages with descriptions
    // matching ALL of those terms are returned.
    async fn qs(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(&["port", "-v", "installed"]).kws(kws).flags(flags))
            .await
    }

    /// Qu lists packages which have an update available.
    async fn qu(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(&["port", "outdated"]).kws(kws).flags(flags))
            .await
    }

    /// R removes a single package, leaving all of its dependencies installed.
    async fn r(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::with_sudo(&["port", "uninstall"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_PROMPT))
            .await
    }

    /// Rss removes a package and its dependencies which are not required by any
    /// other installed package.
    async fn rss(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::with_sudo(&["port", "uninstall", "--follow-dependencies"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_PROMPT))
            .await
    }

    /// S installs one or more packages by name.
    async fn s(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::with_sudo(&["port", "install"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_INSTALL))
            .await
    }

    /// Sc removes all the cached packages that are not currently installed, and
    /// the unused sync database.
    async fn sc(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::with_sudo(if flags.is_empty() {
            &["port", "clean", "--all", "inactive"]
        } else {
            &["port", "clean", "--all"]
        })
        .kws(kws)
        .flags(flags)
        .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_PROMPT))
        .await
    }

    /// Scc removes all files from the cache.
    async fn scc(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::with_sudo(if flags.is_empty() {
            &["port", "clean", "--all", "installed"]
        } else {
            &["port", "clean", "--all"]
        })
        .kws(kws)
        .flags(flags)
        .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_PROMPT))
        .await
    }

    /// Si displays remote package information: name, version, description, etc.
    async fn si(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(&["port", "info"]).kws(kws).flags(flags))
            .await
    }

    /// Ss searches for package(s) by searching the expression in name,
    /// description, short description.
    async fn ss(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(&["port", "search"]).kws(kws).flags(flags))
            .await
    }

    /// Su updates outdated packages.
    async fn su(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::with_sudo(if flags.is_empty() {
            &["port", "upgrade", "outdated"]
        } else {
            &["port", "upgrade"]
        })
        .kws(kws)
        .flags(flags)
        .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_INSTALL))
        .await
    }

    /// Suy refreshes the local package database, then updates outdated
    /// packages.
    async fn suy(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.sy(&[], flags).await?;
        self.su(kws, flags).await
    }

    /// Sy refreshes the local package database.
    async fn sy(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(&["port", "selfupdate"]).flags(flags))
            .await?;
        if !kws.is_empty() {
            self.s(kws, flags).await?;
        }
        Ok(())
    }
}
