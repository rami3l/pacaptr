use super::{DryRunStrategy, NoCacheStrategy, PackageManager, PmMode, PromptStrategy, Strategies};
use crate::dispatch::config::Config;
use crate::error::Error;
use crate::exec::{self, Cmd, Mode};
use crate::print::{self, PROMPT_INFO, PROMPT_RUN};

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
    /*
    const CASK_PREFIX: &'static str = "cask/";

    fn strip_cask_prefix(pack: &str) -> String {
        {
            if pack.starts_with(Self::CASK_PREFIX) {
                &pack[Self::CASK_PREFIX.len()..]
            } else {
                pack
            }
        }
        .to_owned()
    }
    */

    /// Search the output of `brew info` to see if we need `brew cask` for a certain package.
    fn search(&self, pack: &str, flags: &[&str]) -> Result<CaskState, Error> {
        let out_bytes = Cmd::new(&["brew", "info"])
            .kws(&[pack])
            .flags(flags)
            .exec(Mode::Mute)?;
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
    fn auto_cask_do<'s>(
        &self,
        subcmd: &'s [&str],
        pack: &str,
        flags: &[&str],
        strat: Strategies,
    ) -> Result<(), Error> {
        let run = |mut cmd: Vec<&'s str>, pack: &str| {
            cmd.extend(subcmd);
            self.just_run(
                Cmd::new(&cmd).kws(&[pack]).flags(flags),
                Default::default(),
                strat,
            )
        };

        if self.cfg.force_cask {
            return run(vec!["brew", "cask"], pack);
        }

        let code = self.search(pack, flags)?;
        match code {
            CaskState::NotFound | CaskState::Brew => run(vec!["brew"], pack),
            CaskState::Cask => run(vec!["brew", "cask"], pack),
        }
    }
}

impl PackageManager for Homebrew {
    /// Get the name of the package manager.
    fn name(&self) -> String {
        "brew".into()
    }

    fn cfg(&self) -> Config {
        self.cfg.clone()
    }

    /// Q generates a list of installed packages.
    fn q(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        if kws.is_empty() {
            self.just_run_default(Cmd::new(&["brew", "list"]).flags(flags))
        } else {
            self.qs(kws, flags)
        }
    }

    /// Qc shows the changelog of a package.
    fn qc(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run_default(Cmd::new(&["brew", "log"]).kws(kws).flags(flags))
    }

    /// Qi displays local package information: name, version, description, etc.
    fn qi(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.si(kws, flags)
    }

    /// Ql displays files provided by local package.
    fn ql(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        // TODO: it seems that the output of `brew list python` in fish has a mechanism against duplication:
        // /usr/local/Cellar/python/3.6.0/Frameworks/Python.framework/ (1234 files)
        self.just_run_default(Cmd::new(&["brew", "list"]).kws(kws).flags(flags))
    }

    /// Qs searches locally installed package for names or descriptions.
    // According to https://www.archlinux.org/pacman/pacman.8.html#_query_options_apply_to_em_q_em_a_id_qo_a,
    // when including multiple search terms, only packages with descriptions matching ALL of those terms are returned.
    fn qs(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        let search = |contents: &str| {
            exec::grep(contents, kws)
                .iter()
                .for_each(|ln| println!("{}", ln))
        };

        let search_output = |cmd| {
            let cmd = Cmd::new(cmd).flags(flags);
            if !self.cfg.dry_run {
                print::print_cmd(&cmd, PROMPT_RUN);
            }
            let out_bytes = self.run(cmd, PmMode::Mute, Default::default())?;
            search(&String::from_utf8(out_bytes)?);
            Ok(())
        };

        search_output(&["brew", "list"])
    }

    /// Qu lists packages which have an update available.
    fn qu(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run_default(Cmd::new(&["brew", "outdated"]).kws(kws).flags(flags))
    }

    /// R removes a single package, leaving all of its dependencies installed.
    fn r(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        kws.iter()
            .map(|&pack| self.auto_cask_do(&["uninstall"], pack, flags, PROMPT_STRAT.clone()))
            .collect()
    }

    /// Rss removes a package and its dependencies which are not required by any other installed package.
    fn rss(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        let err_bytes = self.run(
            Cmd::new(&["brew", "rmtree"]).kws(kws).flags(flags),
            Default::default(),
            Strategies {
                dry_run: DryRunStrategy::with_flags(&["--dry-run"]),
                ..Default::default()
            },
        )?;
        let err_msg = String::from_utf8(err_bytes)?;

        let pattern = "Unknown command: rmtree";
        if !exec::grep(&err_msg, &[pattern]).is_empty() {
            print::print_msg(
                "`rmtree` is not installed. You may install it with the following command:",
                PROMPT_INFO,
            );
            print::print_msg("`brew tap beeftornado/rmtree`", PROMPT_INFO);
            return Err("`rmtree` required".into());
        }

        Ok(())
    }

    /// S installs one or more packages by name.
    fn s(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        for &pack in kws {
            if self.cfg.needed {
                self.auto_cask_do(&["install"], pack, flags, INSTALL_STRAT.clone())?;
            } else {
                // If the package is not installed, `brew reinstall` behaves just like `brew install`,
                // so `brew reinstall` matches perfectly the behavior of `pacman -S`.
                self.auto_cask_do(&["reinstall"], pack, flags, INSTALL_STRAT.clone())?;
            }
        }
        Ok(())
    }

    /// Sc removes all the cached packages that are not currently installed, and the unused sync database.
    fn sc(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run(
            Cmd::new(&["brew", "cleanup"]).kws(kws).flags(flags),
            Default::default(),
            Strategies {
                dry_run: DryRunStrategy::with_flags(&["--dry-run"]),
                prompt: PromptStrategy::CustomPrompt,
                ..Default::default()
            },
        )
    }

    /// Scc removes all files from the cache.
    fn scc(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run(
            Cmd::new(&["brew", "cleanup", "-s"]).kws(kws).flags(flags),
            Default::default(),
            Strategies {
                dry_run: DryRunStrategy::with_flags(&["--dry-run"]),
                prompt: PromptStrategy::CustomPrompt,
                ..Default::default()
            },
        )
    }

    /// Si displays remote package information: name, version, description, etc.
    fn si(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        let cmd: &[&str] = if self.cfg.force_cask {
            &["brew", "cask", "info"]
        } else {
            &["brew", "info"]
        };
        self.just_run_default(Cmd::new(cmd).kws(kws).flags(flags))
    }

    /// Sii displays packages which require X to be installed, aka reverse dependencies.
    fn sii(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run_default(Cmd::new(&["brew", "uses"]).kws(kws).flags(flags))
    }

    /// Ss searches for package(s) by searching the expression in name, description, short description.
    fn ss(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run_default(Cmd::new(&["brew", "search"]).kws(kws).flags(flags))
    }

    /// Su updates outdated packages.
    fn su(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        if kws.is_empty() {
            // `brew cask upgrade` is now deprecated.
            // A simple `brew upgrade` should do the job.
            self.just_run(
                Cmd::new(&["brew", "upgrade"]).kws(kws).flags(flags),
                Default::default(),
                PROMPT_STRAT.clone(),
            )
        } else {
            for &pack in kws {
                self.auto_cask_do(&["upgrade"], pack, flags, PROMPT_STRAT.clone())?;
            }
            if self.cfg.no_cache {
                self.scc(&[], flags)?;
            }
            Ok(())
        }
    }

    /// Suy refreshes the local package database, then updates outdated packages.
    fn suy(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.sy(&[], flags)?;
        self.su(kws, flags)
    }

    /// Sw retrieves all packages from the server, but does not install/upgrade anything.
    fn sw(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        kws.iter()
            .map(|&pack| self.auto_cask_do(&["fetch"], pack, flags, PROMPT_STRAT.clone()))
            .collect()
    }

    /// Sy refreshes the local package database.
    fn sy(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run_default(Cmd::new(&["brew", "update"]).flags(flags))?;
        if !kws.is_empty() {
            self.s(kws, flags)?;
        }
        Ok(())
    }
}
