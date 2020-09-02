use super::{PackageManager, PmMode, PromptStrategy, Strategies};
use crate::dispatch::config::Config;
use crate::error::Error;
use crate::exec::{self, Cmd, Mode};
use crate::print::{self, PROMPT_INFO, PROMPT_RUN};

pub struct Homebrew {
    pub cfg: Config,
}

enum CaskState {
    NotFound,
    Unneeded,
    Needed,
}

impl Homebrew {
    const PROMPT_STRAT: Strategies = Strategies {
        prompt: PromptStrategy::CustomPrompt,
        ..Default::default()
    };

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
                CaskState::Unneeded
            } else {
                // Found no formula
                if !exec::grep(&out, &[found_cask]).is_empty() {
                    // Found a cask
                    CaskState::Needed
                } else {
                    CaskState::NotFound
                }
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
    ) -> Result<(), Error> {
        let do_impl = |mut cmd: Vec<&'s str>| {
            cmd.extend(subcmd);
            self.just_run(
                Cmd::new(&cmd).kws(&[pack]).flags(flags),
                Default::default(),
                Self::PROMPT_STRAT,
            )
        };
        let brew_do = || do_impl(vec!["brew"]);
        let brew_cask_do = || do_impl(vec!["brew", "cask"]);

        if self.cfg.force_cask {
            return brew_cask_do();
        }

        let code = self.search(pack, flags)?;
        match code {
            CaskState::NotFound | CaskState::Unneeded => brew_do(),
            CaskState::Needed => brew_cask_do(),
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
            self.just_run_default(Cmd::new(&["brew", "list"]).flags(flags))?;
            self.just_run_default(Cmd::new(&["brew", "cask", "list"]).flags(flags))
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
        self.just_run("brew", &["list"], kws, flags)?;
        self.just_run("brew", &["cask", "list"], kws, flags)
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

        let search_output = |cmd, subcmd| {
            print::print_cmd(cmd, subcmd, &[], flags, PROMPT_RUN);
            let out_bytes = exec::exec(cmd, subcmd, &[], flags, Mode::Mute)?;
            search(&String::from_utf8(out_bytes)?);
            Ok(())
        };

        search_output("brew", &["list"])?;
        search_output("brew", &["cask", "list"])
    }

    /// Qu lists packages which have an update available.
    fn qu(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("brew", &["outdated"], kws, flags)
    }

    /// R removes a single package, leaving all of its dependencies installed.
    fn r(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        kws.iter()
            .map(|&pack| self.auto_cask_do(&["uninstall"], pack, flags))
            .collect()
    }

    /// Rss removes a package and its dependencies which are not required by any other installed package.
    fn rss(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        let subcmd: &[&str] = if self.cfg.dry_run {
            &["rmtree", "--dry-run"]
        } else {
            &["rmtree"]
        };
        let err_bytes = exec::exec("brew", subcmd, kws, flags, Mode::CheckErr)?;
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
                self.auto_cask_do(&["install"], pack, flags)?;
            } else {
                // If the package is not installed, `brew reinstall` behaves just like `brew install`,
                // so `brew reinstall` matches perfectly the behavior of `pacman -S`.
                self.auto_cask_do(&["reinstall"], pack, flags)?;
            }
        }
        if self.cfg.no_cache {
            self.scc(kws, flags)?;
        }
        Ok(())
    }

    /// Sc removes all the cached packages that are not currently installed, and the unused sync database.
    fn sc(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        if self.cfg.dry_run {
            exec::exec(
                "brew",
                &["cleanup", "--dry-run"],
                kws,
                flags,
                Mode::CheckErr,
            )?;
            Ok(())
        } else {
            self.prompt_run("brew", &["cleanup"], kws, flags)
        }
    }

    /// Scc removes all files from the cache.
    fn scc(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        if self.cfg.dry_run {
            exec::exec(
                "brew",
                &["cleanup", "-s", "--dry-run"],
                kws,
                flags,
                Mode::CheckErr,
            )?;
            Ok(())
        } else {
            self.prompt_run("brew", &["cleanup", "-s"], kws, flags)
        }
    }

    /// Si displays remote package information: name, version, description, etc.
    fn si(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        let subcmd: &[&str] = if self.cfg.force_cask {
            &["cask", "info"]
        } else {
            &["info"]
        };
        self.just_run("brew", subcmd, kws, flags)
    }

    /// Sii displays packages which require X to be installed, aka reverse dependencies.
    fn sii(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("brew", &["uses"], kws, flags)
    }

    /// Ss searches for package(s) by searching the expression in name, description, short description.
    fn ss(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("brew", &["search"], kws, flags)
    }

    /// Su updates outdated packages.
    fn su(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        if kws.is_empty() {
            self.prompt_run("brew", &["upgrade"], kws, flags)?;
            self.prompt_run("brew", &["cask", "upgrade"], kws, flags)?;
            if self.cfg.no_cache {
                self.scc(kws, flags)?;
            }
            Ok(())
        } else {
            for &pack in kws {
                self.auto_cask_do(&["upgrade"], pack, flags)?;
            }
            if self.cfg.no_cache {
                self.scc(kws, flags)?;
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
        self.prompt_run("brew", &["fetch"], kws, flags)
    }

    /// Sy refreshes the local package database.
    fn sy(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("brew", &["update"], &[], flags)?;
        if !kws.is_empty() {
            self.s(kws, flags)?;
        }
        Ok(())
    }
}
