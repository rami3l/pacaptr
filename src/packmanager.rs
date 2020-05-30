pub mod apk;
pub mod chocolatey;
pub mod dpkg;
pub mod homebrew;
pub mod unknown;

use crate::error::Error;
use crate::exec::{self, Mode};

macro_rules! make_pm {
    ($( $(#[$meta:meta])* $method:ident ), *) => {
        $($(#[$meta])*
        fn $method(&self, _kws: &[&str], _flags: &[&str]) -> std::result::Result<(), crate::error::Error> {
            std::result::Result::Err(format!("Operation `{}` unimplemented for `{}`", stringify!($method), self.name()).into())
        })*
    };
}

/// The behaviors of a Pack(age)Manager.
/// For method explanation see: https://wiki.archlinux.org/index.php/Pacman/Rosetta
/// and https://wiki.archlinux.org/index.php/Pacman
pub trait PackManager {
    /// Get the name of the package manager.
    fn name(&self) -> String;

    /// A helper method to simplify direct command invocation.
    /// Override this to implement features such as `dryrun`.
    fn just_run(
        &self,
        cmd: &str,
        subcmd: &[&str],
        kws: &[&str],
        flags: &[&str],
    ) -> Result<(), Error> {
        exec::exec(cmd, subcmd, kws, flags, Mode::CheckErr)?;
        Ok(())
    }

    make_pm!(
        /// Q generates a list of installed packages.
        q,
        /// Qc shows the changelog of a package.
        qc,
        /// Qe lists packages installed explicitly (not as dependencies).
        qe,
        /// Qi displays local package information: name, version, description, etc.
        qi,
        /// Qk verifies one or more packages.
        qk,
        /// Ql displays files provided by local package.
        ql,
        /// Qm lists packages that are installed but are not available in any installation source (anymore).
        qm,
        /// Qo queries the package which provides FILE.
        qo,
        /// Qp queries a package supplied on the command line rather than an entry in the package management database.
        qp,
        /// Qs searches locally installed package for names or descriptions.
        qs,
        /// Qu lists packages which have an update available.
        qu,
        /// R removes a single package, leaving all of its dependencies installed.
        r,
        /// Rn removes a package and skips the generation of configuration backup files.
        rn,
        /// Rns removes a package and its dependencies which are not required by any other installed package,
        /// and skips the generation of configuration backup files.
        rns,
        /// Rs removes a package and its dependencies which are not required by any other installed package.
        rs,
        /// S installs one or more packages by name.
        s,
        /// Sc removes all the cached packages that are not currently installed, and the unused sync database.
        sc,
        /// Scc removes all files from the cache.
        scc,
        /// Sccc ...
        /// What is this?
        sccc,
        /// Sg lists all packages belonging to the GROUP.
        sg,
        /// Si displays remote package information: name, version, description, etc.
        si,
        /// Sii displays packages which require X to be installed, aka reverse dependencies.
        sii,
        /// Sl displays a list of all packages in all installation sources that are handled by the packages management.
        sl,
        /// Ss searches for package(s) by searching the expression in name, description, short description.
        ss,
        /// Su updates outdated packages.
        su,
        /// Suy refreshes the local package database, then updates outdated packages.
        suy,
        /// Sw retrieves all packages from the server, but does not install/upgrade anything.
        sw,
        /// Sy refreshes the local package database.
        sy,
        /// U upgrades or adds package(s) to the system and installs the required dependencies from sync repositories.
        u
    );
}
