#![doc = docs_self!()]

use async_trait::async_trait;
use indoc::indoc;
use once_cell::sync::Lazy;
use tap::prelude::*;
use which::which;

use super::{NoCacheStrategy, Pm, PmHelper, PmMode, PromptStrategy, Strategy};
use crate::{config::Config, error::Result, exec::Cmd};

macro_rules! docs_self {
    () => {
        indoc! {"
            The [Scoop CLI Installer](https://scoop.sh/).
        "}
    };
}
use docs_self;

#[doc = docs_self!()]
#[derive(Debug)]
pub struct Scoop {
    cfg: Config,
    shell: String,
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

impl Scoop {
    #[must_use]
    #[allow(missing_docs)]
    pub fn new(cfg: Config) -> Self {
        let shell = which("pwsh").and(Ok("pwsh")).unwrap_or("powershell");
        Self::with_shell(cfg, shell)
    }

    #[must_use]
    #[allow(missing_docs)]
    pub fn with_shell(cfg: Config, shell: &str) -> Self {
        Self {
            cfg,
            shell: shell.to_owned(),
        }
    }
}

// Windows is so special! It's better not to "sudo" automatically.
#[async_trait]
impl Pm for Scoop {
    /// Gets the name of the package manager.
    fn name(&self) -> &str {
        "scoop"
    }

    fn cfg(&self) -> &Config {
        &self.cfg
    }

    /// Q generates a list of installed packages.
    async fn q(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        if kws.is_empty() {
            self.run(Cmd::new([&self.shell, "-Command", "scoop", "list"]).flags(flags))
                .await
        } else {
            self.qs(kws, flags).await
        }
    }

    /// Qi displays local package information: name, version, description, etc.
    async fn qi(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.si(kws, flags).await
    }

    /// Qs searches locally installed package for names or descriptions.
    // According to https://www.archlinux.org/pacman/pacman.8.html#_query_options_apply_to_em_q_em_a_id_qo_a,
    // when including multiple search terms, only packages with descriptions
    // matching ALL of those terms are returned.
    async fn qs(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new([&self.shell, "-Command", "scoop", "list"])
            .flags(flags)
            .pipe(|cmd| self.search_regex_with_header(cmd, kws, 4))
            .await
    }

    /// Qu lists packages which have an update available.
    async fn qu(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new([&self.shell, "-Command", "scoop", "status"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run(cmd))
            .await
    }

    /// R removes a single package, leaving all of its dependencies installed.
    async fn r(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new([&self.shell, "-Command", "scoop", "uninstall"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_PROMPT))
            .await
    }

    /// Rn removes a package and skips the generation of configuration backup
    /// files.
    async fn rn(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new([&self.shell, "-Command", "scoop", "uninstall", "--purge"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_PROMPT))
            .await
    }

    /// S installs one or more packages by name.
    async fn s(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new([&self.shell, "-Command", "scoop", "install"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_INSTALL))
            .await
    }

    /// Sc removes all the cached packages that are not currently installed, and
    /// the unused sync database.
    async fn sc(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new([&self.shell, "-Command", "scoop", "cache", "rm"])
            .kws(if kws.is_empty() { &["*"][..] } else { kws })
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_PROMPT))
            .await
    }

    /// Scc removes all files from the cache.
    async fn scc(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.sc(kws, flags).await
    }

    /// Si displays remote package information: name, version, description, etc.
    async fn si(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new([&self.shell, "-Command", "scoop", "info"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run(cmd))
            .await
    }

    /// Ss searches for package(s) by searching the expression in name,
    /// description, short description.
    async fn ss(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new([&self.shell, "-Command", "scoop", "search"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run(cmd))
            .await
    }

    /// Su updates outdated packages.
    async fn su(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new([&self.shell, "-Command", "scoop", "update"])
            .kws(if kws.is_empty() { &["*"][..] } else { kws })
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
        self.run(Cmd::new([&self.shell, "-Command", "scoop", "update"]).flags(flags))
            .await?;
        if !kws.is_empty() {
            self.s(kws, flags).await?;
        }
        Ok(())
    }
}
