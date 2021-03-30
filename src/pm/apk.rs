use super::{NoCacheStrategy, Pm, PmHelper, PmMode, PromptStrategy, Strategies};
use crate::{
    dispatch::config::Config,
    error::Result,
    exec::{self, Cmd},
    print::{self, PROMPT_RUN},
};
use async_trait::async_trait;
use once_cell::sync::Lazy;

pub struct Apk {
    pub cfg: Config,
}

static PROMPT_STRAT: Lazy<Strategies> = Lazy::new(|| Strategies {
    prompt: PromptStrategy::CustomPrompt,
    ..Default::default()
});

static INSTALL_STRAT: Lazy<Strategies> = Lazy::new(|| Strategies {
    prompt: PromptStrategy::CustomPrompt,
    no_cache: NoCacheStrategy::with_flags(&["--no-cache"]),
    ..Default::default()
});

#[async_trait]
impl Pm for Apk {
    /// Gets the name of the package manager.
    fn name(&self) -> String {
        "apk".into()
    }

    fn cfg(&self) -> &Config {
        &self.cfg
    }

    /// Q generates a list of installed packages.
    async fn q(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        if kws.is_empty() {
            self.just_run_default(Cmd::new(&["apk", "info"]).flags(flags))
                .await
        } else {
            self.qs(kws, flags).await
        }
    }

    /// Qi displays local package information: name, version, description, etc.
    async fn qi(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.si(kws, flags).await
    }

    /// Ql displays files provided by local package.
    async fn ql(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run_default(Cmd::new(&["apk", "info", "-L"]).kws(kws).flags(flags))
            .await
    }

    /// Qo queries the package which provides FILE.
    async fn qo(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run_default(
            Cmd::new(&["apk", "info", "--who-owns"])
                .kws(kws)
                .flags(flags),
        )
        .await
    }

    /// Qs searches locally installed package for names or descriptions.
    // According to https://www.archlinux.org/pacman/pacman.8.html#_query_options_apply_to_em_q_em_a_id_qo_a,
    // when including multiple search terms, only packages with descriptions matching ALL of those terms are returned.
    async fn qs(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        let cmd = &["apk", "info", "-d"];
        let cmd = Cmd::new(cmd).flags(flags);
        if !self.cfg.dry_run {
            print::print_cmd(&cmd, PROMPT_RUN);
        }
        let out_bytes = self
            .run(cmd, PmMode::Mute, &Default::default())
            .await?
            .contents;
        exec::grep_print(&String::from_utf8(out_bytes)?, kws)
    }

    /// Qu lists packages which have an update available.
    //? Is that the right way to input '<'?
    async fn qu(&self, _kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run_default(Cmd::new(&["apk", "version", "-l", "<"]).flags(flags))
            .await
    }

    /// R removes a single package, leaving all of its dependencies installed.
    async fn r(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run(
            Cmd::with_sudo(&["apk", "del"]).kws(kws).flags(flags),
            Default::default(),
            &PROMPT_STRAT,
        )
        .await
    }

    /// Rn removes a package and skips the generation of configuration backup files.
    async fn rn(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run(
            Cmd::with_sudo(&["apk", "del", "--purge"])
                .kws(kws)
                .flags(flags),
            Default::default(),
            &PROMPT_STRAT,
        )
        .await
    }

    /// Rns removes a package and its dependencies which are not required by any other installed package, and skips the generation of configuration backup files.
    async fn rns(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run(
            Cmd::with_sudo(&["apk", "del", "--purge", "-r"])
                .kws(kws)
                .flags(flags),
            Default::default(),
            &PROMPT_STRAT,
        )
        .await
    }

    /// Rs removes a package and its dependencies which are not required by any other installed package,
    /// and not explicitly installed by the user.
    async fn rs(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.r(kws, flags).await
    }

    /// S installs one or more packages by name.
    async fn s(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run(
            Cmd::with_sudo(&["apk", "add"]).kws(kws).flags(flags),
            Default::default(),
            &INSTALL_STRAT,
        )
        .await
    }

    /// Sc removes all the cached packages that are not currently installed, and the unused sync database.
    async fn sc(&self, _kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run(
            Cmd::with_sudo(&["apk", "cache", "-v", "clean"]).flags(flags),
            Default::default(),
            &PROMPT_STRAT,
        )
        .await
    }

    /// Scc removes all files from the cache.
    async fn scc(&self, _kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run(
            Cmd::with_sudo(&["rm", "-vrf", "/var/cache/apk/*"]).flags(flags),
            Default::default(),
            &PROMPT_STRAT,
        )
        .await
    }

    /// Si displays remote package information: name, version, description, etc.
    async fn si(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run_default(Cmd::new(&["apk", "info", "-a"]).kws(kws).flags(flags))
            .await
    }

    /// Sii displays packages which require X to be installed, aka reverse dependencies.
    async fn sii(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run_default(Cmd::new(&["apk", "info", "-r"]).kws(kws).flags(flags))
            .await
    }

    /// Sl displays a list of all packages in all installation sources that are handled by the packages management.
    async fn sl(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run_default(Cmd::new(&["apk", "search"]).kws(kws).flags(flags))
            .await
    }

    /// Ss searches for package(s) by searching the expression in name, description, short description.
    async fn ss(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run_default(Cmd::new(&["apk", "search", "-v"]).kws(kws).flags(flags))
            .await
    }

    /// Su updates outdated packages.
    async fn su(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        let cmd = if kws.is_empty() {
            Cmd::with_sudo(&["apk", "upgrade"]).kws(kws).flags(flags)
        } else {
            Cmd::with_sudo(&["apk", "add", "-u"]).kws(kws).flags(flags)
        };
        self.just_run(cmd, Default::default(), &INSTALL_STRAT).await
    }

    /// Suy refreshes the local package database, then updates outdated packages.
    async fn suy(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        let cmd = if kws.is_empty() {
            Cmd::with_sudo(&["apk", "upgrade", "-U", "-a"])
                .kws(kws)
                .flags(flags)
        } else {
            Cmd::with_sudo(&["apk", "add", "-U", "-u"])
                .kws(kws)
                .flags(flags)
        };
        self.just_run(cmd, Default::default(), &INSTALL_STRAT).await
    }

    /// Sw retrieves all packages from the server, but does not install/upgrade anything.
    async fn sw(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run(
            Cmd::new(&["apk", "fetch"]).kws(kws).flags(flags),
            Default::default(),
            &PROMPT_STRAT,
        )
        .await
    }

    /// Sy refreshes the local package database.
    async fn sy(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run_default(Cmd::with_sudo(&["apk", "update"]).kws(kws).flags(flags))
            .await?;
        if !kws.is_empty() {
            self.s(kws, flags).await?;
        }
        Ok(())
    }

    /// U upgrades or adds package(s) to the system and installs the required dependencies from sync repositories.
    async fn u(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.just_run(
            Cmd::with_sudo(&["apk", "add", "--allow-untrusted"])
                .kws(kws)
                .flags(flags),
            Default::default(),
            &INSTALL_STRAT,
        )
        .await
    }
}
