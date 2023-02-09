#![doc = docs_self!()]

use async_trait::async_trait;
use indoc::indoc;
use itertools::Itertools;
use once_cell::sync::Lazy;
use tap::prelude::*;

use super::{NoCacheStrategy, Pm, PmHelper, PmMode, PromptStrategy, Strategy};
use crate::{dispatch::Config, error::Result, exec::Cmd};

macro_rules! docs_self {
    () => {
        indoc! {"
            The [Portage Package Manager](https://wiki.gentoo.org/wiki/Portage).
        "}
    };
}

#[doc = docs_self!()]
#[derive(Debug)]
pub struct Emerge {
    cfg: Config,
}

static STRAT_ASK: Lazy<Strategy> = Lazy::new(|| Strategy {
    prompt: PromptStrategy::native_confirm(["--ask"]),
    ..Strategy::default()
});

static STRAT_INTERACTIVE: Lazy<Strategy> = Lazy::new(|| Strategy {
    prompt: PromptStrategy::native_confirm(["--interactive"]),
    ..Strategy::default()
});

static STRAT_INSTALL: Lazy<Strategy> = Lazy::new(|| Strategy {
    prompt: PromptStrategy::native_confirm(["--ask"]),
    no_cache: NoCacheStrategy::Scc,
    ..Strategy::default()
});

impl Emerge {
    #[must_use]
    #[allow(missing_docs)]
    pub const fn new(cfg: Config) -> Self {
        Self { cfg }
    }
}

#[async_trait]
impl Pm for Emerge {
    /// Gets the name of the package manager.
    fn name(&self) -> &str {
        "emerge"
    }

    fn cfg(&self) -> &Config {
        &self.cfg
    }

    /// Q generates a list of installed packages.
    async fn q(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.qs(kws, flags).await
    }

    /// Qi displays local package information: name, version, description, etc.
    async fn qi(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.si(kws, flags).await
    }

    /// Ql displays files provided by local package.
    async fn ql(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(["qlist"]).kws(kws).flags(flags)).await
    }

    /// Qo queries the package which provides FILE.
    async fn qo(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(["qfile"]).kws(kws).flags(flags)).await
    }

    /// Qs searches locally installed package for names or descriptions.
    // According to https://www.archlinux.org/pacman/pacman.8.html#_query_options_apply_to_em_q_em_a_id_qo_a,
    // when including multiple search terms, only packages with descriptions
    // matching ALL of those terms are returned.
    async fn qs(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(["qlist", "-I"]).kws(kws).flags(flags))
            .await
    }

    /// Qu lists packages which have an update available.
    async fn qu(&self, _kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(["emerge", "-uDNp", "@world"]).flags(flags))
            .await
    }

    /// R removes a single package, leaving all of its dependencies installed.
    async fn r(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::with_sudo(["emerge", "--unmerge"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_ASK))
            .await
    }

    /// Rs removes a package and its dependencies which are not required by any
    /// other installed package, and not explicitly installed by the user.
    async fn rs(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::with_sudo(["emerge", "--depclean"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_ASK))
            .await
    }

    /// S installs one or more packages by name.
    async fn s(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::with_sudo(["emerge"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_INSTALL))
            .await
    }

    /// Sc removes all the cached packages that are not currently installed, and
    /// the unused sync database.
    async fn sc(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::with_sudo(["eclean-dist"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_INTERACTIVE))
            .await
    }

    /// Scc removes all files from the cache.
    async fn scc(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.sc(kws, flags).await
    }

    /// Si displays remote package information: name, version, description, etc.
    async fn si(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        let kws = kws.iter().map(|kw| format!("^{kw}$")).collect_vec();
        self.run(Cmd::new(["emerge", "-s"]).kws(kws).flags(flags))
            .await
    }

    /// Ss searches for package(s) by searching the expression in name,
    /// description, short description.
    async fn ss(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(["qsearch"]).kws(kws).flags(flags)).await
    }

    /// Su updates outdated packages.
    async fn su(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::with_sudo(["emerge", "-uDN"])
            .kws(if kws.is_empty() { &["@world"][..] } else { kws })
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
        self.run(Cmd::with_sudo(["emerge", "--sync"]).flags(flags))
            .await?;
        if !kws.is_empty() {
            self.s(kws, flags).await?;
        }
        Ok(())
    }
}
