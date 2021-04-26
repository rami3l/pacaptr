use super::{NoCacheStrategy, Pm, PmHelper, PmMode, PromptStrategy, Strategies};
use crate::{
    dispatch::config::Config,
    error::Result,
    exec::{self, Cmd},
    print::{self, PROMPT_RUN},
};
use async_trait::async_trait;
use futures::prelude::*;
use once_cell::sync::Lazy;

pub struct Dnf {
    pub cfg: Config,
}

static PROMPT_STRAT: Lazy<Strategies> = Lazy::new(|| Strategies {
    prompt: PromptStrategy::native_prompt(&["-y"]),
    ..Default::default()
});

static INSTALL_STRAT: Lazy<Strategies> = Lazy::new(|| Strategies {
    prompt: PromptStrategy::native_prompt(&["-y"]),
    no_cache: NoCacheStrategy::Sccc,
    ..Default::default()
});

#[async_trait]
impl Pm for Dnf {
    /// Gets the name of the package manager.
    fn name(&self) -> String {
        "dnf".into()
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

    /// Qe lists packages installed explicitly (not as dependencies).
    async fn qe(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run_default(
            Cmd::new(&["dnf", "repoquery", "--userinstalled"])
                .kws(kws)
                .flags(flags),
        )
        .await
    }

    /// Qi displays local package information: name, version, description, etc.
    async fn qi(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        let cmds: &[&[&str]] = &[
            &["dnf", "info", "--installed"],
            &["dnf", "repoquery", "--deplist"],
        ];
        stream::iter(cmds)
            .map(Ok)
            .try_for_each(|&cmd| self.just_run_default(Cmd::new(cmd).kws(kws).flags(flags)))
            .await
    }

    /// Ql displays files provided by local package.
    async fn ql(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run_default(Cmd::new(&["rpm", "-ql"]).kws(kws).flags(flags))
            .await
    }

    /// Qm lists packages that are installed but are not available in any installation source (anymore).
    async fn qm(&self, _kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run_default(Cmd::new(&["dnf", "list", "extras"]).flags(flags))
            .await
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
    // TODO: Is this right?
    async fn qs(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        let cmd = Cmd::new(&["rpm", "-qa"]).flags(flags);
        if !self.cfg.dry_run {
            print::print_cmd(&cmd, PROMPT_RUN);
        }
        let out_bytes = self
            .run(cmd, PmMode::Mute, &Default::default())
            .await?
            .contents;
        exec::grep_print(&String::from_utf8(out_bytes)?, kws)?;
        Ok(())
    }

    /// Qu lists packages which have an update available.
    async fn qu(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run_default(Cmd::new(&["dnf", "list", "updates"]).kws(kws).flags(flags))
            .await
    }

    /// R removes a single package, leaving all of its dependencies installed.
    async fn r(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run(
            Cmd::with_sudo(&["dnf", "remove"]).kws(kws).flags(flags),
            Default::default(),
            &PROMPT_STRAT,
        )
        .await
    }

    /// S installs one or more packages by name.
    async fn s(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run(
            Cmd::with_sudo(&["dnf", "install"]).kws(kws).flags(flags),
            Default::default(),
            &INSTALL_STRAT,
        )
        .await
    }

    /// Sc removes all the cached packages that are not currently installed, and the unused sync database.
    async fn sc(&self, _kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run(
            Cmd::new(&["dnf", "clean", "expire-cache"]).flags(flags),
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
        self.just_run(
            Cmd::new(&["dnf", "clean", "packages"]).flags(flags),
            Default::default(),
            &Strategies {
                prompt: PromptStrategy::CustomPrompt,
                ..Default::default()
            },
        )
        .await
    }

    /// Sccc ...
    /// What is this?
    async fn sccc(&self, _kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run(
            Cmd::new(&["dnf", "clean", "all"]).flags(flags),
            Default::default(),
            &Strategies {
                prompt: PromptStrategy::CustomPrompt,
                ..Default::default()
            },
        )
        .await
    }

    /// Si displays remote package information: name, version, description, etc.
    async fn si(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        let cmds: &[&[&str]] = &[&["dnf", "info"], &["dnf", "repoquery", "--deplist"]];
        stream::iter(cmds)
            .map(Ok)
            .try_for_each(|&cmd| self.just_run_default(Cmd::new(cmd).kws(kws).flags(flags)))
            .await
    }

    /// Sg lists all packages belonging to the GROUP.
    async fn sg(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run_default(
            Cmd::new(if kws.is_empty() {
                &["dnf", "group", "list"]
            } else {
                &["dnf", "group", "info"]
            })
            .kws(kws)
            .flags(flags),
        )
        .await
    }

    /// Sl displays a list of all packages in all installation sources that are handled by the packages management.
    async fn sl(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run_default(
            Cmd::new(&["dnf", "list", "available"])
                .kws(kws)
                .flags(flags),
        )
        .await
    }

    /// Ss searches for package(s) by searching the expression in name, description, short description.
    async fn ss(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run_default(Cmd::new(&["dnf", "search"]).kws(kws).flags(flags))
            .await
    }

    /// Su updates outdated packages.
    async fn su(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run(
            Cmd::with_sudo(&["dnf", "upgrade"]).kws(kws).flags(flags),
            Default::default(),
            &INSTALL_STRAT,
        )
        .await
    }

    /// Suy refreshes the local package database, then updates outdated packages.
    async fn suy(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.su(kws, flags).await
    }

    /// Sw retrieves all packages from the server, but does not install/upgrade anything.
    async fn sw(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run(
            Cmd::with_sudo(&["dnf", "install", "--downloadonly"])
                .kws(kws)
                .flags(flags),
            Default::default(),
            &INSTALL_STRAT,
        )
        .await
    }

    /// Sy refreshes the local package database.
    async fn sy(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.sc(&[], flags).await?;
        self.just_run_default(Cmd::new(&["dnf", "check-update"]).flags(flags))
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
