use async_trait::async_trait;
use once_cell::sync::Lazy;
use tap::prelude::*;
use tokio::sync::Mutex;

use super::{DryRunStrategy, NoCacheStrategy, Pm, PmHelper, PmMode, PromptStrategy, Strategy};
use crate::{
    dispatch::config::Config,
    error::{Error, Result},
    exec::{self, Cmd, StatusCode},
    print::{self, PROMPT_INFO, PROMPT_RUN},
};

#[derive(Debug)]
pub struct Brew {
    cfg: Config,
    code: Mutex<StatusCode>,
}

static STRAT_PROMPT: Lazy<Strategy> = Lazy::new(|| Strategy {
    prompt: PromptStrategy::CustomPrompt,
    ..Strategy::default()
});

static STRAT_INSTALL: Lazy<Strategy> = Lazy::new(|| Strategy {
    prompt: PromptStrategy::CustomPrompt,
    no_cache: NoCacheStrategy::Scc,
    ..Strategy::default()
});

impl Brew {
    async fn search_regex(&self, cmd: &[&str], kws: &[&str], flags: &[&str]) -> Result<()> {
        let cmd = Cmd::new(cmd).flags(flags);
        if !self.cfg.dry_run {
            print::print_cmd(&cmd, PROMPT_RUN);
        }
        let out_bytes = self
            .check_output(cmd, PmMode::Mute, &Strategy::default())
            .await?
            .contents;

        exec::grep_print(&String::from_utf8(out_bytes)?, kws)?;
        Ok(())
    }
}

impl Brew {
    #[must_use]
    pub fn new(cfg: Config) -> Self {
        Brew {
            cfg,
            code: Mutex::default(),
        }
    }
}

#[async_trait]
impl Pm for Brew {
    /// Gets the name of the package manager.
    fn name(&self) -> &str {
        "brew"
    }

    fn cfg(&self) -> &Config {
        &self.cfg
    }

    async fn code(&self) -> StatusCode {
        *self.code.lock().await
    }

    async fn set_code(&self, to: StatusCode) {
        *self.code.lock().await = to;
    }

    /// Q generates a list of installed packages.
    async fn q(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        if kws.is_empty() {
            self.run(Cmd::new(&["brew", "list"]).flags(flags)).await
        } else {
            self.qs(kws, flags).await
        }
    }

    /// Qc shows the changelog of a package.
    async fn qc(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(&["brew", "log"]).kws(kws).flags(flags))
            .await
    }

    /// Qi displays local package information: name, version, description, etc.
    async fn qi(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.si(kws, flags).await
    }

    /// Ql displays files provided by local package.
    async fn ql(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        // TODO: it seems that the output of `brew list python` in fish has a mechanism
        // against duplication: /usr/local/Cellar/python/3.6.0/Frameworks/
        // Python.framework/ (1234 files)
        self.run(Cmd::new(&["brew", "list"]).kws(kws).flags(flags))
            .await
    }

    /// Qs searches locally installed package for names or descriptions.
    // According to https://www.archlinux.org/pacman/pacman.8.html#_query_options_apply_to_em_q_em_a_id_qo_a,
    // when including multiple search terms, only packages with descriptions
    // matching ALL of those terms are returned.
    async fn qs(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        // ! `brew list` lists all formulae and casks only when using tty.
        self.search_regex(&["brew", "list", "--formula"], kws, flags)
            .await?;
        if cfg!(target_os = "macos") {
            self.search_regex(&["brew", "list", "--cask"], kws, flags)
                .await?;
        }

        Ok(())
    }

    /// Qu lists packages which have an update available.
    async fn qu(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(&["brew", "outdated"]).kws(kws).flags(flags))
            .await
    }

    /// R removes a single package, leaving all of its dependencies installed.
    async fn r(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(&["brew", "uninstall"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_PROMPT))
            .await
    }

    /// Rss removes a package and its dependencies which are not required by any
    /// other installed package.
    async fn rss(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        let strat = Strategy {
            dry_run: DryRunStrategy::with_flags(&["--dry-run"]),
            ..Strategy::default()
        };
        let err_msg = Cmd::new(&["brew", "rmtree"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.check_output(cmd, PmMode::default(), &strat))
            .await?
            .contents
            .pipe(String::from_utf8)?;

        let pattern = "Unknown command: rmtree";
        if !exec::grep(&err_msg, &[pattern])?.is_empty() {
            print::print_msg(
                "`rmtree` is not installed. You may install it with the following command:",
                PROMPT_INFO,
            );
            print::print_msg("`brew tap beeftornado/rmtree`", PROMPT_INFO);
            return Err(Error::OtherError("`rmtree` required".into()));
        }
        Ok(())
    }

    /// S installs one or more packages by name.
    async fn s(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(if self.cfg.needed {
            &["brew", "install"]
        } else {
            // If the package is not installed, `brew reinstall` behaves just like `brew
            // install`, so `brew reinstall` matches perfectly the behavior of
            // `pacman -S`.
            &["brew", "reinstall"]
        })
        .kws(kws)
        .flags(flags)
        .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_INSTALL))
        .await
    }

    /// Sc removes all the cached packages that are not currently installed, and
    /// the unused sync database.
    async fn sc(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        let strat = Strategy {
            dry_run: DryRunStrategy::with_flags(&["--dry-run"]),
            prompt: PromptStrategy::CustomPrompt,
            ..Strategy::default()
        };
        Cmd::new(&["brew", "cleanup"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, PmMode::default(), &strat))
            .await
    }

    /// Scc removes all files from the cache.
    async fn scc(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        let strat = Strategy {
            dry_run: DryRunStrategy::with_flags(&["--dry-run"]),
            prompt: PromptStrategy::CustomPrompt,
            ..Strategy::default()
        };
        Cmd::new(&["brew", "cleanup", "-s"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, PmMode::default(), &strat))
            .await
    }

    /// Si displays remote package information: name, version, description, etc.
    async fn si(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(&["brew", "info"]).kws(kws).flags(flags))
            .await
    }

    /// Sii displays packages which require X to be installed, aka reverse
    /// dependencies.
    async fn sii(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(&["brew", "uses"]).kws(kws).flags(flags))
            .await
    }

    /// Ss searches for package(s) by searching the expression in name,
    /// description, short description.
    async fn ss(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(&["brew", "search"]).kws(kws).flags(flags))
            .await
    }

    /// Su updates outdated packages.
    async fn su(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(&["brew", "upgrade"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_INSTALL))
            .await
    }

    /// Suy refreshes the local package database, then updates outdated
    /// packages.
    async fn suy(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.sy(&[], flags).await?;
        self.su(kws, flags).await
    }

    /// Sw retrieves all packages from the server, but does not install/upgrade
    /// anything.
    async fn sw(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(&["brew", "fetch"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_PROMPT))
            .await
    }

    /// Sy refreshes the local package database.
    async fn sy(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.run(Cmd::new(&["brew", "update"]).flags(flags)).await?;
        if !kws.is_empty() {
            self.s(kws, flags).await?;
        }
        Ok(())
    }
}
