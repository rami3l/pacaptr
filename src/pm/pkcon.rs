#![doc = doc_self!()]

use std::sync::LazyLock;

use async_trait::async_trait;
use futures::prelude::*;
use indoc::indoc;
use tap::prelude::*;

use super::{Pm, PmHelper, PromptStrategy, Strategy};
use crate::{config::Config, error::Result, exec::Cmd};

macro_rules! doc_self {
    () => {
        indoc! {"
            The [PackageKit Console Client](https://www.freedesktop.org/software/PackageKit).
        "}
    };
}
use doc_self;

#[doc = doc_self!()]
#[derive(Debug)]
pub struct Pkcon {
    cfg: Config,
}

static STRAT_PROMPT: LazyLock<Strategy> = LazyLock::new(|| Strategy {
    prompt: PromptStrategy::native_no_confirm(["-y"]),
    ..Strategy::default()
});

impl Pkcon {
    #[must_use]
    #[allow(missing_docs)]
    pub const fn new(cfg: Config) -> Self {
        Self { cfg }
    }
}

#[async_trait]
impl Pm for Pkcon {
    /// Gets the name of the package manager.
    fn name(&self) -> &'static str {
        "pkcon"
    }

    fn cfg(&self) -> &Config {
        &self.cfg
    }

    /// Q generates a list of installed packages.
    async fn q(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        if kws.is_empty() {
            Cmd::new(["pkcon", "get-packages", "--filter", "installed"])
                .kws(kws)
                .flags(flags)
                .pipe(|cmd| self.run(cmd))
                .await
        } else {
            self.qs(kws, flags).await
        }
    }

    /// Qc shows the changelog of a package.
    async fn qc(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(["pkcon", "get-update-detail"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run(cmd))
            .await
    }

    /// Qi displays local package information: name, version, description, etc.
    async fn qi(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.si(kws, flags).await
    }

    /// Qii displays local packages which require X to be installed, aka local
    /// reverse dependencies.
    async fn qii(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.sii(kws, flags).await
    }

    /// Ql displays files provided by local package.
    async fn ql(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(["pkcon", "get-files"]).kws(kws).flags(flags))
            .await
    }

    /// Qo queries the package which provides FILE.
    async fn qo(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(["pkcon", "what-provides"]).kws(kws).flags(flags))
            .await
    }

    /// Qs searches locally installed package for names or descriptions.
    // According to https://www.archlinux.org/pacman/pacman.8.html#_query_options_apply_to_em_q_em_a_id_qo_a,
    // when including multiple search terms, only packages with descriptions
    // matching ALL of those terms are returned.
    async fn qs(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(["pkcon", "get-packages", "--filter", "installed"])
            .flags(flags)
            .pipe(|cmd| self.search_regex(cmd, kws))
            .await
    }

    /// Qu lists packages which have an update available.
    async fn qu(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::with_sudo(["pkcon", "get-updates"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run(cmd))
            .await
    }

    /// R removes a single package, leaving all of its dependencies installed.
    async fn r(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        stream::iter(kws)
            .map(Ok)
            .try_for_each(|kw| {
                Cmd::with_sudo(["pkcon", "remove"])
                    .kws([kw])
                    .flags(flags)
                    .pipe(|cmd| self.run_with(cmd, self.default_mode(), &STRAT_PROMPT))
            })
            .await
    }

    /// Rs removes a package and its dependencies which are not required by any
    /// other installed package, and not explicitly installed by the user.
    async fn rs(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        stream::iter(kws)
            .map(Ok)
            .try_for_each(|kw| {
                Cmd::with_sudo(["pkcon", "remove", "--autoremove"])
                    .kws([kw])
                    .flags(flags)
                    .pipe(|cmd| self.run_with(cmd, self.default_mode(), &STRAT_PROMPT))
            })
            .await
    }

    /// S installs one or more packages by name.
    async fn s(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::with_sudo(if self.cfg.needed {
            &["pkcon", "install"][..]
        } else {
            &["pkcon", "install", "--allow-reinstall"][..]
        })
        .kws(kws)
        .flags(flags)
        .pipe(|cmd| self.run_with(cmd, self.default_mode(), &STRAT_PROMPT))
        .await
    }

    /// Si displays remote package information: name, version, description, etc.
    async fn si(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(["pkcon", "get-details"]).kws(kws).flags(flags))
            .await
    }

    /// Sii displays packages which require X to be installed, aka reverse
    /// dependencies.
    async fn sii(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(["pkcon", "required-by"]).kws(kws).flags(flags))
            .await
    }

    /// Ss searches for package(s) by searching the expression in name,
    /// description, short description.
    async fn ss(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(["pkcon", "search", "name"]).kws(kws).flags(flags))
            .await
    }

    /// Su updates outdated packages.
    async fn su(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::with_sudo(["pkcon", "update"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, self.default_mode(), &STRAT_PROMPT))
            .await
    }

    /// Suy refreshes the local package database, then updates outdated
    /// packages.
    async fn suy(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.sy(&[], flags).await?;
        self.su(kws, flags).await
    }

    /// Sw retrieves all packages from the server, but does not install/upgrade
    /// anything.
    async fn sw(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::with_sudo(["pkcon", "install", "--only-download"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, self.default_mode(), &STRAT_PROMPT))
            .await
    }

    /// Sy refreshes the local package database.
    async fn sy(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::with_sudo(["pkcon", "refresh"]).flags(flags))
            .await?;
        if !kws.is_empty() {
            self.s(kws, flags).await?;
        }
        Ok(())
    }
}
