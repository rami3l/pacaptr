#![doc = doc_self!()]

use std::sync::LazyLock;

use async_trait::async_trait;
use indoc::indoc;
use tap::prelude::*;

use super::{NoCacheStrategy, Pm, PmHelper, PromptStrategy, Strategy};
use crate::{config::Config, error::Result, exec::Cmd};

macro_rules! doc_self {
    () => {
        indoc! {"
            The [Advanced Package Tool](https://salsa.debian.org/apt-team/apt).
        "}
    };
}
use doc_self;

#[doc = doc_self!()]
#[derive(Debug)]
pub struct Apt {
    cfg: Config,
}

static STRAT_PROMPT: LazyLock<Strategy> = LazyLock::new(|| Strategy {
    prompt: PromptStrategy::native_no_confirm(["--yes"]),
    ..Strategy::default()
});

static STRAT_INSTALL: LazyLock<Strategy> = LazyLock::new(|| Strategy {
    prompt: PromptStrategy::native_no_confirm(["--yes"]),
    no_cache: NoCacheStrategy::Scc,
    ..Strategy::default()
});

impl Apt {
    #[must_use]
    #[allow(missing_docs)]
    pub const fn new(cfg: Config) -> Self {
        Self { cfg }
    }

    /// Returns the command used to invoke [`Apt`], eg. `apt`, `pkg`.
    #[must_use]
    fn cmd(&self) -> &str {
        self.cfg
            .default_pm
            .as_deref()
            .expect("default package manager should have been assigned before initialization")
    }
}

#[async_trait]
impl Pm for Apt {
    /// Gets the name of the package manager.
    fn name(&self) -> &'static str {
        "apt"
    }

    fn cfg(&self) -> &Config {
        &self.cfg
    }

    /// Q generates a list of installed packages.
    async fn q(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(["apt", "list", "--installed"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run(cmd))
            .await
    }

    /// Qc shows the changelog of a package.
    async fn qc(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(["apt", "changelog"]).kws(kws).flags(flags))
            .await
    }

    /// Qe lists packages installed explicitly (not as dependencies).
    async fn qe(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(["apt-mark", "showmanual"]).kws(kws).flags(flags))
            .await
    }

    /// Qi displays local package information: name, version, description, etc.
    async fn qi(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(["dpkg-query", "-s"]).kws(kws).flags(flags))
            .await
    }

    /// Qii displays local packages which require X to be installed, aka local
    /// reverse dependencies.
    async fn qii(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.sii(kws, flags).await
    }

    /// Qo queries the package which provides FILE.
    async fn qo(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(["dpkg-query", "-S"]).kws(kws).flags(flags))
            .await
    }

    /// Qp queries a package supplied through a file supplied on the command
    /// line rather than an entry in the package management database.
    async fn qp(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(["dpkg-deb", "-I"]).kws(kws).flags(flags))
            .await
    }

    /// Qs searches locally installed package for names or descriptions.
    // According to https://www.archlinux.org/pacman/pacman.8.html#_query_options_apply_to_em_q_em_a_id_qo_a,
    // when including multiple search terms, only packages with descriptions
    // matching ALL of those terms are returned.
    async fn qs(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(["dpkg-query", "-l"])
            .flags(flags)
            .pipe(|cmd| self.search_regex_with_header(cmd, kws, 4))
            .await
    }

    /// Qu lists packages which have an update available.
    async fn qu(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::with_sudo(["apt", "upgrade", "--trivial-only"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run(cmd))
            .await
    }

    /// R removes a single package, leaving all of its dependencies installed.
    async fn r(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::with_sudo(["apt", "remove"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, self.default_mode(), &STRAT_PROMPT))
            .await
    }

    /// Rn removes a package and skips the generation of configuration backup
    /// files.
    async fn rn(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::with_sudo(["apt", "purge"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, self.default_mode(), &STRAT_PROMPT))
            .await
    }

    /// Rns removes a package and its dependencies which are not required by any
    /// other installed package, and skips the generation of configuration
    /// backup files.
    async fn rns(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::with_sudo(["apt", "autoremove", "--purge"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, self.default_mode(), &STRAT_PROMPT))
            .await
    }

    /// Rs removes a package and its dependencies which are not required by any
    /// other installed package, and not explicitly installed by the user.
    async fn rs(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::with_sudo(["apt", "autoremove"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, self.default_mode(), &STRAT_PROMPT))
            .await
    }

    /// S installs one or more packages by name.
    async fn s(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        if self.cfg.needed {
            Cmd::with_sudo(&[self.cmd(), "install"][..])
        } else {
            Cmd::with_sudo(&[self.cmd(), "install", "--reinstall"][..])
        }
        .kws(kws)
        .flags(flags)
        .pipe(|cmd| self.run_with(cmd, self.default_mode(), &STRAT_INSTALL))
        .await
    }

    /// Sc removes all the cached packages that are not currently installed, and
    /// the unused sync database.
    async fn sc(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::with_sudo(["apt", "clean"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, self.default_mode(), &STRAT_PROMPT))
            .await
    }

    /// Scc removes all files from the cache.
    async fn scc(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::with_sudo(["apt", "autoclean"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, self.default_mode(), &STRAT_PROMPT))
            .await
    }

    /// Sg lists all packages belonging to the GROUP.
    async fn sg(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(if kws.is_empty() {
            ["tasksel", "--list-task"]
        } else {
            ["tasksel", "--task-packages"]
        })
        .kws(kws)
        .flags(flags)
        .pipe(|cmd| self.run(cmd))
        .await
    }

    /// Si displays remote package information: name, version, description, etc.
    async fn si(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(["apt", "show"]).kws(kws).flags(flags))
            .await
    }

    /// Sii displays packages which require X to be installed, aka reverse
    /// dependencies.
    async fn sii(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(["apt", "rdepends"]).kws(kws).flags(flags))
            .await
    }

    /// Ss searches for package(s) by searching the expression in name,
    /// description, short description.
    async fn ss(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new([self.cmd(), "search"]).kws(kws).flags(flags))
            .await
    }

    /// Su updates outdated packages.
    async fn su(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        if kws.is_empty() {
            Cmd::with_sudo(["apt", "upgrade"])
                .flags(flags)
                .pipe(|cmd| self.run_with(cmd, self.default_mode(), &STRAT_PROMPT))
                .await?;
            Cmd::with_sudo(["apt", "dist-upgrade"])
                .flags(flags)
                .pipe(|cmd| self.run_with(cmd, self.default_mode(), &STRAT_INSTALL))
                .await
        } else {
            self.s(kws, flags).await
        }
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
        Cmd::with_sudo([self.cmd(), "install", "--download-only"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, self.default_mode(), &STRAT_INSTALL))
            .await
    }

    /// Sy refreshes the local package database.
    async fn sy(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::with_sudo([self.cmd(), "update"]).flags(flags))
            .await?;
        if !kws.is_empty() {
            self.s(kws, flags).await?;
        }
        Ok(())
    }
}
