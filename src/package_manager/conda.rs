use super::PackageManager;
use crate::dispatch::config::Config;
use crate::error::Error;
use crate::exec::{self, Mode};
use crate::print::{self, PROMPT_RUN};

pub struct Conda {
    pub cfg: Config,
}

impl Conda {
    /// A helper method to simplify prompted command invocation.
    fn prompt_run(
        &self,
        cmd: &str,
        subcmd: &[&str],
        kws: &[&str],
        flags: &[&str],
    ) -> Result<(), Error> {
        let mut subcmd: Vec<&str> = subcmd.to_vec();
        if self.cfg.no_confirm {
            subcmd.push("-y");
        }
        self.just_run(cmd, &subcmd, kws, flags)
    }
}

impl PackageManager for Conda {
    /// Get the name of the package manager.
    fn name(&self) -> String {
        "conda".into()
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
            self.just_run("conda", &["list"], kws, flags)
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
            print::print_cmd(cmd, subcmd, &[], flags, PROMPT_RUN);
            let out_bytes = exec::exec(cmd, subcmd, &[], flags, Mode::Mute)?;
            search(&String::from_utf8(out_bytes)?);
            Ok(())
        };

        search_output("conda", &["list"])
    }

    /// R removes a single package, leaving all of its dependencies installed.
    fn r(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.prompt_run("conda", &["remove"], kws, flags)
    }

    /// S installs one or more packages by name.
    fn s(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.prompt_run("conda", &["install"], kws, flags)
    }

    /// Sc removes all the cached packages that are not currently installed, and the unused sync database.
    fn sc(&self, _kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.prompt_run("conda", &["clean", "--all"], &[], flags)
    }

    /// Si displays remote package information: name, version, description, etc.
    fn si(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run("conda", &["search", "--info"], kws, flags)
    }

    /// Ss searches for package(s) by searching the expression in name, description, short description.
    fn ss(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        let kws: Vec<String> = kws.iter().map(|&s| format!("*{}*", s)).collect();
        kws.iter()
            .map(|kw| self.just_run("conda", &["search"], &[kw], flags))
            .collect()
    }

    /// Su updates outdated packages.
    fn su(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.prompt_run("conda", &["update", "--all"], kws, flags)
    }

    /// Suy refreshes the local package database, then updates outdated packages.
    fn suy(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.su(kws, flags)
    }
}
