use super::PackManager;
use crate::error::Error;
use crate::exec::{self, Mode};

pub struct Dpkg {
    pub dry_run: bool,
    pub no_confirm: bool,
}

impl Dpkg {
    fn check_no_confirm(&self, cmd: &str, subcmd: &[&str], kws: &[&str]) -> Result<(), Error> {
        let mut subcmd: Vec<&str> = subcmd.iter().cloned().collect();
        if self.no_confirm {
            subcmd.push("--yes");
        }
        self.just_run(cmd, &subcmd, kws)
    }
}

impl PackManager for Dpkg {
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
        self.just_run("dpkg", &["-l"], kws)
    }

    /// Qi displays local package information: name, version, description, etc.
    fn qi(&self, kws: &[&str]) -> Result<(), Error> {
        self.just_run("dpkg-query", &["-s"], kws)
    }

    /// Qo queries the package which provides FILE.
    fn qo(&self, kws: &[&str]) -> Result<(), Error> {
        self.just_run("dpkg-query", &["-S"], kws)
    }

    /// Qp queries a package supplied on the command line rather than an entry in the package management database.
    fn qp(&self, kws: &[&str]) -> Result<(), Error> {
        self.just_run("dpkg-deb", &["-I"], kws)
    }

    /// Qu lists packages which have an update available.
    fn qu(&self, kws: &[&str]) -> Result<(), Error> {
        self.just_run("apt-get", &["upgrade", "--trivial-only"], kws)
    }

    /// R removes a single package, leaving all of its dependencies installed.
    fn r(&self, kws: &[&str]) -> Result<(), Error> {
        self.check_no_confirm("apt-get", &["remove"], kws)
    }

    /// Rn removes a package and skips the generation of configuration backup files.
    fn rn(&self, kws: &[&str]) -> Result<(), Error> {
        self.check_no_confirm("apt-get", &["purge"], kws)
    }

    /// Rns removes a package and its dependencies which are not required by any other installed package, and skips the generation of configuration backup files.
    fn rns(&self, kws: &[&str]) -> Result<(), Error> {
        self.check_no_confirm("apt-get", &["autoremove", "--purge"], kws)
    }

    /// Rs removes a package and its dependencies which are not required by any other installed package.
    fn rs(&self, kws: &[&str]) -> Result<(), Error> {
        self.check_no_confirm("apt-get", &["autoremove"], kws)
    }

    /// S installs one or more packages by name.
    fn s(&self, kws: &[&str]) -> Result<(), Error> {
        self.check_no_confirm("apt-get", &["install"], kws)
    }

    /// Sc removes all the cached packages that are not currently installed, and the unused sync database.
    fn sc(&self, kws: &[&str]) -> Result<(), Error> {
        self.check_no_confirm("apt-get", &["clean"], kws)
    }

    /// Scc removes all files from the cache.
    fn scc(&self, kws: &[&str]) -> Result<(), Error> {
        self.check_no_confirm("apt-get", &["autoclean"], kws)
    }

    /// Si displays remote package information: name, version, description, etc.
    fn si(&self, kws: &[&str]) -> Result<(), Error> {
        self.just_run("apt-cache", &["show"], kws)
    }

    /// Sii displays packages which require X to be installed, aka reverse dependencies.
    fn sii(&self, kws: &[&str]) -> Result<(), Error> {
        self.just_run("apt-cache", &["rdepends"], kws)
    }

    /// Ss searches for package(s) by searching the expression in name, description, short description.
    fn ss(&self, kws: &[&str]) -> Result<(), Error> {
        self.just_run("apt-cache", &["search"], kws)
    }

    /// Su updates outdated packages.
    fn su(&self, kws: &[&str]) -> Result<(), Error> {
        if kws.is_empty() {
            self.check_no_confirm("apt-get", &["upgrade"], &[])?;
            self.check_no_confirm("apt-get", &["dist-upgrade"], &[])
        } else {
            self.s(kws)
        }
    }

    /// Suy refreshes the local package database, then updates outdated packages.
    fn suy(&self, kws: &[&str]) -> Result<(), Error> {
        self.sy(kws)?;
        self.su(kws)
    }

    /// Sw retrieves all packages from the server, but does not install/upgrade anything.
    fn sw(&self, kws: &[&str]) -> Result<(), Error> {
        self.check_no_confirm("apt-get", &["install", "--download-only"], kws)
    }

    /// Sy refreshes the local package database.
    fn sy(&self, _kws: &[&str]) -> Result<(), Error> {
        self.just_run("apt-get", &["update"], &[])
    }
}
