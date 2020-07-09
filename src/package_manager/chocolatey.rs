use super::PackageManager;
use crate::dispatch::config::Config;
use crate::error::Error;
use crate::exec::{self, Mode};

pub struct Chocolatey {
    pub cfg: Config,
}

impl Chocolatey {
    fn prompt_run(
        &self,
        cmd: &str,
        subcmd: &[&str],
        kws: &[&str],
        flags: &[&str],
    ) -> Result<(), Error> {
        let mut subcmd: Vec<&str> = subcmd.to_vec();
        if self.cfg.no_confirm {
            subcmd.push("--yes");
        }
        self.just_run(cmd, &subcmd, kws, flags)
    }
}

impl PackageManager for Chocolatey {
    /// Get the name of the package manager.
    fn name(&self) -> String {
        "choco".into()
    }

    /// A helper method to simplify direct command invocation.
    fn just_run(
        &self,
        cmd: &str,
        subcmd: &[&str],
        kws: &[&str],
        flags: &[&str],
    ) -> Result<(), Error> {
        let mut flags: Vec<&str> = flags.to_vec();
        if self.cfg.dry_run {
            flags.push("--what-if");
        }
        exec::exec(cmd, subcmd, kws, &flags, Mode::CheckErr)?;
        Ok(())
    }

    /// Q generates a list of installed packages.
    fn q(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("choco", &["list", "--localonly"], kws, flags)
    }

    /// Qi displays local package information: name, version, description, etc.
    fn qi(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.si(kws, flags)
    }

    /// Qu lists packages which have an update available.
    fn qu(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("choco", &["outdated"], kws, flags)
    }

    /// R removes a single package, leaving all of its dependencies installed.
    fn r(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.prompt_run("choco", &["uninstall"], kws, flags)
    }

    /// Rss removes a package and its dependencies which are not required by any other installed package.
    fn rss(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.prompt_run("choco", &["uninstall", "--removedependencies"], kws, flags)
    }

    /// S installs one or more packages by name.
    fn s(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        let subcmd: &[&str] = if self.cfg.needed {
            &["install"]
        } else {
            &["install", "--force"]
        };
        self.prompt_run("choco", subcmd, kws, flags)
    }

    /// Si displays remote package information: name, version, description, etc.
    fn si(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("choco", &["info"], kws, flags)
    }

    /// Ss searches for package(s) by searching the expression in name, description, short description.
    fn ss(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("choco", &["search"], kws, flags)
    }

    /// Su updates outdated packages.
    fn su(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        if kws.is_empty() {
            self.prompt_run("choco", &["upgrade"], &["all"], flags)
        } else {
            self.prompt_run("choco", &["upgrade"], kws, flags)
        }
    }

    /// Suy refreshes the local package database, then updates outdated packages.
    fn suy(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.su(kws, flags)
    }
}
