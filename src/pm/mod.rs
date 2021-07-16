//! Mapping from [`pacman`] commands to various operations of specific package
//! managers.
//!
//! [`pacman`]: https://wiki.archlinux.org/index.php/Pacman

pub mod apk;
pub mod apt;
pub mod brew;
pub mod choco;
pub mod conda;
pub mod dnf;
pub mod emerge;
pub mod pip;
pub mod port;
pub mod scoop;
pub mod tlmgr;
pub mod unknown;
pub mod zypper;

use async_trait::async_trait;
use macro_rules_attribute::macro_rules_attribute;
use once_cell::sync::Lazy;
use tokio::sync::Mutex;
use tt_call::tt_call;

pub use self::{
    apk::Apk, apt::Apt, brew::Brew, choco::Choco, conda::Conda, dnf::Dnf, emerge::Emerge, pip::Pip,
    port::Port, scoop::Scoop, tlmgr::Tlmgr, unknown::Unknown, zypper::Zypper,
};
use crate::{
    dispatch::Config,
    error::Result,
    exec::{Cmd, Mode, Output, StatusCode},
};

/// The list of `pacman` methods supported by `pacaptr`.
#[macro_export]
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
///     def = [{ pub trait Pm { .. } }]
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
    fn boxed<'a>(self) -> Box<dyn Pm + 'a>
    where
        Self: Sized + 'a,
    {
        Box::new(self)
    }

    /// Gets the [`StatusCode`] to be returned.
    async fn code(&self) -> StatusCode {
        self.get_set_code(None).await
    }

    /// Sets the [`StatusCode`] to be returned.
    async fn set_code(&self, to: StatusCode) {
        self.get_set_code(Some(to)).await;
    }

    /// Gets/Sets the [`StatusCode`] to be returned.
    ///
    /// If `to` is `Some(n)`, the current [`StatusCode`] will be reset to `n`,
    /// then return [`StatusCode`].
    #[doc(hidden)]
    async fn get_set_code(&self, to: Option<StatusCode>) -> StatusCode {
        static CODE: Lazy<Mutex<StatusCode>> = Lazy::new(|| Mutex::new(0));
        let mut code = CODE.lock().await;
        if let Some(n) = to {
            *code = n;
        }
        *code
    }
}

/// Extra implementation helper functions for [`Pm`],
/// focusing on the ability to run commands ([`Cmd`]s) in a configured and
/// [`Pm`]-specific context.
#[async_trait]
pub trait PmHelper: Pm {
    /// Executes a command in the context of the [`Pm`] implementation. Returns
    /// the [`Output`] of this command.
    async fn check_output(&self, mut cmd: Cmd, mode: PmMode, strat: &Strategy) -> Result<Output> {
        let cfg = self.cfg();

        async fn run(cfg: &Config, cmd: &Cmd, mode: PmMode, strat: &Strategy) -> Result<Output> {
            let mut curr_cmd = cmd.clone();
            let no_confirm = cfg.no_confirm;
            if cfg.no_cache {
                if let NoCacheStrategy::WithFlags(v) = &strat.no_cache {
                    curr_cmd.flags.extend(v.to_owned());
                }
            }
            match &strat.prompt {
                PromptStrategy::None => curr_cmd.exec(mode.into()).await,
                PromptStrategy::CustomPrompt if no_confirm => curr_cmd.exec(mode.into()).await,
                PromptStrategy::CustomPrompt => curr_cmd.exec(Mode::Prompt).await,
                PromptStrategy::NativeNoConfirm(v) => {
                    if no_confirm {
                        curr_cmd.flags.extend(v.to_owned());
                    }
                    curr_cmd.exec(mode.into()).await
                }
                PromptStrategy::NativeConfirm(v) => {
                    if !no_confirm {
                        curr_cmd.flags.extend(v.to_owned());
                    }
                    curr_cmd.exec(mode.into()).await
                }
            }
        }

        // `--dry-run` should apply to both the main command and the cleanup.
        let res = match &strat.dry_run {
            DryRunStrategy::PrintCmd if cfg.dry_run => cmd.clone().exec(Mode::PrintCmd).await?,
            DryRunStrategy::WithFlags(v) if cfg.dry_run => {
                cmd.flags.extend(v.to_owned());
                // -- A dry run with extra flags does not need `sudo`. --
                cmd = cmd.sudo(false);
                run(cfg, &cmd, mode, strat).await?
            }
            _ => run(cfg, &cmd, mode, strat).await?,
        };

        // Perform the cleanup.
        if cfg.no_cache {
            let flags = cmd.flags.iter().map(|s| s.as_ref()).collect::<Vec<_>>();
            match &strat.no_cache {
                NoCacheStrategy::Sc => self.sc(&[], &flags).await?,
                NoCacheStrategy::Scc => self.scc(&[], &flags).await?,
                NoCacheStrategy::Sccc => self.sccc(&[], &flags).await?,
                _ => (),
            };
        }

        // Reset the current status code.
        // If the code is `None`, then the subprocess ends with a signal.
        self.set_code(res.code.unwrap_or(1)).await;
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
        self.run_with(cmd, Default::default(), &Default::default())
            .await
    }
}

impl<P: Pm> PmHelper for P {}

/// Different ways in which a command shall be dealt with.
/// This is a [`Pm`] specified version intended to be used along with
/// [`Strategy`].
#[derive(Copy, Clone, Debug)]
pub enum PmMode {
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
pub struct Strategy<S = String> {
    pub dry_run: DryRunStrategy<S>,
    pub prompt: PromptStrategy<S>,
    pub no_cache: NoCacheStrategy<S>,
}

/// How a dry run is dealt with.
#[derive(Debug, Clone)]
pub enum DryRunStrategy<S = String> {
    /// Prints the command to be run, and stop.
    PrintCmd,
    /// Invokes the corresponding package manager with the flags given.
    WithFlags(Vec<S>),
}

impl DryRunStrategy<String> {
    pub fn with_flags(flags: &[&str]) -> Self {
        Self::WithFlags(flags.iter().map(|&s| s.to_owned()).collect())
    }
}

impl<S> Default for DryRunStrategy<S> {
    fn default() -> Self {
        DryRunStrategy::PrintCmd
    }
}

/// How the prompt is dealt with when running the package manager.
#[derive(Debug, Clone)]
pub enum PromptStrategy<S = String> {
    /// There is no prompt.
    None,
    /// There is no prompt, but a custom prompt is added.
    CustomPrompt,
    /// There is a native prompt provided by the package manager
    /// that can be disabled with a flag.
    NativeNoConfirm(Vec<S>),
    /// There is a native prompt provided by the package manager
    /// that can be enabled with a flag.
    NativeConfirm(Vec<S>),
}

impl PromptStrategy<String> {
    pub fn native_no_confirm(no_confirm: &[&str]) -> Self {
        Self::NativeNoConfirm(no_confirm.iter().map(|&s| s.to_owned()).collect())
    }

    pub fn native_confirm(confirm: &[&str]) -> Self {
        Self::NativeConfirm(confirm.iter().map(|&s| s.to_owned()).collect())
    }
}

impl<S> Default for PromptStrategy<S> {
    fn default() -> Self {
        PromptStrategy::None
    }
}

/// How the cache is cleaned when `no_cache` is set to `true`.
#[derive(Debug, Clone)]
pub enum NoCacheStrategy<S = String> {
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
    WithFlags(Vec<S>),
}

impl NoCacheStrategy<String> {
    pub fn with_flags(flags: &[&str]) -> Self {
        Self::WithFlags(flags.iter().map(|&s| s.to_owned()).collect())
    }
}

impl<S> Default for NoCacheStrategy<S> {
    fn default() -> Self {
        NoCacheStrategy::None
    }
}
