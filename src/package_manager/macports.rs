use super::PackageManager;
use crate::dispatch::config::Config;
use crate::error::Error;
use crate::exec::{self, Mode};

pub struct Macports {
    pub cfg: Config,
}

impl Macports {
    /// A helper method to simplify prompted command invocation.
    fn prompt_run(
        &self,
        cmd: &str,
        subcmd: &[&str],
        kws: &[&str],
        flags: &[&str],
    ) -> Result<(), Error> {
        let mode = match () {
            _ if self.cfg.dry_run => Mode::DryRun,
            _ if self.cfg.no_confirm => Mode::CheckErr,
            _ => Mode::Prompt,
        };
        exec::exec(cmd, subcmd, kws, flags, mode)?;
        Ok(())
    }
}

impl PackageManager for Macports {
    /// Get the name of the package manager.
    fn name(&self) -> String {
        "port".into()
    }

    /// A helper method to simplify direct command invocation.
    fn just_run(
        &self,
        cmd: &str,
        subcmd: &[&str],
        kws: &[&str],
        flags: &[&str],
    ) -> Result<(), Error> {
        let mode = if self.cfg.dry_run {
            Mode::DryRun
        } else {
            Mode::CheckErr
        };
        exec::exec(cmd, subcmd, kws, flags, mode)?;
        Ok(())
    }

    /// Q generates a list of installed packages.
    fn q(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("port", &["installed"], kws, flags)
    }

    /// Qc shows the changelog of a package.
    fn qc(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("port", &["log"], kws, flags)
    }

    /// Qi displays local package information: name, version, description, etc.
    fn qi(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.si(kws, flags)
    }

    /// Ql displays files provided by local package.
    fn ql(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("port", &["contents"], kws, flags)
    }

    /// Qo queries the package which provides FILE.
    fn qo(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("port", &["provides"], kws, flags)
    }

    /// Qs searches locally installed package for names or descriptions.
    // According to https://www.archlinux.org/pacman/pacman.8.html#_query_options_apply_to_em_q_em_a_id_qo_a,
    // when including multiple search terms, only packages with descriptions matching ALL of those terms are returned.
    fn qs(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("port", &["-v", "installed"], kws, flags)
    }

    /// Qu lists packages which have an update available.
    fn qu(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("port", &["outdated"], kws, flags)
    }

    /// R removes a single package, leaving all of its dependencies installed.
    fn r(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.prompt_run("port", &["uninstall"], kws, flags)
    }

    /// Rss removes a package and its dependencies which are not required by any other installed package.
    fn rss(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.prompt_run("port", &["uninstall", "--follow-dependencies"], kws, flags)
    }

    /// S installs one or more packages by name.
    fn s(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("port", &["install"], kws, flags)?;
        if self.cfg.no_cache {
            self.scc(kws, flags)?;
        }
        Ok(())
    }

    /// Sc removes all the cached packages that are not currently installed, and the unused sync database.
    fn sc(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        if flags.is_empty() {
            self.prompt_run("port", &["clean", "--all"], &["inactive"], flags)
        } else {
            self.prompt_run("port", &["clean", "--all"], kws, flags)
        }
    }

    /// Scc removes all files from the cache.
    fn scc(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        if flags.is_empty() {
            self.prompt_run("port", &["clean", "--all"], &["installed"], flags)
        } else {
            self.sc(kws, flags)
        }
    }

    /// Si displays remote package information: name, version, description, etc.
    fn si(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("port", &["info"], kws, flags)
    }

    /// Ss searches for package(s) by searching the expression in name, description, short description.
    fn ss(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("port", &["search"], kws, flags)
    }

    /// Su updates outdated packages.
    fn su(&self, _kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.prompt_run("port", &["upgrade"], &["outdated"], flags)?;
        if self.cfg.no_cache {
            self.scc(_kws, flags)?;
        }
        Ok(())
    }

    /// Suy refreshes the local package database, then updates outdated packages.
    fn suy(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.sy(&[], flags)?;
        self.su(kws, flags)
    }

    /// Sy refreshes the local package database.
    fn sy(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("port", &["selfupdate"], &[], flags)?;
        if !kws.is_empty() {
            self.s(kws, flags)?;
        }
        Ok(())
    }
}
