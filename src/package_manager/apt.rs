use super::{NoCacheStrategy, PackageManager, PromptStrategy, Strategies};
use crate::dispatch::config::Config;
use crate::error::Error;
use crate::exec::Cmd;

pub struct Apt {
    pub cfg: Config,
}

lazy_static! {
    static ref PROMPT_STRAT: Strategies = Strategies {
        prompt: PromptStrategy::native_prompt(&["--yes"]),
        ..Default::default()
    };
    static ref INSTALL_STRAT: Strategies = Strategies {
        prompt: PromptStrategy::native_prompt(&["--yes"]),
        no_cache: NoCacheStrategy::Scc,
        ..Default::default()
    };
}

impl PackageManager for Apt {
    /// Get the name of the package manager.
    fn name(&self) -> String {
        "apt".into()
    }

    fn cfg(&self) -> Config {
        self.cfg.clone()
    }

    /// Q generates a list of installed packages.
    fn q(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run_default(Cmd::new(&["apt", "list"]).kws(kws).flags(flags))
    }

    /// Qi displays local package information: name, version, description, etc.
    fn qi(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run_default(Cmd::new(&["dpkg-query", "-s"]).kws(kws).flags(flags))
    }

    /// Qo queries the package which provides FILE.
    fn qo(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run_default(Cmd::new(&["dpkg-query", "-S"]).kws(kws).flags(flags))
    }

    /// Qp queries a package supplied on the command line rather than an entry in the package management database.
    fn qp(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run_default(Cmd::new(&["dpkg-deb", "-I"]).kws(kws).flags(flags))
    }

    /// Qu lists packages which have an update available.
    fn qu(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run_default(
            Cmd::new(&["apt", "upgrade", "--trivial-only"])
                .kws(kws)
                .flags(flags),
        )
    }

    /// R removes a single package, leaving all of its dependencies installed.
    fn r(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run(
            Cmd::new(&["apt", "remove"]).kws(kws).flags(flags),
            Default::default(),
            PROMPT_STRAT.clone(),
        )
    }

    /// Rn removes a package and skips the generation of configuration backup files.
    fn rn(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run(
            Cmd::new(&["apt", "purge"]).kws(kws).flags(flags),
            Default::default(),
            PROMPT_STRAT.clone(),
        )
    }

    /// Rns removes a package and its dependencies which are not required by any other installed package, and skips the generation of configuration backup files.
    fn rns(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run(
            Cmd::new(&["apt", "autoremove", "--purge"])
                .kws(kws)
                .flags(flags),
            Default::default(),
            PROMPT_STRAT.clone(),
        )
    }

    /// Rs removes a package and its dependencies which are not required by any other installed package,
    /// and not explicitly installed by the user.
    fn rs(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run(
            Cmd::new(&["apt", "autoremove"]).kws(kws).flags(flags),
            Default::default(),
            PROMPT_STRAT.clone(),
        )
    }

    /// S installs one or more packages by name.
    fn s(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        let cmd: &[&str] = if self.cfg.needed {
            &["apt", "install"]
        } else {
            &["apt", "install", "--reinstall"]
        };
        self.just_run(
            Cmd::new(cmd).kws(kws).flags(flags),
            Default::default(),
            INSTALL_STRAT.clone(),
        )
    }

    /// Sc removes all the cached packages that are not currently installed, and the unused sync database.
    fn sc(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run(
            Cmd::new(&["apt", "clean"]).kws(kws).flags(flags),
            Default::default(),
            PROMPT_STRAT.clone(),
        )
    }

    /// Scc removes all files from the cache.
    fn scc(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run(
            Cmd::new(&["apt", "autoclean"]).kws(kws).flags(flags),
            Default::default(),
            PROMPT_STRAT.clone(),
        )
    }

    /// Si displays remote package information: name, version, description, etc.
    fn si(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run_default(Cmd::new(&["apt", "show"]).kws(kws).flags(flags))
    }

    /// Sii displays packages which require X to be installed, aka reverse dependencies.
    fn sii(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run_default(Cmd::new(&["apt", "rdepends"]).kws(kws).flags(flags))
    }

    /// Ss searches for package(s) by searching the expression in name, description, short description.
    fn ss(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run_default(Cmd::new(&["apt", "search"]).kws(kws).flags(flags))
    }

    /// Su updates outdated packages.
    fn su(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        if kws.is_empty() {
            self.just_run(
                Cmd::new(&["apt", "upgrade"]).flags(flags),
                Default::default(),
                PROMPT_STRAT.clone(),
            )?;
            self.just_run(
                Cmd::new(&["apt", "dist-upgrade"]).flags(flags),
                Default::default(),
                INSTALL_STRAT.clone(),
            )
        } else {
            self.s(kws, flags)
        }
    }

    /// Suy refreshes the local package database, then updates outdated packages.
    fn suy(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.sy(kws, flags)?;
        self.su(kws, flags)
    }

    /// Sw retrieves all packages from the server, but does not install/upgrade anything.
    fn sw(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run_default(
            Cmd::new(&["apt", "install", "--download-only"])
                .kws(kws)
                .flags(flags),
        )
    }

    /// Sy refreshes the local package database.
    fn sy(&self, kws: &[&str], flags: &[&str]) -> Result<(), Error> {
        self.just_run_default(Cmd::new(&["apt", "update"]).kws(kws).flags(flags))?;
        if !kws.is_empty() {
            self.s(kws, flags)?;
        }
        Ok(())
    }
}
