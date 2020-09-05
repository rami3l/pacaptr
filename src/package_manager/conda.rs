use super::{PackageManager, PromptStrategy, Strategies};
use crate::dispatch::config::Config;
use crate::error::Error;
use crate::exec::{self, Cmd, Mode};
use crate::print::{self, PROMPT_RUN};

pub struct Conda {
    pub cfg: Config,
}

lazy_static! {
    static ref PROMPT_STRAT: Strategies = Strategies {
        prompt: PromptStrategy::native_prompt(&["-y"]),
        ..Default::default()
    };
}

impl PackageManager for Conda {
    /// Get the name of the package manager.
    fn name(&self) -> String {
        "conda".into()
    }

    fn cfg(&self) -> Config {
        self.cfg.clone()
    }

    /// Q generates a list of installed packages.
    fn q(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        if kws.is_empty() {
            self.just_run_default(Cmd::new(&["conda", "list"]).flags(flags))
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

        let search_output = |cmd| {
            let cmd = Cmd::new(cmd).flags(flags);
            print::print_cmd(&cmd, PROMPT_RUN);
            let out_bytes = cmd.exec(Mode::Mute)?;
            search(&String::from_utf8(out_bytes)?);
            Ok(())
        };

        search_output(&["conda", "list"])
    }

    /// R removes a single package, leaving all of its dependencies installed.
    fn r(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run(
            Cmd::new(&["conda", "remove"]).kws(kws).flags(flags),
            Default::default(),
            PROMPT_STRAT.clone(),
        )
    }

    /// S installs one or more packages by name.
    fn s(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run(
            Cmd::new(&["conda", "install"]).kws(kws).flags(flags),
            Default::default(),
            PROMPT_STRAT.clone(),
        )
    }

    /// Sc removes all the cached packages that are not currently installed, and the unused sync database.
    fn sc(&self, _kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run(
            Cmd::new(&["conda", "clean", "--all"]).flags(flags),
            Default::default(),
            PROMPT_STRAT.clone(),
        )
    }

    /// Si displays remote package information: name, version, description, etc.
    fn si(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run_default(
            Cmd::new(&["conda", "search", "--info"])
                .kws(kws)
                .flags(flags),
        )
    }

    /// Ss searches for package(s) by searching the expression in name, description, short description.
    fn ss(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        let kws: Vec<String> = kws.iter().map(|&s| format!("*{}*", s)).collect();
        kws.iter()
            .map(|kw| self.just_run_default(Cmd::new(&["conda", "search"]).kws(&[kw]).flags(flags)))
            .collect()
    }

    /// Su updates outdated packages.
    fn su(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run(
            Cmd::new(&["conda", "update", "--all"])
                .kws(kws)
                .flags(flags),
            Default::default(),
            PROMPT_STRAT.clone(),
        )
    }

    /// Suy refreshes the local package database, then updates outdated packages.
    fn suy(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.su(kws, flags)
    }
}
