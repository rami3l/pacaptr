pub mod apk;
pub mod apt;
pub mod aptget;
pub mod chocolatey;
pub mod conda;
pub mod dnf;
pub mod homebrew;
pub mod linuxbrew;
pub mod macports;
pub mod pip;
pub mod tlmgr;
pub mod unknown;
pub mod zypper;

use crate::dispatch::config::Config;
use crate::error::Error;
use crate::exec::{self, Cmd, Mode};

macro_rules! make_pm {
    ($( $(#[$meta:meta])* $method:ident ), *) => {
        $($(#[$meta])*
        fn $method(&self, _kws: &[&str], _flags: &[&str]) -> std::result::Result<(), crate::error::Error> {
            std::result::Result::Err(format!("Operation `{}` unimplemented for `{}`", stringify!($method), self.name()).into())
        })*
    };
}

/// The behaviors of a Pack(age)Manager.
/// For method explanation see: https://wiki.archlinux.org/index.php/Pacman/Rosetta
/// and https://wiki.archlinux.org/index.php/Pacman
pub trait PackageManager {
    /// Get the name of the package manager.
    fn name(&self) -> String;

    /// Get the config of the package manager.
    fn cfg(&self) -> Config;

    /// A helper method to simplify direct command invocation.
    fn run(&self, mut cmd: Cmd, mode: PmMode, strat: Strategies) -> Result<Vec<u8>, Error> {
        match strat.dry_run {
            _ if !self.cfg().dry_run => (),
            DryRunStrategy::PrintCmd => return cmd.exec(Mode::PrintCmd),
            DryRunStrategy::WithFlags(v) => cmd.flags.extend(v),
        };

        let no_confirm = self.cfg().no_confirm;
        let res = match strat.prompt {
            PromptStrategy::None | PromptStrategy::CustomPrompt if no_confirm => {
                cmd.exec(Mode::CheckErr)?
            }
            PromptStrategy::CustomPrompt => cmd.exec(Mode::Prompt)?,
            PromptStrategy::NativePrompt { no_confirm: v } => {
                let mut curr_cmd = cmd.clone();
                if no_confirm {
                    curr_cmd.flags.extend(v);
                }
                curr_cmd.exec(Mode::CheckErr)?
            }
        };

        let flags = cmd.flags.iter().map(|s| s.as_ref()).collect::<Vec<_>>();
        match strat.no_cache {
            _ if !self.cfg().no_cache => (),
            NoCacheStrategy::None => (),
            NoCacheStrategy::Sc => self.sc(&[], &flags)?,
            NoCacheStrategy::Scc => self.scc(&[], &flags)?,
        };

        Ok(res)
    }

    make_pm!(
        /// Q generates a list of installed packages.
        q,
        /// Qc shows the changelog of a package.
        qc,
        /// Qe lists packages installed explicitly (not as dependencies).
        qe,
        /// Qi displays local package information: name, version, description, etc.
        qi,
        /// Qk verifies one or more packages.
        qk,
        /// Ql displays files provided by local package.
        ql,
        /// Qm lists packages that are installed but are not available in any installation source (anymore).
        qm,
        /// Qo queries the package which provides FILE.
        qo,
        /// Qp queries a package supplied on the command line rather than an entry in the package management database.
        qp,
        /// Qs searches locally installed package for names or descriptions.
        qs,
        /// Qu lists packages which have an update available.
        qu,
        /// R removes a single package, leaving all of its dependencies installed.
        r,
        /// Rn removes a package and skips the generation of configuration backup files.
        rn,
        /// Rns removes a package and its dependencies which are not required by any other installed package,
        /// and skips the generation of configuration backup files.
        rns,
        /// Rs removes a package and its dependencies which are not required by any other installed package,
        /// and not explicitly installed by the user.
        rs,
        /// Rss removes a package and its dependencies which are not required by any other installed package.
        rss,
        /// S installs one or more packages by name.
        s,
        /// Sc removes all the cached packages that are not currently installed, and the unused sync database.
        sc,
        /// Scc removes all files from the cache.
        scc,
        /// Sccc ...
        /// What is this?
        sccc,
        /// Sg lists all packages belonging to the GROUP.
        sg,
        /// Si displays remote package information: name, version, description, etc.
        si,
        /// Sii displays packages which require X to be installed, aka reverse dependencies.
        sii,
        /// Sl displays a list of all packages in all installation sources that are handled by the packages management.
        sl,
        /// Ss searches for package(s) by searching the expression in name, description, short description.
        ss,
        /// Su updates outdated packages.
        su,
        /// Suy refreshes the local package database, then updates outdated packages.
        suy,
        /// Sw retrieves all packages from the server, but does not install/upgrade anything.
        sw,
        /// Sy refreshes the local package database.
        sy,
        /// U upgrades or adds package(s) to the system and installs the required dependencies from sync repositories.
        u
    );
}

/// Different ways in which a command shall be dealt with.
/// This is a `PackageManager` specified version intended to be used along with `Strategies`.
#[derive(Copy, Clone, Debug)]
pub enum PmMode {
    /// Silently collect all the `stdout`/`stderr` combined. Print nothing.
    Mute,

    /// Print out the command which should be executed, run it and collect its `stdout`/`stderr` combined.
    /// Potentially dangerous as it destroys the colored `stdout`. Use it only if really necessary.
    CheckAll,

    /// Print out the command which should be executed, run it and collect its `stderr`.
    /// This will work with a colored `stdout`.
    CheckErr,
}

impl Into<Mode> for PmMode {
    fn into(self) -> Mode {
        match self {
            PmMode::Mute => Mode::Mute,
            PmMode::CheckAll => Mode::CheckAll,
            PmMode::CheckErr => Mode::CheckErr,
        }
    }
}

/// The intrinsic properties of a command of a specific package manager
/// indicating how it is run.
#[derive(Clone, Debug, Default)]
pub struct Strategies<S = String> {
    dry_run: DryRunStrategy<S>,
    prompt: PromptStrategy<S>,
    no_cache: NoCacheStrategy<S>,
}

/// How a dry run is dealt with.
#[derive(Debug, Clone)]
pub enum DryRunStrategy<S = String> {
    /// Print the command to be run, and stop.
    PrintCmd,
    /// Invoke the corresponding package manager with the flags given.
    WithFlags(Vec<S>),
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
    /// There is a native prompt provided by the package manager.
    NativePrompt {
        /// Flags required to bypass native prompt.
        no_confirm: Vec<S>,
    },
}

impl<S> Default for PromptStrategy<S> {
    fn default() -> Self {
        PromptStrategy::None
    }
}

/// How the cache is cleaned when `no_cache` is set to `true`.
#[derive(Debug, Clone)]
pub enum NoCacheStrategy<S = String> {
    /// Do not clean cache.
    None,
    /// Use `-Sc` to clean the cache.
    Sc,
    /// Use `-Scc`.
    Scc,
    /// Invoke the corresponding package manager with the flags given.
    WithFlags(Vec<S>),
}

impl<S> Default for NoCacheStrategy<S> {
    fn default() -> Self {
        NoCacheStrategy::None
    }
}
