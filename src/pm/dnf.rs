#![doc = doc_self!()]

use std::sync::LazyLock;

use async_trait::async_trait;
use futures::prelude::*;
use indoc::indoc;
use tap::prelude::*;

use super::{NoCacheStrategy, Pm, PmHelper, PmMode, PromptStrategy, Strategy};
use crate::{config::Config, error::Result, exec::Cmd};

macro_rules! doc_self {
    () => {
        indoc! {"
            The [Dandified YUM](https://github.com/rpm-software-management/dnf).
        "}
    };
}
use doc_self;

#[doc = doc_self!()]
#[derive(Debug)]
pub struct Dnf {
    cfg: Config,
}

static STRAT_PROMPT: LazyLock<Strategy> = LazyLock::new(|| Strategy {
    prompt: PromptStrategy::native_no_confirm(["-y"]),
    ..Strategy::default()
});

static STRAT_PROMPT_CUSTOM: LazyLock<Strategy> = LazyLock::new(|| Strategy {
    prompt: PromptStrategy::CustomPrompt,
    ..Strategy::default()
});

static STRAT_INSTALL: LazyLock<Strategy> = LazyLock::new(|| Strategy {
    prompt: PromptStrategy::native_no_confirm(["-y"]),
    no_cache: NoCacheStrategy::Sccc,
    ..Strategy::default()
});

impl Dnf {
    #[must_use]
    #[allow(missing_docs)]
    pub const fn new(cfg: Config) -> Self {
        Self { cfg }
    }
}

#[async_trait]
impl Pm for Dnf {
    /// Gets the name of the package manager.
    fn name(&self) -> &str {
        "dnf"
    }

    fn cfg(&self) -> &Config {
        &self.cfg
    }

    /// Q generates a list of installed packages.
    async fn q(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        if kws.is_empty() {
            self.run(Cmd::new(["rpm", "-qa", "--qf", "%{NAME} %{VERSION}\\n"]).flags(flags))
                .await
        } else {
            self.qs(kws, flags).await
        }
    }

    /// Qc shows the changelog of a package.
    async fn qc(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(["rpm", "-q", "--changelog"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run(cmd))
            .await
    }

    /// Qe lists packages installed explicitly (not as dependencies).
    async fn qe(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(["dnf", "repoquery", "--userinstalled"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run(cmd))
            .await
    }

    /// Qi displays local package information: name, version, description, etc.
    async fn qi(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(["dnf", "info", "--installed"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run(cmd))
            .await
    }

    /// Qii displays local packages which require X to be installed, aka local
    /// reverse dependencies.
    async fn qii(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(["dnf", "repoquery", "--installed", "--whatdepends"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run(cmd))
            .await
    }

    /// Ql displays files provided by local package.
    async fn ql(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(["rpm", "-ql"]).kws(kws).flags(flags))
            .await
    }

    /// Qm lists packages that are installed but are not available in any
    /// installation source (anymore).
    async fn qm(&self, _kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(["dnf", "list", "--extras"]).flags(flags))
            .await
    }

    /// Qo queries the package which provides FILE.
    async fn qo(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(["rpm", "-qf"]).kws(kws).flags(flags))
            .await
    }

    /// Qp queries a package supplied through a file supplied on the command
    /// line rather than an entry in the package management database.
    async fn qp(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(["rpm", "-qip"]).kws(kws).flags(flags))
            .await
    }

    /// Qs searches locally installed package for names or descriptions.
    // According to https://www.archlinux.org/pacman/pacman.8.html#_query_options_apply_to_em_q_em_a_id_qo_a,
    // when including multiple search terms, only packages with descriptions
    // matching ALL of those terms are returned.
    // TODO: Is this right?
    async fn qs(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.search_regex(Cmd::new(["rpm", "-qa"]).flags(flags), kws)
            .await
    }

    /// Qu lists packages which have an update available.
    async fn qu(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(["dnf", "list", "updates"]).kws(kws).flags(flags))
            .await
    }

    /// R removes a single package, leaving all of its dependencies installed.
    async fn r(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::with_sudo(["dnf", "remove"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_PROMPT))
            .await
    }

    /// S installs one or more packages by name.
    async fn s(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::with_sudo(["dnf", "install"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_INSTALL))
            .await
    }

    /// Sc removes all the cached packages that are not currently installed, and
    /// the unused sync database.
    async fn sc(&self, _kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(["dnf", "clean", "expire-cache"])
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_PROMPT_CUSTOM))
            .await
    }

    /// Scc removes all files from the cache.
    async fn scc(&self, _kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(["dnf", "clean", "packages"])
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_PROMPT_CUSTOM))
            .await
    }

    /// Sccc performs a deeper cleaning of the cache than `Scc` (if applicable).
    async fn sccc(&self, _kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(["dnf", "clean", "all"])
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_PROMPT_CUSTOM))
            .await
    }

    /// Si displays remote package information: name, version, description, etc.

    async fn si(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(["dnf", "info"]).kws(kws).flags(flags))
            .await
    }

    /// Sii displays packages which require X to be installed, aka reverse
    /// dependencies.
    async fn sii(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(["dnf", "repoquery", "--whatdepends"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run(cmd))
            .await
    }

    /// Sg lists all packages belonging to the GROUP.
    async fn sg(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(if kws.is_empty() {
            ["dnf", "group", "list"]
        } else {
            ["dnf", "group", "info"]
        })
        .kws(kws)
        .flags(flags)
        .pipe(|cmd| self.run(cmd))
        .await
    }

    /// Sl displays a list of all packages in all installation sources that are
    /// handled by the package management.
    async fn sl(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(["dnf", "list", "--available"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run(cmd))
            .await
    }

    /// Ss searches for package(s) by searching the expression in name,
    /// description, short description.
    async fn ss(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(["dnf", "search"]).kws(kws).flags(flags))
            .await
    }

    /// Su updates outdated packages.
    async fn su(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::with_sudo(["dnf", "upgrade"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_INSTALL))
            .await
    }

    /// Suy refreshes the local package database, then updates outdated
    /// packages.
    async fn suy(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.su(kws, flags).await
    }

    /// Sw retrieves all packages from the server, but does not install/upgrade
    /// anything.
    async fn sw(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::with_sudo(["dnf", "install", "--downloadonly"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_INSTALL))
            .await
    }

    /// Sy refreshes the local package database.
    async fn sy(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.sc(&[], flags).await?;
        self.run(Cmd::new(["dnf", "check-update"]).flags(flags))
            .await?;
        if !kws.is_empty() {
            self.s(kws, flags).await?;
        }
        Ok(())
    }

    /// U upgrades or adds package(s) to the system and installs the required
    /// dependencies from sync repositories.
    async fn u(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.s(kws, flags).await
    }
}
