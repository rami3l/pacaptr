use super::PackManager;
use crate::error::Error;
use crate::exec::{self, Mode};
use regex::Regex;

pub struct Zypper {
    pub dry_run: bool,
    pub no_confirm: bool,
    pub no_cache: bool,
}

impl Zypper {
    /// A helper method to simplify prompted command invocation.
    fn prompt_run(
        &self,
        cmd: &str,
        subcmd: &[&str],
        kws: &[&str],
        flags: &[&str],
    ) -> Result<(), Error> {
        let mut subcmd: Vec<&str> = subcmd.iter().cloned().collect();
        if self.no_confirm {
            subcmd.push("-y");
        }
        self.just_run(cmd, &subcmd, kws, flags)
    }
}

impl PackManager for Zypper {
    /// Get the name of the package manager.
    fn name(&self) -> String {
        "zypper".into()
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
        if kws.is_empty() {
            self.just_run("rpm", &["-qa", "--qf"], &["%{NAME} %{VERSION}\\n"], flags)
        } else {
            self.qs(kws, flags)
        }
    }

    /// Qc shows the changelog of a package.
    fn qc(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("rpm", &["-q", "--changelog"], kws, flags)
    }

    /// Qi displays local package information: name, version, description, etc.
    fn qi(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.si(kws, flags)
    }

    /// Ql displays files provided by local package.
    fn ql(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("rpm", &["-ql"], kws, flags)
    }

    /// Qm lists packages that are installed but are not available in any installation source (anymore).
    fn qm(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        let search = |contents: &str, regex: &Regex| {
            for line in contents.lines() {
                if let Some(_) = regex.find(line) {
                    println!("{}", line);
                }
            }
        };

        let out_bytes = exec::exec("zypper", &["search", "-si"], kws, flags, Mode::Mute)?;
        let out = String::from_utf8(out_bytes)?;

        lazy_static! {
            static ref SYSTEM_PACKAGES: Regex = Regex::new(r"System Packages").unwrap();
        }

        search(&out, &*SYSTEM_PACKAGES);
        Ok(())
    }

    /// Qo queries the package which provides FILE.
    fn qo(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("rpm", &["-qf"], kws, flags)
    }

    /// Qp queries a package supplied on the command line rather than an entry in the package management database.
    fn qp(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("rpm", &["-qip"], kws, flags)
    }

    /// Qs searches locally installed package for names or descriptions.
    // According to https://www.archlinux.org/pacman/pacman.8.html#_query_options_apply_to_em_q_em_a_id_qo_a,
    // when including multiple search terms, only packages with descriptions matching ALL of those terms are returned.
    fn qs(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("zypper", &["search", "--installed-only"], kws, flags)
    }

    /// Qu lists packages which have an update available.
    fn qu(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("zypper", &["list-updates"], kws, flags)
    }

    /// R removes a single package, leaving all of its dependencies installed.
    fn r(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.prompt_run("zypper", &["remove"], kws, flags)
    }

    /// Rss removes a package and its dependencies which are not required by any other installed package.
    fn rss(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.prompt_run("zypper", &["remove", "--clean-deps"], kws, flags)
    }

    /// S installs one or more packages by name.
    fn s(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        let subcmd: &[&str] = if self.dry_run {
            &["install", "--dry-run"]
        } else {
            &["install"]
        };
        exec::exec("zypper", subcmd, kws, flags, Mode::CheckErr)?;
        if self.no_cache {
            self.scc(kws, flags)?;
        }
        Ok(())
    }

    /// Sc removes all the cached packages that are not currently installed, and the unused sync database.
    fn sc(&self, _kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.prompt_run("zypper", &["clean"], &[], flags)
    }

    /// Scc removes all files from the cache.
    fn scc(&self, _kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.sc(_kws, flags)
    }

    /// Si displays remote package information: name, version, description, etc.
    fn si(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("zypper", &["info", "--requires"], kws, flags)
    }

    /// Sl displays a list of all packages in all installation sources that are handled by the packages management.
    fn sl(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        if !kws.is_empty() {
            self.just_run("zypper", &["packages", "-r"], kws, flags)
        } else {
            self.just_run("zypper", &["packages", "-R"], &[], flags)
        }
    }

    /// Ss searches for package(s) by searching the expression in name, description, short description.
    fn ss(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("zypper", &["search"], kws, flags)
    }

    /// Su updates outdated packages.
    fn su(&self, _kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("zypper", &["--no-refresh", "dist-upgrade"], &[], flags)?;
        if self.no_cache {
            self.sccc(_kws, flags)?;
        }
        Ok(())
    }

    /// Suy refreshes the local package database, then updates outdated packages.
    fn suy(&self, _kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.prompt_run("zypper", &["dist-upgrade"], &[], flags)?;
        if self.no_cache {
            self.sccc(_kws, flags)?;
        }
        Ok(())
    }

    /// Sw retrieves all packages from the server, but does not install/upgrade anything.
    fn sw(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.prompt_run("zypper", &["install", "--download-only"], kws, flags)
    }

    /// Sy refreshes the local package database.
    fn sy(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("zypper", &["refresh"], &[], flags)?;
        if !kws.is_empty() {
            self.s(kws, flags)?;
        }
        Ok(())
    }

    /// U upgrades or adds package(s) to the system and installs the required dependencies from sync repositories.
    fn u(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.s(kws, flags)
    }
}
