use async_trait::async_trait;
use once_cell::sync::Lazy;
use tap::prelude::*;

use super::{NoCacheStrategy, Pm, PmHelper, PmMode, PromptStrategy, Strategy};
use crate::{
    dispatch::config::Config,
    error::Result,
    exec::{self, Cmd},
    print::{self, PROMPT_RUN},
};

#[derive(Debug)]
pub struct Scoop {
    pub cfg: Config,
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
    async fn search_regex(&self, cmd: &[&str], kws: &[&str], flags: &[&str]) -> Result<()> {
        let cmd = Cmd::new(cmd).flags(flags);
        if !self.cfg.dry_run {
            print::print_cmd(&cmd, PROMPT_RUN);
        }
        let out_bytes = self
            .check_output(cmd, PmMode::Mute, &Strategy::default())
            .await?
            .contents;

        exec::grep_print(&String::from_utf8(out_bytes)?, kws)?;
        Ok(())
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
            self.run(Cmd::new(&["powershell", "scoop", "list"]).flags(flags))
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
        self.search_regex(&["powershell", "scoop", "list"], kws, flags)
            .await
    }

    /// Qu lists packages which have an update available.
    async fn qu(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(&["powershell", "scoop", "status"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run(cmd))
            .await
    }

    /// R removes a single package, leaving all of its dependencies installed.
    async fn r(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(&["powershell", "scoop", "uninstall"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_PROMPT))
            .await
    }

    /// Rn removes a package and skips the generation of configuration backup
    /// files.
    async fn rn(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(&["powershell", "scoop", "uninstall", "--purge"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_PROMPT))
            .await
    }

    /// S installs one or more packages by name.
    async fn s(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(&["powershell", "scoop", "install"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_INSTALL))
            .await
    }

    /// Sc removes all the cached packages that are not currently installed, and
    /// the unused sync database.
    async fn sc(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(&["powershell", "scoop", "cache", "rm"])
            .kws(if kws.is_empty() { &["*"] } else { kws })
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
        Cmd::new(&["powershell", "scoop", "info"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run(cmd))
            .await
    }

    /// Ss searches for package(s) by searching the expression in name,
    /// description, short description.
    async fn ss(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(&["powershell", "scoop", "search"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run(cmd))
            .await
    }

    /// Sy refreshes the local package database.
    async fn sy(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(&["powershell", "scoop", "update"]).flags(flags))
            .await?;
        if !kws.is_empty() {
            self.s(kws, flags).await?;
        }
        Ok(())
    }

    /// Su updates outdated packages.
    async fn su(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(&["powershell", "scoop", "update"])
            .kws(if kws.is_empty() { &["*"] } else { kws })
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
}
