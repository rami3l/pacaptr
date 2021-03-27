use super::{Pm, PmHelper, PmMode, PromptStrategy, Strategies};
use crate::{
    dispatch::config::Config,
    error::Result,
    exec::{self, Cmd},
    print::{self, PROMPT_RUN},
};
use async_trait::async_trait;
use futures::prelude::*;
use once_cell::sync::Lazy;

pub struct Conda {
    pub cfg: Config,
}

static PROMPT_STRAT: Lazy<Strategies> = Lazy::new(|| Strategies {
    prompt: PromptStrategy::native_prompt(&["-y"]),
    ..Default::default()
});

#[async_trait]
impl Pm for Conda {
    /// Gets the name of the package manager.
    fn name(&self) -> String {
        "conda".into()
    }

    fn cfg(&self) -> &Config {
        &self.cfg
    }

    /// Q generates a list of installed packages.
    async fn q(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        if kws.is_empty() {
            self.just_run_default(Cmd::new(&["conda", "list"]).flags(flags))
                .await
        } else {
            self.qs(kws, flags).await
        }
    }

    /// Qs searches locally installed package for names or descriptions.
    // According to https://www.archlinux.org/pacman/pacman.8.html#_query_options_apply_to_em_q_em_a_id_qo_a,
    // when including multiple search terms, only packages with descriptions matching ALL of those terms are returned.
    async fn qs(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        let cmd = &["conda", "list"];
        let cmd = Cmd::new(cmd).flags(flags);
        if !self.cfg.dry_run {
            print::print_cmd(&cmd, PROMPT_RUN);
        }
        let out_bytes = self
            .run(cmd, PmMode::Mute, &Default::default())
            .await?
            .contents;
        exec::grep_print(&String::from_utf8(out_bytes)?, kws)?;
        Ok(())
    }

    /// R removes a single package, leaving all of its dependencies installed.
    async fn r(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run(
            Cmd::new(&["conda", "remove"]).kws(kws).flags(flags),
            Default::default(),
            &PROMPT_STRAT,
        )
        .await
    }

    /// S installs one or more packages by name.
    async fn s(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run(
            Cmd::new(&["conda", "install"]).kws(kws).flags(flags),
            Default::default(),
            &PROMPT_STRAT,
        )
        .await
    }

    /// Sc removes all the cached packages that are not currently installed, and the unused sync database.
    async fn sc(&self, _kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run(
            Cmd::new(&["conda", "clean", "--all"]).flags(flags),
            Default::default(),
            &PROMPT_STRAT,
        )
        .await
    }

    /// Si displays remote package information: name, version, description, etc.
    async fn si(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run_default(
            Cmd::new(&["conda", "search", "--info"])
                .kws(kws)
                .flags(flags),
        )
        .await
    }

    /// Ss searches for package(s) by searching the expression in name, description, short description.
    async fn ss(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        let kws: Vec<String> = kws.iter().map(|&s| format!("*{}*", s)).collect();

        stream::iter(kws)
            .map(Ok)
            .try_for_each(|kw| async move {
                self.just_run_default(Cmd::new(&["conda", "search"]).kws(&[kw]).flags(flags))
                    .await
            })
            .await
    }

    /// Su updates outdated packages.
    async fn su(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run(
            Cmd::new(&["conda", "update", "--all"])
                .kws(kws)
                .flags(flags),
            Default::default(),
            &PROMPT_STRAT,
        )
        .await
    }

    /// Suy refreshes the local package database, then updates outdated packages.
    async fn suy(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.su(kws, flags).await
    }
}
