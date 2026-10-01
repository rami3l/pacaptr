//! Mapping from [`pacman`] commands to various operations of specific package
//! managers.
//!
//! [`pacman`]: https://wiki.archlinux.org/index.php/Pacman

#![allow(clippy::module_name_repetitions)]

mod apk;
mod apt;
mod brew;
mod choco;
mod conda;
mod dnf;
mod emerge;
mod imp;
mod pip;
mod pkcon;
mod port;
mod scoop;
mod tlmgr;
mod unknown;
mod winget;
mod xbps;
mod zypper;

use std::env;

use async_trait::async_trait;
use itertools::Itertools;

pub use self::imp::Pm;
use self::{
    apk::Apk, apt::Apt, brew::Brew, choco::Choco, conda::Conda, dnf::Dnf, emerge::Emerge, pip::Pip,
    pkcon::Pkcon, port::Port, scoop::Scoop, tlmgr::Tlmgr, unknown::Unknown, winget::Winget,
    xbps::Xbps, zypper::Zypper,
};
use crate::{
    config::Config,
    error::Result,
    exec::{self, Cmd, Mode, Output, is_exe},
    print::{println_quoted, prompt},
};

/// An owned, dynamically typed [`Pm`].
pub type BoxPm<'a> = Box<dyn Pm + Send + 'a>;

impl From<Config> for BoxPm<'_> {
    /// Generates the `Pm` instance according it's name, feeding it with the
    /// current `Config`.
    fn from(mut cfg: Config) -> Self {
        // If the `Pm` to be used is not stated in any config,
        // we should fall back to automatic detection and overwrite `cfg`.
        let pm = cfg.default_pm.get_or_insert_with(|| detect_pm_str().into());

        #[allow(clippy::match_single_binding)]
        match pm.as_ref() {
            // Chocolatey
            "choco" => Choco::new(cfg).boxed(),

            // Scoop
            "scoop" => Scoop::new(cfg).boxed(),

            // Winget
            "winget" => Winget::new(cfg).boxed(),

            // Homebrew/Linuxbrew
            "brew" => Brew::new(cfg).boxed(),

            // Macports
            "port" if cfg!(target_os = "macos") => Port::new(cfg).boxed(),

            // Apt for Debian/Ubuntu/Termux (newer versions)
            "apt" | "pkg" => Apt::new(cfg).boxed(),

            // Apk for Alpine
            "apk" => Apk::new(cfg).boxed(),

            // Dnf for RedHat
            "dnf" => Dnf::new(cfg).boxed(),

            // Portage for Gentoo
            "emerge" => Emerge::new(cfg).boxed(),

            // Xbps for Void Linux
            "xbps" | "xbps-install" => Xbps::new(cfg).boxed(),

            // Zypper for SUSE
            "zypper" => Zypper::new(cfg).boxed(),

            // -- External Package Managers --

            // Conda
            "conda" => Conda::new(cfg).boxed(),

            // Pip
            "pip" | "pip3" => Pip::new(cfg).boxed(),

            // PackageKit
            "pkcon" => Pkcon::new(cfg).boxed(),

            // Tlmgr
            "tlmgr" => Tlmgr::new(cfg).boxed(),

            // Test-only mock package manager
            #[cfg(feature = "test")]
            "mockpm" => {
                use self::tests::MockPm;
                MockPm { cfg }.boxed()
            }

            // Unknown package manager X
            x => Unknown::new(x).boxed(),
        }
    }
}

/// Detects the name of the package manager to be used in auto dispatch.
#[must_use]
fn detect_pm_str() -> &'static str {
    /// Check if one of the following conditions are met:
    /// - `$TERMUX_APP_PACKAGE_MANAGER` is `apt`;
    /// - `$TERMUX_MAIN_PACKAGE_FORMAT` is `debian`.
    ///
    /// See: <https://github.com/rami3l/pacaptr/issues/576#issuecomment-1565122604>
    fn is_termux_apt() -> bool {
        env::var("TERMUX_APP_PACKAGE_MANAGER").as_deref() == Ok("apt")
            || env::var("TERMUX_MAIN_PACKAGE_FORMAT").as_deref() == Ok("debian")
    }

    let pairs: &[(&str, &str)] = match () {
        () if cfg!(windows) => &[("scoop", ""), ("choco", ""), ("winget", "")],

        () if cfg!(target_os = "macos") => &[
            ("brew", "/usr/local/bin/brew"),
            ("port", "/opt/local/bin/port"),
            ("apt", "/opt/procursus/bin/apt"),
        ],

        () if cfg!(target_os = "ios") => &[("apt", "/usr/bin/apt")],

        () if cfg!(target_os = "linux") => &[
            ("apk", "/sbin/apk"),
            ("apt", "/usr/bin/apt"),
            ("dnf", "/usr/bin/dnf"),
            ("emerge", "/usr/bin/emerge"),
            ("xbps-install", "/usr/bin/xbps-install"),
            ("zypper", "/usr/bin/zypper"),
        ],

        () => &[],
    };

    pairs
        .iter()
        .find_map(|&(name, path)| is_exe(name, path).then_some(name))
        .map_or("unknown", |name| {
            if name == "apt" && is_termux_apt() {
                return "pkg";
            }
            name
        })
}

/// Extra implementation helper functions for [`Pm`],
/// focusing on the ability to run commands ([`Cmd`]s) in a configured and
/// [`Pm`]-specific context.
#[async_trait]
pub trait PmHelper: Pm {
    /// Executes a command in the context of the [`Pm`] implementation. Returns
    /// the [`Output`] of this command.
    async fn check_output(&self, mut cmd: Cmd, mode: PmMode, strat: &Strategy) -> Result<Output> {
        async fn run(cfg: &Config, cmd: &Cmd, mode: PmMode, strat: &Strategy) -> Result<Output> {
            let mut curr_cmd = cmd.clone();
            let no_confirm = cfg.no_confirm;
            if cfg.no_cache
                && let NoCacheStrategy::WithFlags(v) = &strat.no_cache
            {
                curr_cmd.flags.extend(v.clone());
            }
            match &strat.prompt {
                PromptStrategy::None => curr_cmd.exec(mode.into()).await,
                PromptStrategy::CustomPrompt if no_confirm => curr_cmd.exec(mode.into()).await,
                PromptStrategy::CustomPrompt => curr_cmd.exec(Mode::Prompt).await,
                PromptStrategy::NativeNoConfirm(v) => {
                    if no_confirm {
                        curr_cmd.flags.extend(v.clone());
                    }
                    curr_cmd.exec(mode.into()).await
                }
                PromptStrategy::NativeConfirm(v) => {
                    if !no_confirm {
                        curr_cmd.flags.extend(v.clone());
                    }
                    curr_cmd.exec(mode.into()).await
                }
            }
        }

        let cfg = self.cfg();

        // `--dry-run` should apply to both the main command and the cleanup.
        let res = match &strat.dry_run {
            DryRunStrategy::PrintCmd if cfg.dry_run => cmd.clone().exec(Mode::PrintCmd).await?,
            DryRunStrategy::WithFlags(v) if cfg.dry_run => {
                cmd.flags.extend(v.clone());
                // -- A dry run with extra flags does not need `sudo`. --
                cmd = cmd.sudo(false);
                run(cfg, &cmd, mode, strat).await?
            }
            _ => run(cfg, &cmd, mode, strat).await?,
        };

        // Perform the cleanup.
        if cfg.no_cache {
            let flags = cmd.flags.iter().map(AsRef::as_ref).collect_vec();
            match &strat.no_cache {
                NoCacheStrategy::Sc => self.sc(&[], &flags).await?,
                NoCacheStrategy::Scc => self.scc(&[], &flags).await?,
                NoCacheStrategy::Sccc => self.sccc(&[], &flags).await?,
                _ => (),
            }
        }

        Ok(res)
    }

    /// Returns the default [`PmMode`] for this [`Pm`].
    fn default_mode(&self) -> PmMode {
        let quiet = self.cfg().quiet();
        PmMode::CheckErr { quiet }
    }

    /// Executes a command in the context of the [`Pm`] implementation,
    /// with custom [`PmMode`] and [`Strategy`].
    async fn run_with(&self, cmd: Cmd, mode: PmMode, strat: &Strategy) -> Result<()> {
        self.check_output(cmd, mode, strat).await.map(|_| ())
    }

    /// Executes a command in the context of the [`Pm`] implementation with
    /// default settings.
    async fn run(&self, cmd: Cmd) -> Result<()> {
        self.run_with(cmd, self.default_mode(), &Strategy::default())
            .await
    }

    /// Executes a command in [`PmMode::Mute`] and prints the output lines
    /// that match against the given regex `patterns`.
    async fn search_regex(&self, cmd: Cmd, patterns: &[&str]) -> Result<()> {
        self.search_regex_with_header(cmd, patterns, 0).await
    }

    /// Executes a command in [`PmMode::Mute`] and prints `header_lines` of
    /// header followed by the output lines that match against the given regex
    /// `patterns`.
    /// If `header_lines >= text.lines().count()`, then the
    /// output lines are printed without changes.
    async fn search_regex_with_header(
        &self,
        cmd: Cmd,
        patterns: &[&str],
        header_lines: usize,
    ) -> Result<()> {
        let cfg = self.cfg();
        if !(cfg.dry_run || cfg.quiet()) {
            println_quoted(&*prompt::RUNNING, &cmd);
        }
        let out_bytes = self
            .check_output(cmd, PmMode::Mute, &Strategy::default())
            .await?;
        exec::grep_print_with_header(&String::from_utf8(out_bytes)?, patterns, header_lines)
    }
}

impl<P: Pm> PmHelper for P {}

/// Different ways in which a command shall be dealt with.
///
/// This is a [`Pm`] specified version intended to be used along with
/// [`Strategy`].
#[derive(Copy, Clone, Debug)]
pub enum PmMode {
    /// Silently collects all the `stdout`/`stderr` combined. Prints nothing.
    Mute,

    /// Prints out the command which should be executed, runs it and collects
    /// its `stdout`/`stderr` combined.
    ///
    /// This is potentially dangerous as it destroys the colored `stdout`. Use
    /// it only if really necessary.
    #[allow(dead_code)]
    CheckAll {
        /// Whether the log output should be suppressed.
        quiet: bool,
    },

    /// Prints out the command which should be executed, runs it and collects
    /// its `stderr`.
    ///
    /// This will work with a colored `stdout`.
    CheckErr {
        /// Whether the log output should be suppressed.
        quiet: bool,
    },
}

impl From<PmMode> for Mode {
    fn from(pm_mode: PmMode) -> Self {
        match pm_mode {
            PmMode::Mute => Self::Mute,
            PmMode::CheckAll { quiet } => Self::CheckAll { quiet },
            PmMode::CheckErr { quiet } => Self::CheckErr { quiet },
        }
    }
}

/// A set of intrinsic properties of a command in the context of a specific
/// package manager, indicating how it is run.
#[derive(Clone, Debug, Default)]
#[must_use]
pub struct Strategy {
    /// How a dry run is dealt with.
    dry_run: DryRunStrategy,

    /// How the prompt is dealt with when running the package manager.
    prompt: PromptStrategy,

    /// How the cache is cleaned when `no_cache` is set to `true`.
    no_cache: NoCacheStrategy,
}

/// How a dry run is dealt with.
///
/// Default value: [`DryRunStrategy::PrintCmd`].
#[must_use]
#[derive(Debug, Clone, Default)]
pub enum DryRunStrategy {
    /// Prints the command to be run, and stop.
    #[default]
    PrintCmd,
    /// Invokes the corresponding package manager with the flags given.
    WithFlags(Vec<String>),
}

impl DryRunStrategy {
    /// Invokes the corresponding package manager with the flags given.
    pub fn with_flags(flags: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        Self::WithFlags(flags.into_iter().map(|s| s.as_ref().into()).collect())
    }
}

/// How the prompt is dealt with when running the package manager.
///
/// Default value: [`PromptStrategy::None`].
#[must_use]
#[derive(Debug, Clone, Default)]
pub enum PromptStrategy {
    /// There is no prompt.
    #[default]
    None,
    /// There is no prompt, but a custom prompt is added.
    CustomPrompt,
    /// There is a native prompt provided by the package manager
    /// that can be disabled with a flag.
    NativeNoConfirm(Vec<String>),
    /// There is a native prompt provided by the package manager
    /// that can be enabled with a flag.
    NativeConfirm(Vec<String>),
}

impl PromptStrategy {
    /// There is a native prompt provided by the package manager
    /// that can be disabled with a flag.
    pub fn native_no_confirm(no_confirm: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        Self::NativeNoConfirm(no_confirm.into_iter().map(|s| s.as_ref().into()).collect())
    }

    /// There is a native prompt provided by the package manager
    /// that can be enabled with a flag.
    pub fn native_confirm(confirm: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        Self::NativeConfirm(confirm.into_iter().map(|s| s.as_ref().into()).collect())
    }
}

/// How the cache is cleaned when `no_cache` is set to `true`.
///
/// Default value: [`PromptStrategy::None`].
#[must_use]
#[derive(Debug, Clone, Default)]
pub enum NoCacheStrategy {
    /// Does not clean cache.
    /// This variant MUST be used when implementing cache cleaning methods like
    /// `-Sc`.
    #[default]
    None,
    /// Uses `-Sc` to clean the cache.
    #[allow(dead_code)]
    Sc,
    /// Uses `-Scc`.
    Scc,
    /// Uses `-Sccc`.
    Sccc,
    /// Invokes the corresponding package manager with the flags given.
    WithFlags(Vec<String>),
}

impl NoCacheStrategy {
    /// Invokes the corresponding package manager with the flags given.
    pub fn with_flags(flags: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        Self::WithFlags(flags.into_iter().map(|s| s.as_ref().into()).collect())
    }
}

#[cfg(feature = "test")]
pub mod tests;
