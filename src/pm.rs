//! Mapping from [`pacman`] commands to various operations of specific package
//! managers.
//!
//! [`pacman`]: https://wiki.archlinux.org/index.php/Pacman

#![allow(clippy::module_name_repetitions)]

macro_rules! pm_mods {
    ( $( $vis:vis $mod:ident; )+ ) => {
        $(
            $vis mod $mod;
            paste! { pub(crate) use self::$mod::[<$mod:camel>]; }
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
    port;
    scoop;
    tlmgr;
    unknown;
    xbps;
    zypper;
}

use async_trait::async_trait;
use itertools::Itertools;
use macro_rules_attribute::macro_rules_attribute;
use paste::paste;
use tt_call::tt_call;

use crate::{
    dispatch::Config,
    error::Result,
    exec::{Cmd, Mode, Output},
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

                /// Sccc ...
                /// What is this?
                async fn sccc;

                /// Sg lists all packages belonging to the GROUP.
                async fn sg;

                /// Si displays remote package information: name, version, description, etc.
                async fn si;

                /// Sii displays packages which require X to be installed, aka reverse dependencies.
                async fn sii;

                /// Sl displays a list of all packages in all installation sources that are handled by the packages management.
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
        $vis:vis trait $t:ident : $supert:ident {
            $( $inner:tt )*
        }
    }]
    methods = [{ $(
        $( #[$meta1:meta] )*
        async fn $method:ident;
    )* }]
) => {
    $( #[$meta0] )*
    $vis trait $t : $supert {
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
/// ```rust
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
pub(crate) trait Pm: Sync {
    /// Gets the name of the package manager.
    fn name(&self) -> &str;

    /// Gets the config of the package manager.
    fn cfg(&self) -> &Config;

    /// Wraps the [`Pm`] instance in a [`Box`].
    fn boxed<'a>(self) -> Box<dyn Pm + 'a>
    where
        Self: Sized + 'a,
    {
        Box::new(self)
    }
}

/// Extra implementation helper functions for [`Pm`],
/// focusing on the ability to run commands ([`Cmd`]s) in a configured and
/// [`Pm`]-specific context.
#[async_trait]
trait PmHelper: Pm {
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
            let flags = cmd.flags.iter().map(|s| s as _).collect_vec();
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
}

impl<P: Pm> PmHelper for P {}

/// Different ways in which a command shall be dealt with.
/// This is a [`Pm`] specified version intended to be used along with
/// [`Strategy`].
#[derive(Copy, Clone, Debug)]
enum PmMode {
    /// Silently collects all the `stdout`/`stderr` combined. Print nothing.
    Mute,

    /// Prints out the command which should be executed, run it and collect its
    /// `stdout`/`stderr` combined. Potentially dangerous as it destroys the
    /// colored `stdout`. Use it only if really necessary.
    CheckAll,

    /// Prints out the command which should be executed, run it and collect its
    /// `stderr`. This will work with a colored `stdout`.
    CheckErr,
}

impl Default for PmMode {
    fn default() -> Self {
        PmMode::CheckErr
    }
}

impl From<PmMode> for Mode {
    fn from(pm_mode: PmMode) -> Self {
        match pm_mode {
            PmMode::Mute => Mode::Mute,
            PmMode::CheckAll => Mode::CheckAll,
            PmMode::CheckErr => Mode::CheckErr,
        }
    }
}

/// A set of intrinsic properties of a command in the context of a specific
/// package manager, indicating how it is run.
#[derive(Clone, Debug, Default)]
struct Strategy {
    /// How a dry run is dealt with.
    dry_run: DryRunStrategy,

    /// How the prompt is dealt with when running the package manager.
    prompt: PromptStrategy,

    /// How the cache is cleaned when `no_cache` is set to `true`.
    no_cache: NoCacheStrategy,
}

/// How a dry run is dealt with.
#[derive(Debug, Clone)]
enum DryRunStrategy {
    /// Prints the command to be run, and stop.
    PrintCmd,
    /// Invokes the corresponding package manager with the flags given.
    WithFlags(Vec<String>),
}

impl DryRunStrategy {
    /// Invokes the corresponding package manager with the flags given.
    #[must_use]
    fn with_flags(flags: &[impl AsRef<str>]) -> Self {
        Self::WithFlags(flags.iter().map(|s| s.as_ref().into()).collect())
    }
}

impl Default for DryRunStrategy {
    fn default() -> Self {
        DryRunStrategy::PrintCmd
    }
}

/// How the prompt is dealt with when running the package manager.
#[derive(Debug, Clone)]
enum PromptStrategy {
    /// There is no prompt.
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
    #[must_use]
    fn native_no_confirm(no_confirm: &[impl AsRef<str>]) -> Self {
        Self::NativeNoConfirm(no_confirm.iter().map(|s| s.as_ref().into()).collect())
    }

    #[must_use]
    /// There is a native prompt provided by the package manager
    /// that can be enabled with a flag.
    fn native_confirm(confirm: &[impl AsRef<str>]) -> Self {
        Self::NativeConfirm(confirm.iter().map(|s| s.as_ref().into()).collect())
    }
}

impl Default for PromptStrategy {
    fn default() -> Self {
        PromptStrategy::None
    }
}

/// How the cache is cleaned when `no_cache` is set to `true`.
#[derive(Debug, Clone)]
enum NoCacheStrategy {
    /// Does not clean cache.
    /// This variant MUST be used when implementing cache cleaning methods like
    /// `-Sc`.
    None,
    /// Uses `-Sc` to clean the cache.
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
    #[must_use]
    fn with_flags(flags: &[impl AsRef<str>]) -> Self {
        Self::WithFlags(flags.iter().map(|s| s.as_ref().into()).collect())
    }
}

impl Default for NoCacheStrategy {
    fn default() -> Self {
        NoCacheStrategy::None
    }
}
