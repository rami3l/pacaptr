pub mod apk;
pub mod apt;
pub mod chocolatey;
pub mod conda;
pub mod dnf;
pub mod homebrew;
pub mod macports;
pub mod pip;
pub mod tlmgr;
pub mod unknown;
pub mod zypper;

use crate::dispatch::config::Config;
use crate::exec::{Cmd, Mode, Output, StatusCode};
use anyhow::Result;
use tokio::sync::Mutex;

/*
macro_rules! make_pm {(
        $(
            $( #[$meta:meta] )*
            $method:ident
        ),*
    ) => {
        $( $(#[$meta] )*
        fn $method(&self, _kws: &[&str], _flags: &[&str]) -> BoxFuture<'_, anyhow::Result<()>>
        {
            Box::pin(async move {
                let name = self.name();
                ::std::result::Result::Err(anyhow::anyhow!(
                    format!(
                        "Operation `{}` unimplemented for `{}`",
                        stringify!($method),
                        name,
                    ),
                ))
            })
        })*
    };
}
*/

macro_rules! make_op_body {
    ( $self:ident, $method:ident ) => {{
        let name = $self.name();
        Err(anyhow::anyhow!(format!(
            "Operation `{}` unimplemented for `{}`",
            stringify!($method),
            name,
        )))
    }};
}

/// The behaviors of a Pack(age)Manager.
/// For method explanation see: https://wiki.archlinux.org/index.php/Pacman/Rosetta
/// and https://wiki.archlinux.org/index.php/Pacman
#[async_trait]
pub trait PackageManager: Sync {
    /// Get the name of the package manager.
    fn name(&self) -> String;

    /// Get the config of the package manager.
    fn cfg(&self) -> Config;

    /// Get the `StatusCode` to be returned.
    async fn code(&self) -> StatusCode {
        self._code(None).await
    }

    /// Set the `StatusCode` to be returned.
    async fn set_code(&self, to: StatusCode) {
        self._code(Some(to)).await;
    }

    /// Get/Set the `StatusCode` to be returned.
    /// If `to` is `Some(n)`, then the current `StatusCode` will be reset to `n`.
    /// Then the current `StatusCode` will be returned.
    #[doc(hidden)]
    async fn _code(&self, to: Option<StatusCode>) -> StatusCode {
        lazy_static! {
            static ref CODE: Mutex<StatusCode> = Mutex::new(0);
        }

        let mut code = CODE.lock().await;
        if let Some(n) = to {
            *code = n;
        }

        *code
    }

    /// A helper method to simplify direct command invocation.
    async fn run(&self, mut cmd: Cmd, mode: PmMode, strat: Strategies) -> Result<Output> {
        let cfg = self.cfg();

        // `--dry-run` should apply to both the main command and the cleanup.
        async fn body(cfg: &Config, cmd: &Cmd, mode: PmMode, strat: &Strategies) -> Result<Output> {
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
                PromptStrategy::NativePrompt { no_confirm: v } => {
                    if no_confirm {
                        curr_cmd.flags.extend(v.to_owned());
                    }
                    curr_cmd.exec(mode.into()).await
                }
            }
        };

        let res = match &strat.dry_run {
            DryRunStrategy::PrintCmd if self.cfg().dry_run => {
                cmd.clone().exec(Mode::PrintCmd).await?
            }
            DryRunStrategy::WithFlags(v) if self.cfg().dry_run => {
                cmd.flags.extend(v.to_owned());
                // * A dry run with extra flags does not need `sudo`.
                cmd = cmd.sudo(false);
                body(&cfg, &cmd, mode, &strat).await?
            }
            _ => body(&cfg, &cmd, mode, &strat).await?,
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
        self.set_code(res.code.unwrap_or(1)).await;

        Ok(res)
    }

    /// A helper method to simplify direct command invocation.
    /// It is just like `run`, but intended to be used only for its side effects.
    async fn just_run(&self, cmd: Cmd, mode: PmMode, strat: Strategies) -> Result<()>
    where
        Self: Sized,
    {
        self.run(cmd, mode, strat).await.and(Ok(()))
    }

    /// A helper method to simplify direct command invocation.
    /// It is just like `run`, but intended to be used only for its side effects, and always with default mode (`CheckErr` for now) and strategies.
    async fn just_run_default(&self, cmd: Cmd) -> Result<()>
    where
        Self: Sized,
    {
        self.just_run(cmd, Default::default(), Default::default())
            .await
    }

    // ! WARNING!
    // ! Dirty copy-paste!

    /// Q generates a list of installed packages.
    async fn q(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
        make_op_body!(self, q)
    }

    /// Qc shows the changelog of a package.
    async fn qc(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
        make_op_body!(self, qc)
    }

    /// Qe lists packages installed explicitly (not as dependencies).
    async fn qe(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
        make_op_body!(self, qe)
    }

    /// Qi displays local package information: name, version, description, etc.
    async fn qi(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
        make_op_body!(self, qi)
    }

    /// Qk verifies one or more packages.
    async fn qk(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
        make_op_body!(self, qk)
    }

    /// Ql displays files provided by local package.
    async fn ql(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
        make_op_body!(self, ql)
    }

    /// Qm lists packages that are installed but are not available in any installation source (anymore).
    async fn qm(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
        make_op_body!(self, qm)
    }

    /// Qo queries the package which provides FILE.
    async fn qo(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
        make_op_body!(self, qo)
    }

    /// Qp queries a package supplied on the command line rather than an entry in the package management database.
    async fn qp(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
        make_op_body!(self, qp)
    }

    /// Qs searches locally installed package for names or descriptions.
    async fn qs(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
        make_op_body!(self, qs)
    }

    /// Qu lists packages which have an update available.
    async fn qu(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
        make_op_body!(self, qu)
    }

    /// R removes a single package, leaving all of its dependencies installed.
    async fn r(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
        make_op_body!(self, r)
    }

    /// Rn removes a package and skips the generation of configuration backup files.
    async fn rn(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
        make_op_body!(self, rn)
    }

    /// Rns removes a package and its dependencies which are not required by any other installed package,
    /// and skips the generation of configuration backup files.
    async fn rns(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
        make_op_body!(self, rns)
    }

    /// Rs removes a package and its dependencies which are not required by any other installed package,
    /// and not explicitly installed by the user.
    async fn rs(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
        make_op_body!(self, rs)
    }

    /// Rss removes a package and its dependencies which are not required by any other installed package.
    async fn rss(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
        make_op_body!(self, rss)
    }

    /// S installs one or more packages by name.
    async fn s(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
        make_op_body!(self, s)
    }

    /// Sc removes all the cached packages that are not currently installed, and the unused sync database.
    async fn sc(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
        make_op_body!(self, sc)
    }

    /// Scc removes all files from the cache.
    async fn scc(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
        make_op_body!(self, scc)
    }

    /// Sccc ...
    /// What is this?
    async fn sccc(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
        make_op_body!(self, sccc)
    }

    /// Sg lists all packages belonging to the GROUP.
    async fn sg(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
        make_op_body!(self, sg)
    }

    /// Si displays remote package information: name, version, description, etc.
    async fn si(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
        make_op_body!(self, si)
    }

    /// Sii displays packages which require X to be installed, aka reverse dependencies.
    async fn sii(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
        make_op_body!(self, sii)
    }

    /// Sl displays a list of all packages in all installation sources that are handled by the packages management.
    async fn sl(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
        make_op_body!(self, sl)
    }

    /// Ss searches for package(s) by searching the expression in name, description, short description.
    async fn ss(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
        make_op_body!(self, ss)
    }

    /// Su updates outdated packages.
    async fn su(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
        make_op_body!(self, su)
    }

    /// Suy refreshes the local package database, then updates outdated packages.
    async fn suy(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
        make_op_body!(self, suy)
    }

    /// Sw retrieves all packages from the server, but does not install/upgrade anything.
    async fn sw(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
        make_op_body!(self, sw)
    }

    /// Sy refreshes the local package database.
    async fn sy(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
        make_op_body!(self, sy)
    }

    /// U upgrades or adds package(s) to the system and installs the required dependencies from sync repositories.
    async fn u(&self, _kws: &[&str], _flags: &[&str]) -> Result<()> {
        make_op_body!(self, u)
    }
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

impl Default for PmMode {
    fn default() -> Self {
        PmMode::CheckErr
    }
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
    /// There is a native prompt provided by the package manager.
    NativePrompt {
        /// Flags required to bypass native prompt.
        no_confirm: Vec<S>,
    },
}

impl PromptStrategy<String> {
    pub fn native_prompt(no_confirm: &[&str]) -> Self {
        Self::NativePrompt {
            no_confirm: no_confirm.iter().map(|&s| s.to_owned()).collect(),
        }
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
    /// Do not clean cache.
    /// This variant MUST be used when implementing cache cleaning methods like `-Sc`.
    None,
    /// Use `-Sc` to clean the cache.
    Sc,
    /// Use `-Scc`.
    Scc,
    /// Use `-Sccc`.
    Sccc,
    /// Invoke the corresponding package manager with the flags given.
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

/*
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    struct MockPM {}

    #[async_trait]
    impl PackageManager for MockPM {
        /// Get the name of the package manager.
        fn name(&self) -> String {
            "mockpm".into()
        }

        fn cfg(&self) -> Config {
            Config::default()
        }
    }

    #[test]
    async fn simple_run() {
        println!("Starting!");
        let cmd = Cmd::new(&["bash", "-c"])
            .kws(&[r#"printf "Hello\n"; sleep 3; printf "World\n"; sleep 3; printf "!\n""#]);
        let res = MockPM {}
            .run(cmd, PmMode::CheckErr, Default::default())
            .await
            .unwrap();
        dbg!(res);
    }
}
*/
