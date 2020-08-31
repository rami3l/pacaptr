use super::PackageManager;
use crate::dispatch::config::Config;
use crate::error::Error;
use crate::exec::{self, Mode};

pub struct Tlmgr {
    pub cfg: Config,
}

impl Tlmgr {
    /// A helper method to simplify prompted command invocation.
    fn check_dry_run(
        &self,
        cmd: &str,
        subcmd: &[&str],
        kws: &[&str],
        flags: &[&str],
    ) -> Result<(), Error> {
        let mut subcmd: Vec<&str> = subcmd.to_vec();
        if self.cfg.dry_run {
            subcmd.push("--dry-run");
        }
        exec::exec(cmd, &subcmd, kws, flags, Mode::CheckErr)?;
        Ok(())
    }
}

impl PackageManager for Tlmgr {
    /// Get the name of the package manager.
    fn name(&self) -> String {
        "tlmgr".into()
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
            Mode::PrintCmd
        } else {
            Mode::CheckErr
        };
        exec::exec(cmd, subcmd, kws, flags, mode)?;
        Ok(())
    }

    /// Q generates a list of installed packages.
    fn q(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.qi(kws, flags)
    }

    /// Qi displays local package information: name, version, description, etc.
    fn qi(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("tlmgr", &["info", "--only-installed"], kws, flags)
    }

    /// Qk verifies one or more packages.
    fn qk(&self, _kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("tlmgr", &["check", "files"], &[], flags)
    }

    /// Ql displays files provided by local package.
    fn ql(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("tlmgr", &["info", "--only-installed", "--list"], kws, flags)
    }

    /// R removes a single package, leaving all of its dependencies installed.
    fn r(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.check_dry_run("tlmgr", &["remove"], kws, flags)
    }

    /// S installs one or more packages by name.
    fn s(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.check_dry_run("tlmgr", &["install"], kws, flags)
    }

    /// Si displays remote package information: name, version, description, etc.
    fn si(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("tlmgr", &["info"], kws, flags)
    }

    /// Sl displays a list of all packages in all installation sources that are handled by the packages management.
    fn sl(&self, _kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("tlmgr", &["info"], &[], flags)
    }

    /// Ss searches for package(s) by searching the expression in name, description, short description.
    fn ss(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("tlmgr", &["search", "--global"], kws, flags)
    }

    /// Su updates outdated packages.
    fn su(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        if kws.is_empty() {
            self.check_dry_run("tlmgr", &["update", "--self", "--all"], &[], flags)
        } else {
            self.check_dry_run("tlmgr", &["update", "--self"], kws, flags)
        }
    }

    /// Suy refreshes the local package database, then updates outdated packages.
    fn suy(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.su(kws, flags)
    }

    /// U upgrades or adds package(s) to the system and installs the required dependencies from sync repositories.
    fn u(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.check_dry_run("tlmgr", &["install", "--file"], kws, flags)
    }
}
