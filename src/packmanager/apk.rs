use super::PackManager;
use crate::error::Error;
use crate::exec::{self, Mode, PROMPT_RUN};
use regex::Regex;

pub struct Apk {
    pub dry_run: bool,
    pub no_confirm: bool,
}

impl Apk {
    /// A helper method to simplify prompted command invocation.
    fn prompt_run(&self, cmd: &str, subcmd: &[&str], kws: &[&str]) -> Result<(), Error> {
        let mode = if self.dry_run {
            Mode::DryRun
        } else if self.no_confirm {
            Mode::CheckErr
        } else {
            Mode::Prompt
        };
        exec::exec(cmd, subcmd, kws, mode)?;
        Ok(())
    }
}

impl PackManager for Apk {
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
        if kws.is_empty() {
            self.just_run("apk", &["info"], &[])
        } else {
            self.qs(kws)
        }
    }

    /// Qi displays local package information: name, version, description, etc.
    fn qi(&self, kws: &[&str]) -> Result<(), Error> {
        self.si(kws)
    }

    /// Ql displays files provided by local package.
    fn ql(&self, kws: &[&str]) -> Result<(), Error> {
        self.just_run("apk", &["info", "-L"], kws)
    }

    /// Qo queries the package which provides FILE.
    fn qo(&self, kws: &[&str]) -> Result<(), Error> {
        self.just_run("apk", &["info", "--who-owns"], kws)
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

        search_output("apk", &["info", "-d"])
    }

    /// Qu lists packages which have an update available.
    //? Is that the right way to input '<'?
    fn qu(&self, _kws: &[&str]) -> Result<(), Error> {
        self.just_run("apk", &["version", "-l"], &["'<'"])
    }

    /// R removes a single package, leaving all of its dependencies installed.
    fn r(&self, kws: &[&str]) -> Result<(), Error> {
        self.just_run("apk", &["del"], kws)
    }

    /// Rn removes a package and skips the generation of configuration backup files.
    fn rn(&self, kws: &[&str]) -> Result<(), Error> {
        self.just_run("apk", &["del", "--purge"], kws)
    }

    /// Rns removes a package and its dependencies which are not required by any other installed package, and skips the generation of configuration backup files.
    fn rns(&self, kws: &[&str]) -> Result<(), Error> {
        self.just_run("apk", &["del", "--purge", "-r"], kws)
    }

    /// Rs removes a package and its dependencies which are not required by any other installed package.
    fn rs(&self, kws: &[&str]) -> Result<(), Error> {
        self.just_run("apk", &["del", "-r"], kws)
    }

    /// S installs one or more packages by name.
    fn s(&self, kws: &[&str]) -> Result<(), Error> {
        self.prompt_run("apk", &["add"], kws)
    }

    /// Sc removes all the cached packages that are not currently installed, and the unused sync database.
    fn sc(&self, _kws: &[&str]) -> Result<(), Error> {
        self.prompt_run("apk", &["cache", "-v", "clean"], &[])
    }

    /// Si displays remote package information: name, version, description, etc.
    fn si(&self, kws: &[&str]) -> Result<(), Error> {
        self.just_run("apk", &["info", "-a"], kws)
    }

    /// Sii displays packages which require X to be installed, aka reverse dependencies.
    fn sii(&self, kws: &[&str]) -> Result<(), Error> {
        self.just_run("apk", &["info", "-r"], kws)
    }

    /// Sl displays a list of all packages in all installation sources that are handled by the packages management.
    fn sl(&self, kws: &[&str]) -> Result<(), Error> {
        self.just_run("apk", &["search"], kws)
    }

    /// Ss searches for package(s) by searching the expression in name, description, short description.
    fn ss(&self, kws: &[&str]) -> Result<(), Error> {
        self.just_run("apk", &["search", "-v"], kws)
    }

    /// Su updates outdated packages.
    fn su(&self, kws: &[&str]) -> Result<(), Error> {
        if kws.is_empty() {
            self.prompt_run("apk", &["upgrade"], &[])
        } else {
            self.prompt_run("apk", &["add", "-u"], kws)
        }
    }

    /// Suy refreshes the local package database, then updates outdated packages.
    fn suy(&self, kws: &[&str]) -> Result<(), Error> {
        if kws.is_empty() {
            self.prompt_run("apk", &["upgrade", "-U", "-a"], &[])
        } else {
            self.prompt_run("apk", &["add", "-U", "-u"], kws)
        }
    }

    /// Sw retrieves all packages from the server, but does not install/upgrade anything.
    fn sw(&self, kws: &[&str]) -> Result<(), Error> {
        self.prompt_run("apk", &["fetch"], kws)
    }

    /// Sy refreshes the local package database.
    fn sy(&self, _kws: &[&str]) -> Result<(), Error> {
        self.just_run("apk", &["update"], &[])
    }

    /// U upgrades or adds package(s) to the system and installs the required dependencies from sync repositories.
    fn u(&self, kws: &[&str]) -> Result<(), Error> {
        self.prompt_run("apk", &["add", "--allow-untrusted"], kws)
    }
}
