use super::{DryRunStrategy, NoCacheStrategy, Pm, PmHelper, PmMode, PromptStrategy, Strategies};
use crate::dispatch::config::Config;
use crate::error::Result;
use crate::exec::{self, Cmd};
use async_trait::async_trait;
use lazy_static::lazy_static;

pub struct Zypper {
    pub cfg: Config,
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

impl Zypper {
    async fn check_dry(&self, cmd: Cmd) -> Result<()> {
        self.just_run(cmd, Default::default(), &CHECK_DRY_STRAT)
            .await
    }
}

#[async_trait]
impl Pm for Zypper {
    /// Get the name of the package manager.
    fn name(&self) -> String {
        "zypper".into()
    }

    fn cfg(&self) -> &Config {
        &self.cfg
    }

    /// Q generates a list of installed packages.
    async fn q(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        if kws.is_empty() {
            self.just_run_default(
                Cmd::new(&["rpm", "-qa", "--qf", "%{NAME} %{VERSION}\\n"]).flags(flags),
            )
            .await
        } else {
            self.qs(kws, flags).await
        }
    }

    /// Qc shows the changelog of a package.
    async fn qc(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run_default(Cmd::new(&["rpm", "-q", "changelog"]).kws(kws).flags(flags))
            .await
    }

    /// Qi displays local package information: name, version, description, etc.
    async fn qi(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.si(kws, flags).await
    }

    /// Ql displays files provided by local package.
    async fn ql(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run_default(Cmd::new(&["rpm", "-ql"]).kws(kws).flags(flags))
            .await
    }

    /// Qm lists packages that are installed but are not available in any installation source (anymore).
    async fn qm(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        let cmd = &["zypper", "search", "-si"];
        let cmd = Cmd::new(cmd).kws(kws).flags(flags);
        let out_bytes = self
            .run(cmd, PmMode::Mute, &Default::default())
            .await?
            .contents;
        let out = String::from_utf8(out_bytes)?;

        exec::grep_print(&out, &["System Packages"])?;
        Ok(())
    }

    /// Qo queries the package which provides FILE.
    async fn qo(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run_default(Cmd::new(&["rpm", "-qf"]).kws(kws).flags(flags))
            .await
    }

    /// Qp queries a package supplied on the command line rather than an entry in the package management database.
    async fn qp(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run_default(Cmd::new(&["rpm", "-qip"]).kws(kws).flags(flags))
            .await
    }

    /// Qs searches locally installed package for names or descriptions.
    // According to https://www.archlinux.org/pacman/pacman.8.html#_query_options_apply_to_em_q_em_a_id_qo_a,
    // when including multiple search terms, only packages with descriptions matching ALL of those terms are returned.
    async fn qs(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.check_dry(
            Cmd::new(&["zypper", "search", "--installed-only"])
                .kws(kws)
                .flags(flags),
        )
        .await
    }

    /// Qu lists packages which have an update available.
    async fn qu(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.check_dry(Cmd::new(&["zypper", "list-updates"]).kws(kws).flags(flags))
            .await
    }

    /// R removes a single package, leaving all of its dependencies installed.
    async fn r(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run(
            Cmd::with_sudo(&["zypper", "remove"]).kws(kws).flags(flags),
            Default::default(),
            &PROMPT_STRAT,
        )
        .await
    }

    /// Rss removes a package and its dependencies which are not required by any other installed package.
    async fn rss(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run(
            Cmd::with_sudo(&["zypper", "remove", "--clean-deps"])
                .kws(kws)
                .flags(flags),
            Default::default(),
            &PROMPT_STRAT,
        )
        .await
    }

    /// S installs one or more packages by name.
    async fn s(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run(
            Cmd::with_sudo(&["zypper", "install"]).kws(kws).flags(flags),
            Default::default(),
            &INSTALL_STRAT,
        )
        .await
    }

    /// Sc removes all the cached packages that are not currently installed, and the unused sync database.
    async fn sc(&self, _kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run(
            Cmd::with_sudo(&["zypper", "clean"]).flags(flags),
            Default::default(),
            &Strategies {
                prompt: PromptStrategy::CustomPrompt,
                ..Default::default()
            },
        )
        .await
    }

    /// Scc removes all files from the cache.
    async fn scc(&self, _kws: &[&str], flags: &[&str]) -> Result<()> {
        self.sc(_kws, flags).await
    }

    /// Si displays remote package information: name, version, description, etc.
    async fn si(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.check_dry(
            Cmd::new(&["zypper", "info", "--requires"])
                .kws(kws)
                .flags(flags),
        )
        .await
    }

    /// Sl displays a list of all packages in all installation sources that are handled by the packages management.
    async fn sl(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        let cmd: &[&str] = if kws.is_empty() {
            &["zypper", "packages", "-R"]
        } else {
            &["zypper", "packages", "-r"]
        };
        self.check_dry(Cmd::new(cmd).kws(kws).flags(flags)).await
    }

    /// Ss searches for package(s) by searching the expression in name, description, short description.
    async fn ss(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.check_dry(Cmd::new(&["zypper", "search"]).kws(kws).flags(flags))
            .await
    }

    /// Su updates outdated packages.
    async fn su(&self, _kws: &[&str], flags: &[&str]) -> Result<()> {
        self.check_dry(Cmd::with_sudo(&["zypper", "--no-refresh", "dist-upgrade"]).flags(flags))
            .await?;
        if self.cfg.no_cache {
            self.sccc(_kws, flags).await?;
        }
        Ok(())
    }

    /// Suy refreshes the local package database, then updates outdated packages.
    async fn suy(&self, _kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run(
            Cmd::with_sudo(&["zypper", "dist-upgrade"]).flags(flags),
            Default::default(),
            &INSTALL_STRAT,
        )
        .await
    }

    /// Sw retrieves all packages from the server, but does not install/upgrade anything.
    async fn sw(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run(
            Cmd::with_sudo(&["zypper", "install", "--download-only"])
                .kws(kws)
                .flags(flags),
            Default::default(),
            &INSTALL_STRAT,
        )
        .await
    }

    /// Sy refreshes the local package database.
    async fn sy(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.check_dry(Cmd::with_sudo(&["zypper", "refresh"]).flags(flags))
            .await?;
        if !kws.is_empty() {
            self.s(kws, flags).await?;
        }
        Ok(())
    }

    /// U upgrades or adds package(s) to the system and installs the required dependencies from sync repositories.
    async fn u(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.s(kws, flags).await
    }
}
