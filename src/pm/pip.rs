use async_trait::async_trait;
use once_cell::sync::Lazy;
use tap::prelude::*;

use super::{Pm, PmHelper, PmMode, PromptStrategy, Strategy};
use crate::{
    dispatch::config::Config,
    error::Result,
    exec::{self, Cmd},
    print::{self, PROMPT_RUN},
};

pub struct Pip {
    pub cmd: String,
    pub cfg: Config,
}

static STRAT_PROMPT: Lazy<Strategy> = Lazy::new(|| Strategy {
    prompt: PromptStrategy::CustomPrompt,
    ..Default::default()
});

static STRAT_UNINSTALL: Lazy<Strategy> = Lazy::new(|| Strategy {
    prompt: PromptStrategy::native_no_confirm(&["-y"]),
    ..Default::default()
});

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
            self.run(Cmd::new(&[self.cmd.as_ref(), "list"]).flags(flags))
                .await
        } else {
            self.qs(kws, flags).await
        }
    }

    /// Qi displays local package information: name, version, description, etc.
    async fn qi(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(&[self.cmd.as_ref(), "show"]).kws(kws).flags(flags))
            .await
    }

    /// Qs searches locally installed package for names or descriptions.
    // According to https://www.archlinux.org/pacman/pacman.8.html#_query_options_apply_to_em_q_em_a_id_qo_a,
    // when including multiple search terms, only packages with descriptions matching ALL of those terms are returned.
    async fn qs(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        let cmd = Cmd::new(&[self.cmd.as_ref(), "list"]).flags(flags);
        if !self.cfg.dry_run {
            print::print_cmd(&cmd, PROMPT_RUN);
        }
        let out_bytes = self
            .check_output(cmd, PmMode::Mute, &Default::default())
            .await?
            .contents;
        exec::grep_print(&String::from_utf8(out_bytes)?, kws)?;
        Ok(())
    }

    /// Qu lists packages which have an update available.
    async fn qu(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(&[self.cmd.as_ref(), "list", "--outdated"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run(cmd))
            .await
    }

    /// R removes a single package, leaving all of its dependencies installed.
    async fn r(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(&[self.cmd.as_ref(), "uninstall"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, Default::default(), &STRAT_UNINSTALL))
            .await
    }

    /// S installs one or more packages by name.
    async fn s(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(&[self.cmd.as_ref(), "install"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, Default::default(), &STRAT_PROMPT))
            .await
    }

    /// Sc removes all the cached packages that are not currently installed, and the unused sync database.
    async fn sc(&self, _kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(&[self.cmd.as_ref(), "cache", "purge"]).flags(flags))
            .await
    }

    /// Su updates outdated packages.
    async fn su(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        if !kws.is_empty() {
            Cmd::new(&[self.cmd.as_ref(), "install", "--upgrade"])
                .kws(kws)
                .flags(flags)
                .pipe(|cmd| self.run(cmd))
                .await
        } else {
            Err(crate::error::Error::OperationUnimplementedError {
                op: "su".into(),
                pm: self.name().into(),
            })
        }
    }

    /// Sw retrieves all packages from the server, but does not install/upgrade anything.
    async fn sw(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(&[self.cmd.as_ref(), "download"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run(cmd))
            .await
    }
}
