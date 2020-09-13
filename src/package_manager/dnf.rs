use super::{NoCacheStrategy, PackageManager, PmMode, PromptStrategy, Strategies};
use crate::error::Error;
use crate::exec::{self, Cmd};
use crate::print::PROMPT_RUN;
use crate::{dispatch::config::Config, print};

pub struct Dnf {
    pub cfg: Config,
}

lazy_static! {
    static ref PROMPT_STRAT: Strategies = Strategies {
        prompt: PromptStrategy::native_prompt(&["-y"]),
        ..Default::default()
    };
    static ref INSTALL_STRAT: Strategies = Strategies {
        prompt: PromptStrategy::native_prompt(&["-y"]),
        no_cache: NoCacheStrategy::Sccc,
        ..Default::default()
    };
}

impl PackageManager for Dnf {
    /// Get the name of the package manager.
    fn name(&self) -> String {
        "dnf".into()
    }

    fn cfg(&self) -> Config {
        self.cfg.clone()
    }

    /// Q generates a list of installed packages.
    fn q(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        if kws.is_empty() {
            self.just_run_default(
                Cmd::new(&["rpm", "-qa", "--qf", "%{NAME} %{VERSION}\\n"]).flags(flags),
            )
        } else {
            self.qs(kws, flags)
        }
    }

    /// Qc shows the changelog of a package.
    fn qc(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run_default(Cmd::new(&["rpm", "-q", "changelog"]).kws(kws).flags(flags))
    }

    /// Qe lists packages installed explicitly (not as dependencies).
    fn qe(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run_default(
            Cmd::new(&["dnf", "repoquery", "--userinstalled"])
                .kws(kws)
                .flags(flags),
        )
    }

    /// Qi displays local package information: name, version, description, etc.
    fn qi(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.si(kws, flags)
    }

    /// Ql displays files provided by local package.
    fn ql(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run_default(Cmd::new(&["rpm", "-ql"]).kws(kws).flags(flags))
    }

    /// Qm lists packages that are installed but are not available in any installation source (anymore).
    fn qm(&self, _kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run_default(Cmd::new(&["dnf", "list", "extras"]).flags(flags))
    }

    /// Qo queries the package which provides FILE.
    fn qo(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run_default(Cmd::new(&["rpm", "-qf"]).kws(kws).flags(flags))
    }

    /// Qp queries a package supplied on the command line rather than an entry in the package management database.
    fn qp(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run_default(Cmd::new(&["rpm", "-qip"]).kws(kws).flags(flags))
    }

    /// Qs searches locally installed package for names or descriptions.
    // According to https://www.archlinux.org/pacman/pacman.8.html#_query_options_apply_to_em_q_em_a_id_qo_a,
    // when including multiple search terms, only packages with descriptions matching ALL of those terms are returned.
    // TODO: Is this right?
    fn qs(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        let search = |contents: &str| {
            exec::grep(contents, kws)
                .iter()
                .for_each(|ln| println!("{}", ln))
        };

        let search_output = |cmd| {
            let cmd = Cmd::new(cmd).flags(flags);
            if !self.cfg.dry_run {
                print::print_cmd(&cmd, PROMPT_RUN);
            }
            let out_bytes = self.run(cmd, PmMode::Mute, Default::default())?;
            search(&String::from_utf8(out_bytes)?);
            Ok(())
        };

        search_output(&["rpm", "-qa"])
    }

    /// Qu lists packages which have an update available.
    fn qu(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run_default(Cmd::new(&["dnf", "list", "updates"]).kws(kws).flags(flags))
    }

    /// R removes a single package, leaving all of its dependencies installed.
    fn r(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run(
            Cmd::new(&["dnf", "remove"]).kws(kws).flags(flags),
            Default::default(),
            PROMPT_STRAT.clone(),
        )
    }

    /// S installs one or more packages by name.
    fn s(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run(
            Cmd::new(&["dnf", "install"]).kws(kws).flags(flags),
            Default::default(),
            INSTALL_STRAT.clone(),
        )
    }

    /// Sc removes all the cached packages that are not currently installed, and the unused sync database.
    fn sc(&self, _kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run(
            Cmd::new(&["dnf", "clean", "expire-cache"]).flags(flags),
            Default::default(),
            Strategies {
                prompt: PromptStrategy::CustomPrompt,
                ..Default::default()
            },
        )
    }

    /// Scc removes all files from the cache.
    fn scc(&self, _kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run(
            Cmd::new(&["dnf", "clean", "packages"]).flags(flags),
            Default::default(),
            Strategies {
                prompt: PromptStrategy::CustomPrompt,
                ..Default::default()
            },
        )
    }

    /// Sccc ...
    /// What is this?
    fn sccc(&self, _kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run(
            Cmd::new(&["dnf", "clean", "all"]).flags(flags),
            Default::default(),
            Strategies {
                prompt: PromptStrategy::CustomPrompt,
                ..Default::default()
            },
        )
    }

    /// Si displays remote package information: name, version, description, etc.
    fn si(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run_default(Cmd::new(&["dnf", "info"]).kws(kws).flags(flags))
    }

    /// Sg lists all packages belonging to the GROUP.
    fn sg(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        let cmd: &[&str] = if kws.is_empty() {
            &["dnf", "group", "list"]
        } else {
            &["dnf", "group", "info"]
        };
        self.just_run_default(Cmd::new(cmd).kws(kws).flags(flags))
    }

    /// Sl displays a list of all packages in all installation sources that are handled by the packages management.
    fn sl(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run_default(
            Cmd::new(&["dnf", "list", "available"])
                .kws(kws)
                .flags(flags),
        )
    }

    /// Ss searches for package(s) by searching the expression in name, description, short description.
    fn ss(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run_default(Cmd::new(&["dnf", "search"]).kws(kws).flags(flags))
    }

    /// Su updates outdated packages.
    fn su(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run(
            Cmd::new(&["dnf", "upgrade"]).kws(kws).flags(flags),
            Default::default(),
            INSTALL_STRAT.clone(),
        )
    }

    /// Suy refreshes the local package database, then updates outdated packages.
    fn suy(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.su(kws, flags)
    }

    /// Sw retrieves all packages from the server, but does not install/upgrade anything.
    fn sw(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run(
            Cmd::new(&["dnf", "download"]).kws(kws).flags(flags),
            Default::default(),
            INSTALL_STRAT.clone(),
        )
    }

    /// Sy refreshes the local package database.
    fn sy(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.sc(&[], flags)?;
        self.just_run_default(Cmd::new(&["dnf", "check-update"]).flags(flags))?;
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
