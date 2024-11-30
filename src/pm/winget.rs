#![doc = doc_self!()]

use std::sync::LazyLock;

use async_trait::async_trait;
use indoc::indoc;
use tap::prelude::*;

use super::{Pm, PmHelper, PromptStrategy, Strategy};
use crate::{config::Config, error::Result, exec::Cmd};

macro_rules! doc_self {
    () => {
        indoc! {"
            The [Windows Package Manager CLI](https://github.com/microsoft/winget-cli).
        "}
    };
}
use doc_self;

#[doc = doc_self!()]
#[derive(Debug)]
pub struct Winget {
    cfg: Config,
}

static STRAT_PROMPT: LazyLock<Strategy> = LazyLock::new(|| Strategy {
    prompt: PromptStrategy::CustomPrompt,
    ..Strategy::default()
});

static STRAT_INSTALL: LazyLock<Strategy> = LazyLock::new(|| Strategy {
    prompt: PromptStrategy::CustomPrompt,
    ..Strategy::default()
});

impl Winget {
    #[must_use]
    #[allow(missing_docs)]
    pub const fn new(cfg: Config) -> Self {
        Self { cfg }
    }
}

// Windows is so special! It's better not to "sudo" automatically.
#[async_trait]
impl Pm for Winget {
    /// Gets the name of the package manager.
    fn name(&self) -> &'static str {
        "winget"
    }

    fn cfg(&self) -> &Config {
        &self.cfg
    }

    /// Q generates a list of installed packages.
    async fn q(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        if kws.is_empty() {
            self.run(Cmd::new(["winget", "list", "--accept-source-agreements"]).flags(flags))
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
        Cmd::new(["winget", "list", "--accept-source-agreements"])
            .flags(flags)
            .pipe(|cmd| self.search_regex(cmd, kws))
            .await
    }

    /// R removes a single package, leaving all of its dependencies installed.
    async fn r(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(["winget", "uninstall", "--accept-source-agreements"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, self.default_mode(), &STRAT_PROMPT))
            .await
    }

    /// Rn removes a package and skips the generation of configuration backup
    /// files.
    async fn rn(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new([
            "winget",
            "uninstall",
            "--accept-source-agreements",
            "--purge",
        ])
        .kws(kws)
        .flags(flags)
        .pipe(|cmd| self.run_with(cmd, self.default_mode(), &STRAT_PROMPT))
        .await
    }

    /// S installs one or more packages by name.
    async fn s(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new([
            "winget",
            "install",
            "--accept-package-agreements",
            "--accept-source-agreements",
        ])
        .kws(kws)
        .flags(flags)
        .pipe(|cmd| self.run_with(cmd, self.default_mode(), &STRAT_INSTALL))
        .await
    }

    /// Si displays remote package information: name, version, description, etc.
    async fn si(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(["winget", "show", "--accept-source-agreements"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run(cmd))
            .await
    }

    /// Ss searches for package(s) by searching the expression in name,
    /// description, short description.
    async fn ss(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(["winget", "search", "--accept-source-agreements"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run(cmd))
            .await
    }

    /// Sy refreshes the local package database.
    async fn sy(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(["winget", "source", "update", "--accept-source-agreements"])
            .flags(flags)
            .pipe(|cmd| self.run(cmd))
            .await?;
        if !kws.is_empty() {
            self.s(kws, flags).await?;
        }
        Ok(())
    }

    /// Su updates outdated packages.
    async fn su(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new([
            "winget",
            "upgrade",
            "--accept-package-agreements",
            "--accept-source-agreements",
        ])
        .kws(if kws.is_empty() { &["--all"][..] } else { kws })
        .flags(flags)
        .pipe(|cmd| self.run_with(cmd, self.default_mode(), &STRAT_INSTALL))
        .await
    }

    /// Suy refreshes the local package database, then updates outdated
    /// packages.
    async fn suy(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.sy(&[], flags).await?;
        self.su(kws, flags).await
    }
}
