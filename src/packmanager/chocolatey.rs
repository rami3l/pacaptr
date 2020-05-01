use super::PackManager;
use crate::error::Error;
use crate::exec::{self, Mode};

pub struct Chocolatey {
    pub dry_run: bool,
    pub no_confirm: bool,
}

impl Chocolatey {
    fn check_no_confirm(&self, cmd: &str, subcmd: &[&str], kws: &[&str]) -> Result<(), Error> {
        let mut subcmd: Vec<&str> = subcmd.iter().cloned().collect();
        if self.no_confirm {
            subcmd.push("--yes");
        }
        self.just_run(cmd, &subcmd, kws)
    }
}

impl PackManager for Chocolatey {
    /// A helper method to simplify direct command invocation.
    fn just_run(&self, cmd: &str, subcmd: &[&str], kws: &[&str]) -> Result<(), Error> {
        let mode = if self.dry_run {
            Mode::DryRun
        } else {
            Mode::CheckErr
        };
        exec::exec(cmd, subcmd, kws, mode)?;
        Ok(())
    }

    /// Q generates a list of installed packages.
    fn q(&self, kws: &[&str]) -> Result<(), Error> {
        self.just_run("choco", &["list", "--localonly"], kws)
    }

    /// Qi displays local package information: name, version, description, etc.
    fn qi(&self, kws: &[&str]) -> Result<(), Error> {
        self.si(kws)
    }

    /// Qu lists packages which have an update available.
    fn qu(&self, kws: &[&str]) -> Result<(), Error> {
        self.just_run("choco", &["outdated"], kws)
    }

    /// R removes a single package, leaving all of its dependencies installed.
    fn r(&self, kws: &[&str]) -> Result<(), Error> {
        self.check_no_confirm("choco", &["uninstall"], kws)
    }

    /// Rs removes a package and its dependencies which are not required by any other installed package.
    fn rs(&self, kws: &[&str]) -> Result<(), Error> {
        self.check_no_confirm("choco", &["uninstall", "--removedependencies"], kws)
    }

    /// S installs one or more packages by name.
    fn s(&self, kws: &[&str]) -> Result<(), Error> {
        self.check_no_confirm("choco", &["install"], kws)
    }

    /// Si displays remote package information: name, version, description, etc.
    fn si(&self, kws: &[&str]) -> Result<(), Error> {
        self.just_run("choco", &["info"], kws)
    }

    /// Ss searches for package(s) by searching the expression in name, description, short description.
    fn ss(&self, kws: &[&str]) -> Result<(), Error> {
        self.just_run("choco", &["search"], kws)
    }

    /// Su updates outdated packages.
    fn su(&self, kws: &[&str]) -> Result<(), Error> {
        if kws.is_empty() {
            self.check_no_confirm("choco", &["upgrade"], &["all"])
        } else {
            self.check_no_confirm("choco", &["upgrade"], kws)
        }
    }

    /// Suy refreshes the local package database, then updates outdated packages.
    fn suy(&self, kws: &[&str]) -> Result<(), Error> {
        self.su(kws)
    }
}
