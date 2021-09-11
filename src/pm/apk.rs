#![doc = docs_self!()]

use async_trait::async_trait;
use indoc::indoc;
use once_cell::sync::Lazy;
use tap::prelude::*;

use super::{NoCacheStrategy, Pm, PmHelper, PmMode, PromptStrategy, Strategy};
use crate::{
    dispatch::Config,
    error::Result,
    exec::{self, Cmd},
    print::{self, PROMPT_RUN},
};

macro_rules! docs_self {
    () => {
        indoc! {r"
            The [Alpine Linux package management system](https://wiki.alpinelinux.org/wiki/Alpine_Linux_package_management).
        "}
    };
}

#[doc = docs_self!()]
#[derive(Debug)]
pub(crate) struct Apk {
    cfg: Config,
}

static STRAT_PROMPT: Lazy<Strategy> = Lazy::new(|| Strategy {
    prompt: PromptStrategy::CustomPrompt,
    ..Strategy::default()
});

static STRAT_INSTALL: Lazy<Strategy> = Lazy::new(|| Strategy {
    prompt: PromptStrategy::CustomPrompt,
    no_cache: NoCacheStrategy::with_flags(&["--no-cache"]),
    ..Strategy::default()
});

impl Apk {
    #[must_use]
    #[allow(missing_docs)]
    pub(crate) fn new(cfg: Config) -> Self {
        Apk { cfg }
    }
}

#[async_trait]
impl Pm for Apk {
    /// Gets the name of the package manager.
    fn name(&self) -> &str {
        "apk"
    }

    fn cfg(&self) -> &Config {
        &self.cfg
    }

    /// Q generates a list of installed packages.
    async fn q(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        if kws.is_empty() {
            self.run(Cmd::new(&["apk", "info"]).flags(flags)).await
        } else {
            self.qs(kws, flags).await
        }
    }

    /// Qi displays local package information: name, version, description, etc.
    async fn qi(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.si(kws, flags).await
    }

    /// Ql displays files provided by local package.
    async fn ql(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(&["apk", "info", "-L"]).kws(kws).flags(flags))
            .await
    }

    /// Qo queries the package which provides FILE.
    async fn qo(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(&["apk", "info", "--who-owns"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run(cmd))
            .await
    }

    /// Qs searches locally installed package for names or descriptions.
    // According to https://www.archlinux.org/pacman/pacman.8.html#_query_options_apply_to_em_q_em_a_id_qo_a,
    // when including multiple search terms, only packages with descriptions
    // matching ALL of those terms are returned.
    async fn qs(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        let cmd = Cmd::new(&["apk", "info", "-d"]).flags(flags);
        if !self.cfg.dry_run {
            print::print_cmd(&cmd, PROMPT_RUN);
        }
        let out_bytes = self
            .check_output(cmd, PmMode::Mute, &Strategy::default())
            .await?;
        exec::grep_print(&String::from_utf8(out_bytes)?, kws)
    }

    /// Qu lists packages which have an update available.
    //? Is that the right way to input '<'?
    async fn qu(&self, _kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(&["apk", "version", "-l", "<"]).flags(flags))
            .await
    }

    /// R removes a single package, leaving all of its dependencies installed.
    async fn r(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::with_sudo(&["apk", "del"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_PROMPT))
            .await
    }

    /// Rn removes a package and skips the generation of configuration backup
    /// files.
    async fn rn(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::with_sudo(&["apk", "del", "--purge"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_PROMPT))
            .await
    }

    /// Rns removes a package and its dependencies which are not required by any
    /// other installed package, and skips the generation of configuration
    /// backup files.
    async fn rns(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::with_sudo(&["apk", "del", "--purge", "-r"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_PROMPT))
            .await
    }

    /// Rs removes a package and its dependencies which are not required by any
    /// other installed package, and not explicitly installed by the user.
    async fn rs(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.r(kws, flags).await
    }

    /// S installs one or more packages by name.
    async fn s(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::with_sudo(&["apk", "add"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_INSTALL))
            .await
    }

    /// Sc removes all the cached packages that are not currently installed, and
    /// the unused sync database.
    async fn sc(&self, _kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::with_sudo(&["apk", "cache", "-v", "clean"])
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_PROMPT))
            .await
    }

    /// Scc removes all files from the cache.
    async fn scc(&self, _kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::with_sudo(&["rm", "-vrf", "/var/cache/apk/*"])
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_PROMPT))
            .await
    }

    /// Si displays remote package information: name, version, description, etc.
    async fn si(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(&["apk", "info", "-a"]).kws(kws).flags(flags))
            .await
    }

    /// Sii displays packages which require X to be installed, aka reverse
    /// dependencies.
    async fn sii(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(&["apk", "info", "-r"]).kws(kws).flags(flags))
            .await
    }

    /// Sl displays a list of all packages in all installation sources that are
    /// handled by the packages management.
    async fn sl(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(&["apk", "search"]).kws(kws).flags(flags))
            .await
    }

    /// Ss searches for package(s) by searching the expression in name,
    /// description, short description.
    async fn ss(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(&["apk", "search", "-v"]).kws(kws).flags(flags))
            .await
    }

    /// Su updates outdated packages.
    async fn su(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::with_sudo(if kws.is_empty() {
            &["apk", "upgrade"]
        } else {
            &["apk", "add", "-u"]
        })
        .kws(kws)
        .flags(flags)
        .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_INSTALL))
        .await
    }

    /// Suy refreshes the local package database, then updates outdated
    /// packages.
    async fn suy(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::with_sudo(if kws.is_empty() {
            &["apk", "upgrade", "-U", "-a"]
        } else {
            &["apk", "add", "-U", "-u"]
        })
        .kws(kws)
        .flags(flags)
        .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_INSTALL))
        .await
    }

    /// Sw retrieves all packages from the server, but does not install/upgrade
    /// anything.
    async fn sw(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(&["apk", "fetch"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_PROMPT))
            .await
    }

    /// Sy refreshes the local package database.
    async fn sy(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::with_sudo(&["apk", "update"]).kws(kws).flags(flags))
            .await?;
        if !kws.is_empty() {
            self.s(kws, flags).await?;
        }
        Ok(())
    }

    /// U upgrades or adds package(s) to the system and installs the required
    /// dependencies from sync repositories.
    async fn u(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::with_sudo(&["apk", "add", "--allow-untrusted"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_INSTALL))
            .await
    }
}
