//! Mapping from [`pacman`] commands to various operations of specific package
//! managers.
//!
//! [`pacman`]: https://wiki.archlinux.org/index.php/Pacman

#![allow(clippy::module_name_repetitions)]

macro_rules! pm_mods {
    ( $( $vis:vis $mod:ident; )+ ) => {
        $(
            $vis mod $mod;
            paste! { pub use self::$mod::[<$mod:camel>]; }
        )+
    }
}

pm_mods! {
    apk;
    apt;
    brew;
    choco;
    conda;
    dnf;
    emerge;
    pip;
    pkcon;
    port;
    scoop;
    tlmgr;
    unknown;
    winget;
    xbps;
    zypper;
}

use std::env;

use async_trait::async_trait;
use itertools::Itertools;
use macro_rules_attribute::macro_rules_attribute;
use paste::paste;
use tt_call::tt_call;

use crate::{
    config::Config,
    error::Result,
    exec::{self, is_exe, Cmd, Mode, Output},
    print::{println_quoted, prompt},
};

/// The list of [`pacman`](https://wiki.archlinux.org/index.php/Pacman) methods supported by [`pacaptr`](crate).
#[macro_export]
#[doc(hidden)]
macro_rules! methods {
    ($caller:tt) => {
        tt_call::tt_return! {
            $caller
            methods = [{
                /// Q generates a list of installed packages.
                async fn q;

                /// Qc shows the changelog of a package.
                async fn qc;

                /// Qe lists packages installed explicitly (not as dependencies).
                async fn qe;

                /// Qi displays local package information: name, version, description, etc.
                async fn qi;

                /// Qii displays local packages which require X to be installed, aka local reverse dependencies.
                async fn qii;

                /// Qk verifies one or more packages.
                async fn qk;

                /// Ql displays files provided by local package.
                async fn ql;

                /// Qm lists packages that are installed but are not available in any installation source (anymore).
                async fn qm;

                /// Qo queries the package which provides FILE.
                async fn qo;

                /// Qp queries a package supplied through a file supplied on the command line rather than an entry in the package management database.
                async fn qp;

                /// Qs searches locally installed package for names or descriptions.
                async fn qs;

                /// Qu lists packages which have an update available.
                async fn qu;

                /// R removes a single package, leaving all of its dependencies installed.
                async fn r;

                /// Rn removes a package and skips the generation of configuration backup files.
                async fn rn;

                /// Rns removes a package and its dependencies which are not required by any other installed package,
                /// and skips the generation of configuration backup files.
                async fn rns;

                /// Rs removes a package and its dependencies which are not required by any other installed package,
                /// and not explicitly installed by the user.
                async fn rs;

                /// Rss removes a package and its dependencies which are not required by any other installed package.
                async fn rss;

                /// S installs one or more packages by name.
                async fn s;

                /// Sc removes all the cached packages that are not currently installed, and the unused sync database.
                async fn sc;

                /// Scc removes all files from the cache.
                async fn scc;

                /// Sccc performs a deeper cleaning of the cache than `Scc` (if applicable).
                async fn sccc;

                /// Sg lists all packages belonging to the GROUP.
                async fn sg;

                /// Si displays remote package information: name, version, description, etc.
                async fn si;

                /// Sii displays packages which require X to be installed, aka reverse dependencies.
                async fn sii;

                /// Sl displays a list of all packages in all installation sources that are handled by the package management.
                async fn sl;

                /// Ss searches for package(s) by searching the expression in name, description, short description.
                async fn ss;

                /// Su updates outdated packages.
                async fn su;

                /// Suy refreshes the local package database, then updates outdated packages.
                async fn suy;

                /// Sw retrieves all packages from the server, but does not install/upgrade anything.
                async fn sw;

                /// Sy refreshes the local package database.
                async fn sy;

                /// U upgrades or adds package(s) to the system and installs the required dependencies from sync repositories.
                async fn u;
            }]
        }
    };
}

macro_rules! make_op_body {
    ($self:ident, $method:ident) => {{
        Err(crate::error::Error::OperationUnimplementedError {
            op: stringify!($method).into(),
            pm: $self.name().into(),
        })
    }};
}

macro_rules! _decor_pm {(
    def = [{
        $( #[$meta0:meta] )*
        $vis:vis trait $t:ident $(: $supert:ident)? {
            $( $inner:tt )*
        }
    }]
    methods = [{ $(
        $( #[$meta1:meta] )*
        async fn $method:ident;
    )* }]
) => {
    $( #[$meta0] )*
    $vis trait $t $(: $supert)? {
        $( $inner )*

        // * Automatically generated methods below... *
        $( $( #[$meta1] )*
        async fn $method(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
            make_op_body!(self, $method)
        } )*
    }
};}

/// Send `methods!()` to `_decor_pm`, that is:
///
/// ```txt
/// _decor_pm! {
///     def = [{ trait Pm { .. } }]
///     methods = [{ q qc qe .. }] )
/// }
/// ```
macro_rules! decor_pm {
    ( $( $def:tt )* ) => {
        tt_call! {
            macro = [{ methods }]
            ~~> _decor_pm! {
                def = [{ $( $def )* }]
            }
        }
    };
}

/// The feature set of a Package Manager defined by `pacman` commands.
///
/// For method explanation see:
/// - <https://wiki.archlinux.org/index.php/Pacman>
/// - <https://wiki.archlinux.org/index.php/Pacman/Rosetta>
#[macro_rules_attribute(decor_pm!)]
#[async_trait]
pub trait Pm: Sync {
    /// Gets the name of the package manager.
    fn name(&self) -> &str;

    /// Gets the config of the package manager.
    fn cfg(&self) -> &Config;

    /// Wraps the [`Pm`] instance in a [`Box`].
    fn boxed<'a>(self) -> BoxPm<'a>
    where
        Self: Sized + Send + 'a,
    {
        Box::new(self)
    }
}

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
            if cfg.no_cache {
                if let NoCacheStrategy::WithFlags(v) = &strat.no_cache {
                    curr_cmd.flags.extend(v.clone());
                }
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
            };
        }

        Ok(res)
    }

    /// Executes a command in the context of the [`Pm`] implementation,
    /// with custom [`PmMode`] and [`Strategy`].
    async fn run_with(&self, cmd: Cmd, mode: PmMode, strat: &Strategy) -> Result<()> {
        self.check_output(cmd, mode, strat).await.map(|_| ())
    }

    /// Executes a command in the context of the [`Pm`] implementation with
    /// default settings.
    async fn run(&self, cmd: Cmd) -> Result<()> {
        self.run_with(cmd, PmMode::default(), &Strategy::default())
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
        if !self.cfg().dry_run {
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
///
/// Default value: [`PmMode::CheckErr`].
#[derive(Copy, Clone, Debug, Default)]
pub enum PmMode {
    /// Silently collects all the `stdout`/`stderr` combined. Prints nothing.
    Mute,

    /// Prints out the command which should be executed, run it and collect its
    /// `stdout`/`stderr` combined. Potentially dangerous as it destroys the
    /// colored `stdout`. Use it only if really necessary.
    #[allow(dead_code)]
    CheckAll,

    /// Prints out the command which should be executed, run it and collect its
    /// `stderr`. This will work with a colored `stdout`.
    #[default]
    CheckErr,
}

impl From<PmMode> for Mode {
    fn from(pm_mode: PmMode) -> Self {
        match pm_mode {
            PmMode::Mute => Self::Mute,
            PmMode::CheckAll => Self::CheckAll,
            PmMode::CheckErr => Self::CheckErr,
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

#[allow(missing_docs)]
#[cfg(feature = "test")]
pub mod tests {
    use async_trait::async_trait;
    use tt_call::tt_call;

    use super::*;
    use crate::config::Config;

    #[derive(Debug)]
    pub struct MockPm {
        pub cfg: Config,
    }

    macro_rules! make_mock_op_body {
        ($self:ident, $kws:ident, $flags:ident, $method:ident) => {{
            let kws: Vec<_> = itertools::chain!($kws, $flags).collect();
            panic!("should run: {} {:?}", stringify!($method), &kws)
        }};
    }

    macro_rules! impl_pm_mock {(
        methods = [{ $(
            $( #[$meta:meta] )*
            async fn $method:ident;
        )* }]
    ) => {
        #[async_trait]
        impl Pm for MockPm {
            /// Gets the name of the package manager.
            fn name(&self) -> &str {
                "mockpm"
            }

            fn cfg(&self) -> &Config {
                &self.cfg
            }

            // * Automatically generated methods below... *
            $( async fn $method(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
                    make_mock_op_body!(self, kws, flags, $method)
            } )*
        }
    };}

    tt_call! {
        macro = [{ methods }]
        ~~> impl_pm_mock
    }
}
