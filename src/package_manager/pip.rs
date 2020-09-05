use super::{PackageManager, PromptStrategy, Strategies};
use crate::dispatch::config::Config;
use crate::error::Error;
use crate::exec::{self, Cmd, Mode};
use crate::print::{self, PROMPT_RUN};

pub struct Pip {
    pub cmd: String,
    pub cfg: Config,
}

lazy_static! {
    static ref PROMPT_STRAT: Strategies = Strategies {
        prompt: PromptStrategy::native_prompt(&["-y"]),
        ..Default::default()
    };
}

impl PackageManager for Pip {
    /// Get the name of the package manager.
    fn name(&self) -> String {
        "pip".into()
    }

    /// Q generates a list of installed packages.
    fn q(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        if kws.is_empty() {
            self.just_run_default(Cmd::new(&[&self.cmd, "list"]).flags(flags))
        } else {
            self.qs(kws, flags)
        }
    }

    fn cfg(&self) -> Config {
        self.cfg.clone()
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

        let search_output = |cmd| {
            let cmd = Cmd::new(cmd).flags(flags);
            print::print_cmd(&cmd, PROMPT_RUN);
            let out_bytes = cmd.exec(Mode::Mute)?;
            search(&String::from_utf8(out_bytes)?);
            Ok(())
        };

        search_output(&[&self.cmd, "list"])
    }

    /// Qu lists packages which have an update available.
    fn qu(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run_default(
            Cmd::new(&[&self.cmd, "list", "--outdated"])
                .kws(kws)
                .flags(flags),
        )
    }

    /// R removes a single package, leaving all of its dependencies installed.
    fn r(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run(
            Cmd::new(&[&self.cmd, "uninstall"]).kws(kws).flags(flags),
            Default::default(),
            PROMPT_STRAT.clone(),
        )
    }

    /// S installs one or more packages by name.
    fn s(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run(
            Cmd::new(&[&self.cmd, "install"]).kws(kws).flags(flags),
            Default::default(),
            PROMPT_STRAT.clone(),
        )
    }

    /// Sc removes all the cached packages that are not currently installed, and the unused sync database.
    fn sc(&self, _kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run_default(Cmd::new(&[&self.cmd, "cache", "purge"]).flags(flags))
    }

    /// Si displays remote package information: name, version, description, etc.
    fn si(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run_default(Cmd::new(&[&self.cmd, "show"]).kws(kws).flags(flags))
    }

    /// Ss searches for package(s) by searching the expression in name, description, short description.
    fn ss(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run_default(Cmd::new(&[&self.cmd, "search"]).kws(kws).flags(flags))
    }

    /// Su updates outdated packages.
    fn su(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        if !kws.is_empty() {
            self.just_run_default(
                Cmd::new(&[&self.cmd, "install", "--upgrade"])
                    .kws(kws)
                    .flags(flags),
            )
        } else {
            Err(format!("Operation `su` unimplemented for `{}`", self.name()).into())
        }
    }

    /// Sw retrieves all packages from the server, but does not install/upgrade anything.
    fn sw(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run_default(Cmd::new(&[&self.cmd, "download"]).kws(kws).flags(flags))
    }
}
