use super::PackManager;
use crate::error::Error;
use crate::exec::{self, print_msg, Mode, PROMPT_INFO, PROMPT_RUN};
use regex::Regex;

pub struct Homebrew {
    pub dry_run: bool,
    pub force_cask: bool,
}

enum CaskState {
    NotFound,
    Unneeded,
    Needed,
}

impl Homebrew {
    fn search(&self, pack: &str) -> Result<CaskState, Error> {
        let out_bytes = exec::exec("brew", &["info"], &[pack], Mode::Mute)?;
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
}

impl PackManager for Homebrew {
    /// A helper method to simplify direction command invocation.
    fn just_run(&self, cmd: &str, subcmd: &[&str], kws: &[&str]) -> Result<(), Error> {
        let mode = if self.dry_run {
            Mode::DryRun
        } else {
            Mode::CheckErr
        };
        exec::exec(cmd, subcmd, kws, mode)?;
        Ok(())
    }

    /// Qc shows the changelog of a package.
    fn qc(&self, kws: &[&str]) -> Result<(), Error> {
        self.just_run("brew", &["log"], kws)
    }

    /// Qi displays local package information: name, version, description, etc.
    fn qi(&self, kws: &[&str]) -> Result<(), Error> {
        self.si(kws)
    }

    /// Ql displays files provided by local package.
    fn ql(&self, kws: &[&str]) -> Result<(), Error> {
        // TODO: it seems that the output of `brew list python` in fish has a mechanism against duplication:
        // /usr/local/Cellar/python/3.6.0/Frameworks/Python.framework/ (1234 files)
        self.just_run("brew", &["list"], kws)
    }

    /// Qs searches locally installed package for names or descriptions.
    // According to https://www.archlinux.org/pacman/pacman.8.html#_query_options_apply_to_em_q_em_a_id_qo_a,
    // when including multiple search terms, only packages with descriptions matching ALL of those terms are returned.
    fn qs(&self, kws: &[&str]) -> Result<(), Error> {
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
            exec::print_cmd(cmd, subcmd, &[], PROMPT_RUN);
            let out_bytes = exec::exec(cmd, subcmd, &[], Mode::Mute)?;
            search(String::from_utf8(out_bytes)?);
            Ok(())
        };

        search_output("brew", &["list"])?;
        search_output("brew", &["cask", "list"])
    }

    /// Qu lists packages which have an update available.
    fn qu(&self, kws: &[&str]) -> Result<(), Error> {
        self.just_run("brew", &["outdated"], kws)
    }

    /// R removes a single package, leaving all of its dependencies installed.
    fn r(&self, kws: &[&str]) -> Result<(), Error> {
        let uninstall = |pack: &str| -> Result<(), Error> {
            let brew_uninstall = || self.just_run("brew", &["uninstall"], &[pack]);
            let brew_cask_uninstall = || self.just_run("brew", &["cask", "uninstall"], &[pack]);

            if self.force_cask {
                return brew_cask_uninstall();
            }

            let code = self.search(pack)?;
            match code {
                CaskState::NotFound | CaskState::Unneeded => brew_uninstall(),
                CaskState::Needed => brew_cask_uninstall(),
            }
        };

        for &pack in kws {
            uninstall(pack)?;
        }

        Ok(())
    }

    /// Rn removes a package and skips the generation of configuration backup files.
    fn rn(&self, kws: &[&str]) -> Result<(), Error> {
        todo!()
    }

    /// Rns removes a package and its dependencies which are not required by any other installed package, and skips the generation of configuration backup files.
    fn rns(&self, kws: &[&str]) -> Result<(), Error> {
        todo!()
    }

    /// Rs removes a package and its dependencies which are not required by any other installed package.
    fn rs(&self, kws: &[&str]) -> Result<(), Error> {
        let subcmd: &[&str] = if self.dry_run {
            &["rmtree", "--dry-run"]
        } else {
            &["rmtree"]
        };
        let err_bytes = exec::exec("brew", subcmd, kws, Mode::CheckErr)?;
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
    fn s(&self, kws: &[&str]) -> Result<(), Error> {
        let install = |pack: &str| -> Result<(), Error> {
            let brew_install = || self.just_run("brew", &["install"], &[pack]);
            let brew_cask_install = || self.just_run("brew", &["cask", "install"], &[pack]);

            if self.force_cask {
                return brew_cask_install();
            }

            let code = self.search(pack)?;
            match code {
                CaskState::NotFound | CaskState::Unneeded => brew_install(),
                CaskState::Needed => brew_cask_install(),
            }
        };

        for &pack in kws {
            install(pack)?;
        }

        Ok(())
    }

    /// Sc removes all the cached packages that are not currently installed, and the unused sync database.
    fn sc(&self, kws: &[&str]) -> Result<(), Error> {
        if self.dry_run {
            exec::exec("brew", &["cleanup", "--dry-run"], kws, Mode::CheckErr)?;
            Ok(())
        } else {
            self.just_run("brew", &["cleanup"], kws)
        }
    }

    /// Scc removes all files from the cache.
    fn scc(&self, kws: &[&str]) -> Result<(), Error> {
        if self.dry_run {
            exec::exec("brew", &["cleanup", "-s", "--dry-run"], kws, Mode::CheckErr)?;
            Ok(())
        } else {
            self.just_run("brew", &["cleanup", "-s"], kws)
        }
    }

    /// Sccc ...
    // ! What is this?
    fn sccc(&self, kws: &[&str]) -> Result<(), Error> {
        todo!()
    }

    /// Si displays remote package information: name, version, description, etc.
    fn si(&self, kws: &[&str]) -> Result<(), Error> {
        self.just_run("brew", &["info"], kws)
    }

    /// Sii displays packages which require X to be installed, aka reverse dependencies.
    fn sii(&self, kws: &[&str]) -> Result<(), Error> {
        self.just_run("brew", &["uses"], kws)
    }

    /// Ss searches for package(s) by searching the expression in name, description, short description.
    fn ss(&self, kws: &[&str]) -> Result<(), Error> {
        self.just_run("brew", &["search"], kws)
    }

    /// Su updates outdated packages.
    // TODO: What if `pacman -Su curl`?
    fn su(&self, kws: &[&str]) -> Result<(), Error> {
        if kws.is_empty() {
            self.just_run("brew", &["upgrade"], kws)?;
            self.just_run("brew", &["cask", "upgrade"], kws)
        } else {
            let upgrade = |pack: &str| -> Result<(), Error> {
                let brew_upgrade = || self.just_run("brew", &["upgrade"], &[pack]);
                let brew_cask_upgrade = || self.just_run("brew", &["cask", "upgrade"], &[pack]);

                if self.force_cask {
                    return brew_cask_upgrade();
                }

                let code = self.search(pack)?;
                match code {
                    CaskState::NotFound | CaskState::Unneeded => brew_upgrade(),
                    CaskState::Needed => brew_cask_upgrade(),
                }
            };

            for &pack in kws {
                upgrade(pack)?;
            }

            Ok(())
        }
    }

    /// Suy refreshes the local package database, then updates outdated packages.
    fn suy(&self, kws: &[&str]) -> Result<(), Error> {
        self.sy(&[])?;
        self.su(kws)
    }

    /// Sw retrieves all packages from the server, but does not install/upgrade anything.
    fn sw(&self, kws: &[&str]) -> Result<(), Error> {
        self.just_run("brew", &["fetch"], kws)
    }

    /// Sy refreshes the local package database.
    fn sy(&self, _kws: &[&str]) -> Result<(), Error> {
        self.just_run("brew", &["update"], &[])
    }
}
