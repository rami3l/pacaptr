use super::{PackageManager, PromptStrategy, Strategies};
use crate::dispatch::config::Config;
use crate::error::{Error, Result};
use crate::exec::{self, Cmd};
use crate::package_manager::{NoCacheStrategy, PmMode};
use crate::print::{self, PROMPT_RUN};
use async_trait::async_trait;
use lazy_static::lazy_static;

pub struct Scoop {
    pub cfg: Config,
}

lazy_static! {
    static ref PROMPT_STRAT: Strategies = Strategies {
        prompt: PromptStrategy::CustomPrompt,
        ..Default::default()
    };
    static ref INSTALL_STRAT: Strategies = Strategies {
        prompt: PromptStrategy::CustomPrompt,
        no_cache: NoCacheStrategy::Scc,
        ..Default::default()
    };
}

// Windows is so special! It's better not to "sudo" automatically.
#[async_trait]
impl PackageManager for Scoop {
    /// Get the name of the package manager.
    fn name(&self) -> String {
        "scoop".into()
    }

    fn cfg(&self) -> Config {
        self.cfg.clone()
    }

    /// Q generates a list of installed packages.
    async fn q(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        if kws.is_empty() {
            self.just_run_default(Cmd::new(&["powershell", "scoop", "list"]).flags(flags))
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
    // when including multiple search terms, only packages with descriptions matching ALL of those terms are returned.
    async fn qs(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        let search = |contents: &str| {
            exec::grep(contents, kws)
                .iter()
                .for_each(|ln| println!("{}", ln))
        };

        macro_rules! run {
            ( $cmd: expr ) => {
                async {
                    let cmd = Cmd::new($cmd).flags(flags);
                    if !self.cfg.dry_run {
                        print::print_cmd(&cmd, PROMPT_RUN);
                    }
                    let out_bytes = self
                        .run(cmd, PmMode::Mute, &Default::default())
                        .await?
                        .contents;

                    search(&String::from_utf8(out_bytes)?);
                    Ok::<(), Error>(())
                }
            };
        }

        run!(&["powershell", "scoop", "list"]).await?;
        Ok(())
    }

    /// Qu lists packages which have an update available.
    async fn qu(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run_default(
            Cmd::new(&["powershell", "scoop", "status"])
                .kws(kws)
                .flags(flags),
        )
        .await
    }

    /// R removes a single package, leaving all of its dependencies installed.
    async fn r(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run(
            Cmd::new(&["powershell", "scoop", "uninstall"])
                .kws(kws)
                .flags(flags),
            Default::default(),
            &PROMPT_STRAT,
        )
        .await
    }

    /// Rn removes a package and skips the generation of configuration backup files.
    async fn rn(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run(
            Cmd::new(&["powershell", "scoop", "uninstall", "--purge"])
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
            Cmd::new(&["powershell", "scoop", "install"])
                .kws(kws)
                .flags(flags),
            Default::default(),
            &INSTALL_STRAT,
        )
        .await
    }

    /// Sc removes all the cached packages that are not currently installed, and the unused sync database.
    async fn sc(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        let kws: &[&str] = if kws.is_empty() { &["*"] } else { kws };
        self.just_run(
            Cmd::new(&["powershell", "scoop", "cache", "rm"])
                .kws(kws)
                .flags(flags),
            Default::default(),
            &PROMPT_STRAT,
        )
        .await
    }

    /// Scc removes all files from the cache.
    async fn scc(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.sc(kws, flags).await
    }

    /// Si displays remote package information: name, version, description, etc.
    async fn si(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run_default(
            Cmd::new(&["powershell", "scoop", "info"])
                .kws(kws)
                .flags(flags),
        )
        .await
    }

    /// Ss searches for package(s) by searching the expression in name, description, short description.
    async fn ss(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run_default(
            Cmd::new(&["powershell", "scoop", "search"])
                .kws(kws)
                .flags(flags),
        )
        .await
    }

    /// Sy refreshes the local package database.
    async fn sy(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run_default(Cmd::new(&["powershell", "scoop", "update"]).flags(flags))
            .await?;
        if !kws.is_empty() {
            self.s(kws, flags).await?;
        }
        Ok(())
    }

    /// Su updates outdated packages.
    async fn su(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        let kws: &[&str] = if kws.is_empty() { &["*"] } else { kws };
        self.just_run(
            Cmd::new(&["powershell", "scoop", "update"])
                .kws(kws)
                .flags(flags),
            Default::default(),
            &INSTALL_STRAT,
        )
        .await
    }

    /// Suy refreshes the local package database, then updates outdated packages.
    async fn suy(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.sy(&[], flags).await?;
        self.su(kws, flags).await
    }
}
