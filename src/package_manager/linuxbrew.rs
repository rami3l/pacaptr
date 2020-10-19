use super::{DryRunStrategy, NoCacheStrategy, PackageManager, PmMode, PromptStrategy, Strategies};
use crate::dispatch::config::Config;
use crate::exec::{self, Cmd};
use crate::print::{self, PROMPT_INFO, PROMPT_RUN};
use anyhow::Result;

pub struct Linuxbrew {
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

#[async_trait]
impl PackageManager for Linuxbrew {
    /// Get the name of the package manager.
    fn name(&self) -> String {
        "brew".into()
    }

    fn cfg(&self) -> Config {
        self.cfg.clone()
    }

    /// Q generates a list of installed packages.
    async fn q(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        if kws.is_empty() {
            self.just_run_default(Cmd::new(&["brew", "list"]).flags(flags))
                .await
        } else {
            self.qs(kws, flags).await
        }
    }

    /// Qc shows the changelog of a package.
    async fn qc(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run_default(Cmd::new(&["brew", "log"]).kws(kws).flags(flags))
            .await
    }

    /// Qi displays local package information: name, version, description, etc.
    async fn qi(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.si(kws, flags).await
    }

    /// Ql displays files provided by local package.
    async fn ql(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        // TODO: it seems that the output of `brew list python` in fish has a mechanism against duplication:
        // /usr/local/Cellar/python/3.6.0/Frameworks/Python.framework/ (1234 files)
        self.just_run_default(Cmd::new(&["brew", "list"]).kws(kws).flags(flags))
            .await
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

        let cmd = &["brew", "list"];
        let cmd = Cmd::new(cmd).flags(flags);
        if !self.cfg.dry_run {
            print::print_cmd(&cmd, PROMPT_RUN);
        }
        let out_bytes = self
            .run(cmd, PmMode::Mute, Default::default())
            .await?
            .contents;
        search(&String::from_utf8(out_bytes)?);
        Ok(())
    }

    /// Qu lists packages which have an update available.
    async fn qu(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run_default(Cmd::new(&["brew", "outdated"]).kws(kws).flags(flags))
            .await
    }

    /// R removes a single package, leaving all of its dependencies installed.
    async fn r(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run(
            Cmd::new(&["brew", "uninstall"]).kws(kws).flags(flags),
            Default::default(),
            PROMPT_STRAT.clone(),
        )
        .await
    }

    /// Rss removes a package and its dependencies which are not required by any other installed package.
    async fn rss(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        let err_bytes = self
            .run(
                Cmd::new(&["brew", "rmtree"]).kws(kws).flags(flags),
                Default::default(),
                Strategies {
                    dry_run: DryRunStrategy::with_flags(&["--dry-run"]),
                    ..Default::default()
                },
            )
            .await?
            .contents;
        let err_msg = String::from_utf8(err_bytes)?;

        let pattern = "Unknown command: rmtree";
        if !exec::grep(&err_msg, &[pattern]).is_empty() {
            print::print_msg(
                "`rmtree` is not installed. You may install it with the following command:",
                PROMPT_INFO,
            );
            print::print_msg("`brew tap beeftornado/rmtree`", PROMPT_INFO);
            return Err(anyhow!("`rmtree` required"));
        }

        Ok(())
    }

    /// S installs one or more packages by name.
    async fn s(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        let cmd = if self.cfg.needed {
            &["brew", "install"]
        } else {
            // If the package is not installed, `brew reinstall` behaves just like `brew install`,
            // so `brew reinstall` matches perfectly the behavior of `pacman -S`.
            &["brew", "reinstall"]
        };
        self.just_run(
            Cmd::new(cmd).kws(kws).flags(flags),
            Default::default(),
            INSTALL_STRAT.clone(),
        )
        .await
    }

    /// Sc removes all the cached packages that are not currently installed, and the unused sync database.
    async fn sc(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run(
            Cmd::new(&["brew", "cleanup"]).kws(kws).flags(flags),
            Default::default(),
            Strategies {
                dry_run: DryRunStrategy::with_flags(&["--dry-run"]),
                prompt: PromptStrategy::CustomPrompt,
                ..Default::default()
            },
        )
        .await
    }

    /// Scc removes all files from the cache.
    async fn scc(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run(
            Cmd::new(&["brew", "cleanup", "-s"]).kws(kws).flags(flags),
            Default::default(),
            Strategies {
                dry_run: DryRunStrategy::with_flags(&["--dry-run"]),
                prompt: PromptStrategy::CustomPrompt,
                ..Default::default()
            },
        )
        .await
    }

    /// Si displays remote package information: name, version, description, etc.
    async fn si(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run_default(Cmd::new(&["brew", "info"]).kws(kws).flags(flags))
            .await
    }

    /// Sii displays packages which require X to be installed, aka reverse dependencies.
    async fn sii(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run_default(Cmd::new(&["brew", "uses"]).kws(kws).flags(flags))
            .await
    }

    /// Ss searches for package(s) by searching the expression in name, description, short description.
    async fn ss(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run_default(Cmd::new(&["brew", "search"]).kws(kws).flags(flags))
            .await
    }

    /// Su updates outdated packages.
    async fn su(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run(
            Cmd::new(&["brew", "upgrade"]).kws(kws).flags(flags),
            Default::default(),
            INSTALL_STRAT.clone(),
        )
        .await
    }

    /// Suy refreshes the local package database, then updates outdated packages.
    async fn suy(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.sy(&[], flags).await?;
        self.su(kws, flags).await
    }

    /// Sw retrieves all packages from the server, but does not install/upgrade anything.
    async fn sw(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run(
            Cmd::new(&["brew", "fetch"]).kws(kws).flags(flags),
            Default::default(),
            PROMPT_STRAT.clone(),
        )
        .await
    }

    /// Sy refreshes the local package database.
    async fn sy(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run_default(Cmd::new(&["brew", "update"]).flags(flags))
            .await?;
        if !kws.is_empty() {
            self.s(kws, flags).await?;
        }
        Ok(())
    }
}
