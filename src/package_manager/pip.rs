use super::PackageManager;
use crate::error::Error;
use crate::exec::{self, Mode, PROMPT_RUN};

pub struct Pip {
    pub cmd: String,
    pub no_confirm: bool,
}

impl Pip {
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

impl PackageManager for Pip {
    /// Get the name of the package manager.
    fn name(&self) -> String {
        "pip".into()
    }

    /// Q generates a list of installed packages.
    fn q(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        if kws.is_empty() {
            self.just_run(&self.cmd, &["list"], kws, flags)
        } else {
            self.qs(kws, flags)
        }
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
            exec::print_cmd(cmd, subcmd, &[], flags, PROMPT_RUN);
            let out_bytes = exec::exec(cmd, subcmd, &[], flags, Mode::Mute)?;
            search(&String::from_utf8(out_bytes)?);
            Ok(())
        };

        search_output(&self.cmd, &["list"])
    }

    /// Qu lists packages which have an update available.
    fn qu(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("pip", &["list", "--outdated"], kws, flags)
    }

    /// R removes a single package, leaving all of its dependencies installed.
    fn r(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.prompt_run(&self.cmd, &["uninstall"], kws, flags)
    }

    /// S installs one or more packages by name.
    fn s(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run(&self.cmd, &["install"], kws, flags)
    }

    /// Sc removes all the cached packages that are not currently installed, and the unused sync database.
    fn sc(&self, _kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run(&self.cmd, &["cache", "purge"], &[], flags)
    }

    /// Si displays remote package information: name, version, description, etc.
    fn si(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run(&self.cmd, &["show"], kws, flags)
    }

    /// Ss searches for package(s) by searching the expression in name, description, short description.
    fn ss(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run(&self.cmd, &["search"], kws, flags)
    }

    /// Sw retrieves all packages from the server, but does not install/upgrade anything.
    fn sw(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run(&self.cmd, &["download"], kws, flags)
    }
}
