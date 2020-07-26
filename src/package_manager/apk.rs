use super::PackageManager;
use crate::dispatch::config::Config;
use crate::error::Error;
use crate::exec::{self, Mode};
use crate::print::{self, PROMPT_RUN};

pub struct Apk {
    pub cfg: Config,
}

impl Apk {
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

impl PackageManager for Apk {
    /// Get the name of the package manager.
    fn name(&self) -> String {
        "apk".into()
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
        if kws.is_empty() {
            self.just_run("apk", &["info"], &[], flags)
        } else {
            self.qs(kws, flags)
        }
    }

    /// Qi displays local package information: name, version, description, etc.
    fn qi(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.si(kws, flags)
    }

    /// Ql displays files provided by local package.
    fn ql(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("apk", &["info", "-L"], kws, flags)
    }

    /// Qo queries the package which provides FILE.
    fn qo(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("apk", &["info", "--who-owns"], kws, flags)
    }

    /// Qs searches locally installed package for names or descriptions.
    // According to https://www.archlinux.org/pacman/pacman.8.html#_query_options_apply_to_em_q_em_a_id_qo_a,
    // when including multiple search terms, only packages with descriptions matching ALL of those terms are returned.
    fn qs(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        let search = |contents: &str| {
            exec::grep(contents, kws)
                .iter()
                .for_each(|ln| println!("{}", ln))
        };

        let search_output = |cmd, subcmd| {
            print::print_cmd(cmd, subcmd, &[], flags, PROMPT_RUN);
            let out_bytes = exec::exec(cmd, subcmd, &[], flags, Mode::Mute)?;
            search(&String::from_utf8(out_bytes)?);
            Ok(())
        };

        search_output("apk", &["info", "-d"])
    }

    /// Qu lists packages which have an update available.
    //? Is that the right way to input '<'?
    fn qu(&self, _kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("apk", &["version", "-l"], &["<"], flags)
    }

    /// R removes a single package, leaving all of its dependencies installed.
    fn r(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("apk", &["del"], kws, flags)
    }

    /// Rn removes a package and skips the generation of configuration backup files.
    fn rn(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("apk", &["del", "--purge"], kws, flags)
    }

    /// Rns removes a package and its dependencies which are not required by any other installed package, and skips the generation of configuration backup files.
    fn rns(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("apk", &["del", "--purge", "-r"], kws, flags)
    }

    /// Rs removes a package and its dependencies which are not required by any other installed package,
    /// and not explicitly installed by the user.
    fn rs(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.r(kws, flags)
    }

    /// S installs one or more packages by name.
    fn s(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        let mut flags: Vec<&str> = flags.to_vec();
        if self.cfg.no_cache {
            flags.push("--no-cache");
        }
        self.prompt_run("apk", &["add"], kws, &flags)
    }

    /// Sc removes all the cached packages that are not currently installed, and the unused sync database.
    fn sc(&self, _kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.prompt_run("apk", &["cache", "-v", "clean"], &[], flags)
    }

    /// Scc removes all files from the cache.
    fn scc(&self, _kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.prompt_run("rm", &["-vrf", "/var/cache/apk/*"], &[], flags)
    }

    /// Si displays remote package information: name, version, description, etc.
    fn si(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("apk", &["info", "-a"], kws, flags)
    }

    /// Sii displays packages which require X to be installed, aka reverse dependencies.
    fn sii(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("apk", &["info", "-r"], kws, flags)
    }

    /// Sl displays a list of all packages in all installation sources that are handled by the packages management.
    fn sl(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("apk", &["search"], kws, flags)
    }

    /// Ss searches for package(s) by searching the expression in name, description, short description.
    fn ss(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("apk", &["search", "-v"], kws, flags)
    }

    /// Su updates outdated packages.
    fn su(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        let mut flags: Vec<&str> = flags.to_vec();
        if self.cfg.no_cache {
            flags.push("--no-cache");
        }
        if kws.is_empty() {
            self.prompt_run("apk", &["upgrade"], &[], &flags)
        } else {
            self.prompt_run("apk", &["add", "-u"], kws, &flags)
        }
    }

    /// Suy refreshes the local package database, then updates outdated packages.
    fn suy(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        let mut flags: Vec<&str> = flags.to_vec();
        if self.cfg.no_cache {
            flags.push("--no-cache");
        }
        if kws.is_empty() {
            self.prompt_run("apk", &["upgrade", "-U", "-a"], &[], &flags)
        } else {
            self.prompt_run("apk", &["add", "-U", "-u"], kws, &flags)
        }
    }

    /// Sw retrieves all packages from the server, but does not install/upgrade anything.
    fn sw(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.prompt_run("apk", &["fetch"], kws, flags)
    }

    /// Sy refreshes the local package database.
    fn sy(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("apk", &["update"], &[], flags)?;
        if !kws.is_empty() {
            self.s(kws, flags)?;
        }
        Ok(())
    }

    /// U upgrades or adds package(s) to the system and installs the required dependencies from sync repositories.
    fn u(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        let mut flags: Vec<&str> = flags.to_vec();
        if self.cfg.no_cache {
            flags.push("--no-cache");
        }
        self.prompt_run("apk", &["add", "--allow-untrusted"], kws, &flags)
    }
}
