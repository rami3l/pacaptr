use super::{DryRunStrategy, NoCacheStrategy, PackageManager, PmMode, PromptStrategy, Strategies};
use crate::dispatch::config::Config;
use crate::exec::{self, Cmd, Mode};
use crate::print::{self, PROMPT_INFO, PROMPT_RUN};
use anyhow::Result;
use futures::stream::{self, TryStreamExt};

pub struct Homebrew {
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

enum CaskState {
    NotFound,
    Brew,
    Cask,
}

impl Homebrew {
    /// Search the output of `brew info` to see if we need `brew cask` for a certain package.
    async fn search(&self, pack: &str, flags: &[&str]) -> Result<CaskState> {
        let out_bytes = Cmd::new(&["brew", "info"])
            .kws(&[pack])
            .flags(flags)
            .exec(Mode::Mute)
            .await?
            .contents;
        let out = String::from_utf8(out_bytes)?;

        let code = {
            let no_formula = "No available formula with the name";
            let found_cask = "Found a cask named";

            if exec::grep(&out, &[no_formula]).is_empty() {
                // Found a formula
                CaskState::Brew
            } else if !exec::grep(&out, &[found_cask]).is_empty() {
                // Found a cask
                CaskState::Cask
            } else {
                CaskState::NotFound
            }
        };

        Ok(code)
    }

    /// The common logic behind functions like S and R to use `brew cask` commands automatically.
    /// With the exception of `self.cfg.force_cask`,
    /// this function will use `self.search()` to see if we need `brew cask` for a certain package,
    /// and then try to execute the corresponding command.
    async fn auto_cask_do(
        &self,
        subcmd: &'_ [&str],
        pack: &str,
        flags: &[&str],
        strat: Strategies,
    ) -> Result<()> {
        async fn run(
            self_: &Homebrew,
            mut cmd: Vec<&str>,
            subcmd: &[&str],
            pack: &str,
            flags: &[&str],
            strat: Strategies,
        ) -> Result<()> {
            cmd.extend(subcmd);
            self_
                .just_run(
                    Cmd::new(&cmd).kws(&[pack]).flags(flags),
                    Default::default(),
                    strat,
                )
                .await
        }

        if self.cfg.force_cask {
            return run(&self, vec!["brew", "cask"], subcmd, pack, flags, strat).await;
        }

        let code = self.search(pack, flags).await?;
        match code {
            CaskState::NotFound | CaskState::Brew => {
                run(&self, vec!["brew"], subcmd, pack, flags, strat).await
            }
            CaskState::Cask => run(&self, vec!["brew", "cask"], subcmd, pack, flags, strat).await,
        }
    }
}

#[async_trait]
impl PackageManager for Homebrew {
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
        async fn run(self_: &Homebrew, cmd: &[&str], kws: &[&str], flags: &[&str]) -> Result<()> {
            let search = |contents: &str| {
                exec::grep(contents, kws)
                    .iter()
                    .for_each(|ln| println!("{}", ln))
            };

            let cmd = Cmd::new(cmd).flags(flags);
            if !self_.cfg.dry_run {
                print::print_cmd(&cmd, PROMPT_RUN);
            }
            let out_bytes = self_
                .run(cmd, PmMode::Mute, Default::default())
                .await?
                .contents;

            search(&String::from_utf8(out_bytes)?);
            Ok(())
        };

        // ! `brew list` lists all formulae and casks only when using tty.
        run(self, &["brew", "list"], kws, flags).await?;
        run(self, &["brew", "list", "--cask"], kws, flags).await?;

        Ok(())
    }

    /// Qu lists packages which have an update available.
    async fn qu(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run_default(Cmd::new(&["brew", "outdated"]).kws(kws).flags(flags))
            .await
    }

    /// R removes a single package, leaving all of its dependencies installed.
    async fn r(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        stream::iter(kws.iter().map(Ok))
            .try_for_each(|&pack| async move {
                self.auto_cask_do(&["uninstall"], pack, flags, PROMPT_STRAT.clone())
                    .await
            })
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
        for &pack in kws {
            if self.cfg.needed {
                self.auto_cask_do(&["install"], pack, flags, INSTALL_STRAT.clone())
                    .await?;
            } else {
                // If the package is not installed, `brew reinstall` behaves just like `brew install`,
                // so `brew reinstall` matches perfectly the behavior of `pacman -S`.
                self.auto_cask_do(&["reinstall"], pack, flags, INSTALL_STRAT.clone())
                    .await?;
            }
        }
        Ok(())
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
        let cmd: &[&str] = if self.cfg.force_cask {
            &["brew", "cask", "info"]
        } else {
            &["brew", "info"]
        };
        self.just_run_default(Cmd::new(cmd).kws(kws).flags(flags))
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
        if kws.is_empty() {
            // `brew cask upgrade` is now deprecated.
            // A simple `brew upgrade` should do the job.
            self.just_run(
                Cmd::new(&["brew", "upgrade"]).kws(kws).flags(flags),
                Default::default(),
                PROMPT_STRAT.clone(),
            )
            .await
        } else {
            for &pack in kws {
                self.auto_cask_do(&["upgrade"], pack, flags, PROMPT_STRAT.clone())
                    .await?;
            }
            if self.cfg.no_cache {
                self.scc(&[], flags).await?;
            }
            Ok(())
        }
    }

    /// Suy refreshes the local package database, then updates outdated packages.
    async fn suy(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.sy(&[], flags).await?;
        self.su(kws, flags).await
    }

    /// Sw retrieves all packages from the server, but does not install/upgrade anything.
    async fn sw(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        stream::iter(kws.iter().map(Ok))
            .try_for_each(|&pack| async move {
                self.auto_cask_do(&["fetch"], pack, flags, PROMPT_STRAT.clone())
                    .await
            })
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
