use super::{PackageManager, PmMode, PromptStrategy, Strategies};
use crate::dispatch::config::Config;
use crate::error::Result;
use crate::exec::{self, Cmd};
use crate::print::{self, PROMPT_RUN};
use async_trait::async_trait;
use lazy_static::lazy_static;

pub struct Pip {
    pub cmd: String,
    pub cfg: Config,
}

lazy_static! {
    static ref PROMPT_STRAT: Strategies = Strategies {
        prompt: PromptStrategy::native_prompt(&["-y"]),
        ..Default::default()
    };
}

#[async_trait]
impl PackageManager for Pip {
    /// Get the name of the package manager.
    fn name(&self) -> String {
        "pip".into()
    }

    fn cfg(&self) -> Config {
        self.cfg.clone()
    }

    /// Q generates a list of installed packages.
    async fn q(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        if kws.is_empty() {
            self.just_run_default(Cmd::new(&[self.cmd.as_ref(), "list"]).flags(flags))
                .await
        } else {
            self.qs(kws, flags).await
        }
    }

    /// Qs searches locally installed package for names or descriptions.
    // According to https://www.archlinux.org/pacman/pacman.8.html#_query_options_apply_to_em_q_em_a_id_qo_a,
    // when including multiple search terms, only packages with descriptions matching ALL of those terms are returned.
    async fn qs(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        let search = |contents: &str| {
            exec::grep(contents, kws)
                .iter()
                .for_each(|ln| println!("{}", ln))
        };

        let cmd = &[self.cmd.as_ref(), "list"];
        let cmd = Cmd::new(cmd).flags(flags);
        if !self.cfg.dry_run {
            print::print_cmd(&cmd, PROMPT_RUN);
        }
        let out_bytes = self
            .run(cmd, PmMode::Mute, &Default::default())
            .await?
            .contents;
        search(&String::from_utf8(out_bytes)?);
        Ok(())
    }

    /// Qu lists packages which have an update available.
    async fn qu(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run_default(
            Cmd::new(&[self.cmd.as_ref(), "list", "--outdated"])
                .kws(kws)
                .flags(flags),
        )
        .await
    }

    /// R removes a single package, leaving all of its dependencies installed.
    async fn r(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run(
            Cmd::new(&[self.cmd.as_ref(), "uninstall"])
                .kws(kws)
                .flags(flags),
            Default::default(),
            &PROMPT_STRAT,
        )
        .await
    }

    /// S installs one or more packages by name.
    async fn s(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run(
            Cmd::new(&[self.cmd.as_ref(), "install"])
                .kws(kws)
                .flags(flags),
            Default::default(),
            &PROMPT_STRAT,
        )
        .await
    }

    /// Sc removes all the cached packages that are not currently installed, and the unused sync database.
    async fn sc(&self, _kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run_default(Cmd::new(&[self.cmd.as_ref(), "cache", "purge"]).flags(flags))
            .await
    }

    /// Si displays remote package information: name, version, description, etc.
    async fn si(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run_default(Cmd::new(&[self.cmd.as_ref(), "show"]).kws(kws).flags(flags))
            .await
    }

    /// Ss searches for package(s) by searching the expression in name, description, short description.
    async fn ss(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run_default(
            Cmd::new(&[self.cmd.as_ref(), "search"])
                .kws(kws)
                .flags(flags),
        )
        .await
    }

    /// Su updates outdated packages.
    async fn su(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        if !kws.is_empty() {
            self.just_run_default(
                Cmd::new(&[self.cmd.as_ref(), "install", "--upgrade"])
                    .kws(kws)
                    .flags(flags),
            )
            .await
        } else {
            Err(crate::error::Error::OperationUnimplementedError {
                op: "su".into(),
                pm: self.name(),
            })
        }
    }

    /// Sw retrieves all packages from the server, but does not install/upgrade anything.
    async fn sw(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run_default(
            Cmd::new(&[self.cmd.as_ref(), "download"])
                .kws(kws)
                .flags(flags),
        )
        .await
    }
}
