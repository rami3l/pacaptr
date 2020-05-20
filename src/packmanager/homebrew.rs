use super::PackManager;
use crate::error::Error;
use crate::exec::{self, print_msg, Mode, PROMPT_INFO, PROMPT_RUN};
use regex::Regex;

pub struct Homebrew {
    pub dry_run: bool,
    pub force_cask: bool,
    pub no_confirm: bool,
    pub needed: bool,
}

enum CaskState {
    NotFound,
    Unneeded,
    Needed,
}

impl Homebrew {
    /// Search the output of `brew info` to see if we need `brew cask` for a certain package.
    fn search(&self, pack: &str, flags: &[&str]) -> Result<CaskState, Error> {
        let out_bytes = exec::exec("brew", &["info"], &[pack], flags, Mode::Mute)?;
        let out = String::from_utf8(out_bytes)?;

        let code = {
            lazy_static! {
                static ref RE_BOTTLE: Regex =
                    Regex::new(r"No available formula with the name").unwrap();
                static ref RE_CASK: Regex = Regex::new(r"Found a cask named").unwrap();
            }

            if RE_BOTTLE.find(&out).is_some() {
                if RE_CASK.find(&out).is_some() {
                    CaskState::Needed
                } else {
                    CaskState::NotFound
                }
            } else {
                CaskState::Unneeded
            }
        };

        Ok(code)
    }

    /// The common logic behind functions like S and R to use `brew cask` commands automatically.
    /// With the exception of `self.force_cask`,
    /// this function will use `self.search()` to see if we need `brew cask` for a certain package,
    /// and then try to execute the corresponding command.
    fn auto_cask_do(&self, subcmd: &[&str], pack: &str, flags: &[&str]) -> Result<(), Error> {
        let brew_do = || self.prompt_run("brew", subcmd, &[pack], flags);
        let brew_cask_do = || {
            let subcmd: Vec<&str> = ["cask"].iter().chain(subcmd).map(|&s| s).collect();
            self.prompt_run("brew", &subcmd, &[pack], flags)
        };

        if self.force_cask {
            return brew_cask_do();
        }

        let code = self.search(pack, flags)?;
        match code {
            CaskState::NotFound | CaskState::Unneeded => brew_do(),
            CaskState::Needed => brew_cask_do(),
        }
    }

    /// A helper method to simplify prompted command invocation.
    fn prompt_run(
        &self,
        cmd: &str,
        subcmd: &[&str],
        kws: &[&str],
        flags: &[&str],
    ) -> Result<(), Error> {
        let mode = if self.dry_run {
            Mode::DryRun
        } else if self.no_confirm {
            Mode::CheckErr
        } else {
            Mode::Prompt
        };
        exec::exec(cmd, subcmd, kws, flags, mode)?;
        Ok(())
    }
}

impl PackManager for Homebrew {
    /// A helper method to simplify direct command invocation.
    fn just_run(
        &self,
        cmd: &str,
        subcmd: &[&str],
        kws: &[&str],
        flags: &[&str],
    ) -> Result<(), Error> {
        let mode = if self.dry_run {
            Mode::DryRun
        } else {
            Mode::CheckErr
        };
        exec::exec(cmd, subcmd, kws, flags, mode)?;
        Ok(())
    }

    /// Q generates a list of installed packages.
    fn q(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        if kws.is_empty() {
            self.just_run("brew", &["list"], &[], flags)?;
            self.just_run("brew", &["cask", "list"], &[], flags)
        } else {
            self.qs(kws, flags)
        }
    }

    /// Qc shows the changelog of a package.
    fn qc(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("brew", &["log"], kws, flags)
    }

    /// Qi displays local package information: name, version, description, etc.
    fn qi(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.si(kws, flags)
    }

    /// Ql displays files provided by local package.
    fn ql(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        // TODO: it seems that the output of `brew list python` in fish has a mechanism against duplication:
        // /usr/local/Cellar/python/3.6.0/Frameworks/Python.framework/ (1234 files)
        self.just_run("brew", &["list"], kws, flags)
    }

    /// Qs searches locally installed package for names or descriptions.
    // According to https://www.archlinux.org/pacman/pacman.8.html#_query_options_apply_to_em_q_em_a_id_qo_a,
    // when including multiple search terms, only packages with descriptions matching ALL of those terms are returned.
    fn qs(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        let search = |contents: String| {
            let rs: Vec<Regex> = kws.iter().map(|kw| Regex::new(kw).unwrap()).collect();
            for line in contents.lines() {
                let matches_all = rs.iter().all(|regex| regex.find(line).is_some());

                if matches_all {
                    println!("{}", line);
                }
            }
        };

        let search_output = |cmd, subcmd| {
            exec::print_cmd(cmd, subcmd, &[], flags, PROMPT_RUN);
            let out_bytes = exec::exec(cmd, subcmd, &[], flags, Mode::Mute)?;
            search(String::from_utf8(out_bytes)?);
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
        for &pack in kws {
            self.auto_cask_do(&["uninstall"], pack, flags)?;
        }
        Ok(())
    }

    /// Rs removes a package and its dependencies which are not required by any other installed package.
    fn rs(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        let subcmd: &[&str] = if self.dry_run {
            &["rmtree", "--dry-run"]
        } else {
            &["rmtree"]
        };
        let err_bytes = exec::exec("brew", subcmd, kws, flags, Mode::CheckErr)?;
        let err_msg = String::from_utf8(err_bytes)?;

        lazy_static! {
            static ref RMTREE_MISSING: Regex = Regex::new(r"Unknown command: rmtree").unwrap();
        }

        if RMTREE_MISSING.find(&err_msg).is_some() {
            print_msg(
                "`rmtree` is not installed. You may try installing it with the following command:",
                PROMPT_INFO,
            );
            print_msg("brew tap beeftornado/rmtree", PROMPT_INFO);
            return Err("`rmtree` required".into());
        }

        Ok(())
    }

    /// S installs one or more packages by name.
    fn s(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        lazy_static! {
            static ref NO_SUCH_KEG: Regex = Regex::new(r"Error: No such keg").unwrap();
            static ref CASK_NOT_INSTALLED: Regex = Regex::new(r"is not installed.").unwrap();
        }

        let is_installed = |pack: &str| -> Result<bool, Error> {
            let mut contents = exec::exec("brew", &["list"], &[pack], flags, Mode::Mute)?;
            contents
                .extend(exec::exec("brew", &["cask", "list"], &[pack], flags, Mode::Mute)?.iter());
            let contents = String::from_utf8(contents)?;
            Ok(NO_SUCH_KEG.find(&contents).is_none()
                || CASK_NOT_INSTALLED.find(&contents).is_none())
        };

        for &pack in kws {
            let is_installed = is_installed(pack)?;
            match () {
                _ if is_installed && self.needed => print_msg(
                    &format!("Skipping installation of installed package `{}`", pack),
                    PROMPT_INFO,
                ),
                _ if is_installed => self.auto_cask_do(&["reinstall"], pack, flags)?,
                _ => self.auto_cask_do(&["install"], pack, flags)?,
            }
        }

        Ok(())
    }

    /// Sc removes all the cached packages that are not currently installed, and the unused sync database.
    fn sc(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        if self.dry_run {
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
        if self.dry_run {
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
        self.just_run("brew", &["info"], kws, flags)
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
            self.prompt_run("brew", &["cask", "upgrade"], kws, flags)
        } else {
            for &pack in kws {
                self.auto_cask_do(&["upgrade"], pack, flags)?;
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
    fn sy(&self, _kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("brew", &["update"], &[], flags)
    }
}
