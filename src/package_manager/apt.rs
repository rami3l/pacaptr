use super::PackageManager;
use crate::error::Error;
use crate::exec::{self, Mode};

pub struct Apt {
    pub dry_run: bool,
    pub no_confirm: bool,
    pub needed: bool,
    pub no_cache: bool,
}

impl Apt {
    /// A helper method to simplify prompted command invocation.
    fn prompt_run(
        &self,
        cmd: &str,
        subcmd: &[&str],
        kws: &[&str],
        flags: &[&str],
    ) -> Result<(), Error> {
        let mut subcmd: Vec<&str> = subcmd.to_vec();
        if self.no_confirm {
            subcmd.push("--yes");
        }
        self.just_run(cmd, &subcmd, kws, flags)
    }
}

impl PackageManager for Apt {
    /// Get the name of the package manager.
    fn name(&self) -> String {
        "apt".into()
    }

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
        self.just_run("apt", &["list"], kws, flags)
    }

    /// Qi displays local package information: name, version, description, etc.
    fn qi(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("dpkg-query", &["-s"], kws, flags)
    }

    /// Qo queries the package which provides FILE.
    fn qo(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("dpkg-query", &["-S"], kws, flags)
    }

    /// Qp queries a package supplied on the command line rather than an entry in the package management database.
    fn qp(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("dpkg-deb", &["-I"], kws, flags)
    }

    /// Qu lists packages which have an update available.
    fn qu(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("apt", &["upgrade", "--trivial-only"], kws, flags)
    }

    /// R removes a single package, leaving all of its dependencies installed.
    fn r(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.prompt_run("apt", &["remove"], kws, flags)
    }

    /// Rn removes a package and skips the generation of configuration backup files.
    fn rn(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.prompt_run("apt", &["purge"], kws, flags)
    }

    /// Rns removes a package and its dependencies which are not required by any other installed package, and skips the generation of configuration backup files.
    fn rns(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.prompt_run("apt", &["autoremove", "--purge"], kws, flags)
    }

    /// Rs removes a package and its dependencies which are not required by any other installed package,
    /// and not explicitly installed by the user.
    fn rs(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.prompt_run("apt", &["autoremove"], kws, flags)
    }

    /// S installs one or more packages by name.
    fn s(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        let subcmd: &[&str] = if self.needed {
            &["install"]
        } else {
            &["install", "--reinstall"]
        };
        self.prompt_run("apt", subcmd, kws, flags)?;
        if self.no_cache {
            self.scc(kws, flags)?;
        }
        Ok(())
    }

    /// Sc removes all the cached packages that are not currently installed, and the unused sync database.
    fn sc(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.prompt_run("apt", &["clean"], kws, flags)
    }

    /// Scc removes all files from the cache.
    fn scc(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.prompt_run("apt", &["autoclean"], kws, flags)
    }

    /// Si displays remote package information: name, version, description, etc.
    fn si(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("apt", &["show"], kws, flags)
    }

    /// Sii displays packages which require X to be installed, aka reverse dependencies.
    fn sii(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("apt", &["rdepends"], kws, flags)
    }

    /// Ss searches for package(s) by searching the expression in name, description, short description.
    fn ss(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("apt", &["search"], kws, flags)
    }

    /// Su updates outdated packages.
    fn su(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        if kws.is_empty() {
            self.prompt_run("apt", &["upgrade"], &[], flags)?;
            self.prompt_run("apt", &["dist-upgrade"], &[], flags)?;
            if self.no_cache {
                self.scc(kws, flags)?;
            }
            Ok(())
        } else {
            self.s(kws, flags)
        }
    }

    /// Suy refreshes the local package database, then updates outdated packages.
    fn suy(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.sy(kws, flags)?;
        self.su(kws, flags)
    }

    /// Sw retrieves all packages from the server, but does not install/upgrade anything.
    fn sw(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.prompt_run("apt", &["install", "--download-only"], kws, flags)
    }

    /// Sy refreshes the local package database.
    fn sy(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("apt", &["update"], &[], flags)?;
        if !kws.is_empty() {
            self.s(kws, flags)?;
        }
        Ok(())
    }
}
