use super::{DryRunStrategy, NoCacheStrategy, PackageManager, PmMode, PromptStrategy, Strategies};
use crate::dispatch::config::Config;
use crate::error::Error;
use crate::exec::{self, Cmd};

pub struct Zypper {
    pub cfg: Config,
}

impl Zypper {
    fn check_dry(&self, cmd: Cmd) -> Result<(), Error> {
        self.just_run(cmd, Default::default(), CHECK_DRY_STRAT.clone())
    }
}

lazy_static! {
    static ref CHECK_DRY_STRAT: Strategies = Strategies {
        dry_run: DryRunStrategy::with_flags(&["--dry-run"]),
        ..Default::default()
    };
    static ref PROMPT_STRAT: Strategies = Strategies {
        prompt: PromptStrategy::native_prompt(&["-y"]),
        dry_run: DryRunStrategy::with_flags(&["--dry-run"]),
        ..Default::default()
    };
    static ref INSTALL_STRAT: Strategies = Strategies {
        prompt: PromptStrategy::native_prompt(&["-y"]),
        no_cache: NoCacheStrategy::Scc,
        dry_run: DryRunStrategy::with_flags(&["--dry-run"]),
    };
}

impl PackageManager for Zypper {
    /// Get the name of the package manager.
    fn name(&self) -> String {
        "zypper".into()
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

    /// Qi displays local package information: name, version, description, etc.
    fn qi(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.si(kws, flags)
    }

    /// Ql displays files provided by local package.
    fn ql(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run_default(Cmd::new(&["rpm", "-ql"]).kws(kws).flags(flags))
    }

    /// Qm lists packages that are installed but are not available in any installation source (anymore).
    fn qm(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        let search = |contents: &str, pattern: &str| {
            exec::grep(contents, &[pattern])
                .iter()
                .for_each(|ln| println!("{}", ln))
        };

        let out_bytes = self.run(
            Cmd::new(&["zypper", "search", "-si"]).kws(kws).flags(flags),
            PmMode::Mute,
            Default::default(),
        )?;
        let out = String::from_utf8(out_bytes)?;

        search(&out, "System Packages");
        Ok(())
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
    fn qs(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.check_dry(
            Cmd::new(&["zypper", "search", "--installed-only"])
                .kws(kws)
                .flags(flags),
        )
    }

    /// Qu lists packages which have an update available.
    fn qu(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.check_dry(Cmd::new(&["zypper", "list-updates"]).kws(kws).flags(flags))
    }

    /// R removes a single package, leaving all of its dependencies installed.
    fn r(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run(
            Cmd::new(&["zypper", "remove"]).kws(kws).flags(flags),
            Default::default(),
            PROMPT_STRAT.clone(),
        )
    }

    /// Rss removes a package and its dependencies which are not required by any other installed package.
    fn rss(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run(
            Cmd::new(&["zypper", "remove", "--clean-deps"])
                .kws(kws)
                .flags(flags),
            Default::default(),
            PROMPT_STRAT.clone(),
        )
    }

    /// S installs one or more packages by name.
    fn s(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run(
            Cmd::new(&["zypper", "install"]).kws(kws).flags(flags),
            Default::default(),
            INSTALL_STRAT.clone(),
        )
    }

    /// Sc removes all the cached packages that are not currently installed, and the unused sync database.
    fn sc(&self, _kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run(
            Cmd::new(&["zypper", "clean"]).flags(flags),
            Default::default(),
            Strategies {
                prompt: PromptStrategy::CustomPrompt,
                ..Default::default()
            },
        )
    }

    /// Scc removes all files from the cache.
    fn scc(&self, _kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.sc(_kws, flags)
    }

    /// Si displays remote package information: name, version, description, etc.
    fn si(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.check_dry(
            Cmd::new(&["zypper", "info", "--requires"])
                .kws(kws)
                .flags(flags),
        )
    }

    /// Sl displays a list of all packages in all installation sources that are handled by the packages management.
    fn sl(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        let cmd: &[&str] = if kws.is_empty() {
            &["zypper", "packages", "-R"]
        } else {
            &["zypper", "packages", "-r"]
        };
        self.check_dry(Cmd::new(cmd).kws(kws).flags(flags))
    }

    /// Ss searches for package(s) by searching the expression in name, description, short description.
    fn ss(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.check_dry(Cmd::new(&["zypper", "search"]).kws(kws).flags(flags))
    }

    /// Su updates outdated packages.
    fn su(&self, _kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.check_dry(Cmd::new(&["zypper", "--no-refresh", "dist-upgrade"]).flags(flags))?;
        if self.cfg.no_cache {
            self.sccc(_kws, flags)?;
        }
        Ok(())
    }

    /// Suy refreshes the local package database, then updates outdated packages.
    fn suy(&self, _kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run(
            Cmd::new(&["zypper", "dist-upgrade"]).flags(flags),
            Default::default(),
            INSTALL_STRAT.clone(),
        )
    }

    /// Sw retrieves all packages from the server, but does not install/upgrade anything.
    fn sw(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run(
            Cmd::new(&["zypper", "install", "--download-only"])
                .kws(kws)
                .flags(flags),
            Default::default(),
            INSTALL_STRAT.clone(),
        )
    }

    /// Sy refreshes the local package database.
    fn sy(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.check_dry(Cmd::new(&["zypper", "refresh"]).flags(flags))?;
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
