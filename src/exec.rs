use crate::error::Error;
use crate::print::*;
pub use is_root::is_root;
use regex::Regex;
use std::ffi::OsStr;
use std::io::{BufReader, Read, Write};
use std::sync::Mutex;
use subprocess::{Exec, Redirection};

/// Different ways in which a command shall be dealt with.
#[derive(Copy, Clone, Debug)]
pub enum Mode {
    /// Solely print out the command that should be executed, and stop.
    PrintCmd,

    /// Silently collect all the `stdout`/`stderr` combined. Print nothing.
    Mute,

    /// Print out the command which should be executed, run it and collect its `stdout`/`stderr` combined.
    /// Potentially dangerous as it destroys the colored `stdout`. Use it only if really necessary.
    CheckAll,

    /// Print out the command which should be executed, run it and collect its `stderr`.
    /// This will work with a colored `stdout`.
    CheckErr,

    /// A CUSTOM prompt implemented by `pacaptr`.
    /// Like `CheckErr`, but will ask for confirmation before proceeding.
    Prompt,
}

/// A command to be executed, provided in `command-keywords-flags` form.  
/// For example, `[brew install]-[curl fish]-[--dry-run]`).
#[derive(Debug, Clone, Default)]
pub struct Cmd<S = String> {
    pub sudo: bool,
    pub cmd: Vec<S>,
    pub kws: Vec<S>,
    pub flags: Vec<S>,
}

impl<S: AsRef<OsStr>> Cmd<S> {
    /// Convert a `Cmd` object into a `subprocess::Exec`.
    pub fn build(self) -> Exec {
        // * We use `sudo -S` to launch subprocess if `sudo` is `true` and the current user is not `root`.
        let builder = if self.sudo && !is_root() {
            Exec::cmd("sudo").arg("-S").args(&self.cmd)
        } else {
            let (cmd, subcmd) = (self.cmd[0], &self.cmd[1..]);
            Exec::cmd(cmd).args(subcmd)
        };
        builder.args(&self.kws).args(&self.flags)
    }
}

impl<S: AsRef<OsStr> + AsRef<str>> Cmd<S> {
    /// Execute a command and return a `Result<Vec<u8>, _>`.  
    /// The exact behavior depends on the `mode` passed in.  
    /// See `exec::Mode`'s documentation for more info.
    pub fn exec(self, mode: Mode) -> Result<Vec<u8>, Error> {
        match mode {
            Mode::PrintCmd => {
                print_cmd(&self, PROMPT_DRYRUN);
                Ok(Vec::new())
            }
            Mode::Mute => self.exec_checkall(true),
            Mode::CheckAll => {
                print_cmd(&self, PROMPT_RUN);
                self.exec_checkall(false)
            }
            Mode::CheckErr => {
                print_cmd(&self, PROMPT_RUN);
                self.exec_checkerr(false)
            }
            Mode::Prompt => self.exec_prompt(false),
        }
    }

    /// Execute a command and return its `stdout` and `stderr`.
    /// If `mute` is `false`, then its normal `stdout/stderr` will be printed in the console too.
    fn exec_checkall(self, mute: bool) -> Result<Vec<u8>, Error> {
        let stdout_reader = self
            .build()
            .stderr(Redirection::Merge)
            .stream_stdout()
            .map_err(|_| Error::from("Could not capture stdout, is the executable valid?"))
            .map(BufReader::new)?;

        let mut out = Vec::<u8>::new();
        let mut stdout = std::io::stdout();

        for mb in stdout_reader.bytes() {
            let b = mb?;
            out.write_all(&[b])?;
            if !mute {
                stdout.write_all(&[b])?;
            }
        }

        Ok(out)
    }

    /// Execute a command and collect its `stderr`.  
    /// If `mute` is `false`, then its normal `stderr` will be printed in the console too.
    fn exec_checkerr(self, mute: bool) -> Result<Vec<u8>, Error> {
        let stderr_reader = self
            .build()
            .stream_stderr()
            .map_err(|_| Error::from("Could not capture stderr, is the executable valid?"))
            .map(BufReader::new)?;

        let mut out = Vec::<u8>::new();
        let mut stderr = std::io::stderr();

        for mb in stderr_reader.bytes() {
            let b = mb?;
            out.write_all(&[b])?;
            if !mute {
                stderr.write_all(&[b])?;
            }
        }

        Ok(out)
    }

    /// Execute a command and collect its `stderr`.
    /// If `mute` is `false`, then its normal `stderr` will be printed in the console too.
    /// The user will be prompted if (s)he wishes to continue with the command execution.
    #[allow(clippy::mutex_atomic)]
    fn exec_prompt(self, mute: bool) -> Result<Vec<u8>, Error> {
        lazy_static! {
            static ref ALL_YES: Mutex<bool> = Mutex::new(false);
        }

        let mut all_yes = ALL_YES.lock().unwrap();
        let proceed: bool = if *all_yes {
            true
        } else {
            print_cmd(&self, PROMPT_DRYRUN);
            match prompt(
                "Proceed",
                "[Yes/all/no]",
                &["", "y", "yes", "a", "all", "n", "no"],
                false,
            )
            .to_lowercase()
            .as_ref()
            {
                // The default answer is `Yes`
                "y" | "yes" | "" => true,
                // You can also say `All` to answer `Yes` to all the other questions that follow.
                "a" | "all" => {
                    *all_yes = true;
                    true
                }
                // Or you can say `No`.
                "n" | "no" => false,
                _ => unreachable!(),
            }
        };
        if !proceed {
            return Ok(Vec::new());
        }
        print_cmd(&self, PROMPT_RUN);
        self.exec_checkerr(mute)
    }

    /// Execute a command and collect its `stderr`.
    /// If `mute` is `false`, then its normal `stderr` will be printed in the console too.
    /// The extra flags will be used during the command execution.
    fn exec_withflags(self, flags: Vec<S>, mode: Mode) -> Result<Vec<u8>, Error> {
        let mut cmd = self;
        self.cmd.extend(flags);
        self.exec(mode)
    }
}

impl<S: AsRef<str>> std::fmt::Display for Cmd<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Cmd {
            sudo,
            cmd,
            kws,
            flags,
        } = self;
        let sudo: &[&str] = if *sudo && !is_root() {
            &["sudo", "-S"]
        } else {
            &[]
        };
        let cmd_full = cmd
            .iter()
            .chain(kws)
            .chain(flags)
            .map(|s| &s as &dyn AsRef<str>);
        let res = sudo
            .into_iter()
            .map(|&s| &s as &dyn AsRef<str>)
            .chain(cmd_full)
            .map(|s| s.as_ref())
            .collect::<Vec<&str>>()
            .join(" ");
        write!(f, "{}", res)
    }
}

/// Prompt and get the output string.
/// This action won't end until an expected answer is found.
/// If `case_sensitive == false`, then `expected` should be all lower case patterns.
pub fn prompt(question: &str, options: &str, expected: &[&str], case_sensitive: bool) -> String {
    loop {
        let mut answer = String::new();
        print_question(question, options);
        let _ = std::io::stdout().flush();
        let read = std::io::stdin().read_line(&mut answer);
        if !case_sensitive {
            answer = answer.to_lowercase();
        }
        if read.is_ok() {
            if let Some('\n') = answer.chars().next_back() {
                answer.pop();
            }
            if let Some('\r') = answer.chars().next_back() {
                answer.pop();
            }
            if expected.iter().any(|&x| x == answer) {
                break answer;
            }
        }
    }
}

/// Find all lines in the given `text` that matches all the `patterns`.
pub fn grep(text: &str, patterns: &[&str]) -> Vec<String> {
    let rs: Vec<Regex> = patterns
        .iter()
        .map(|&pat| Regex::new(pat).unwrap())
        .collect();
    text.lines()
        .filter(|&line| rs.iter().all(|regex| regex.find(line).is_some()))
        .map(|s| s.to_owned())
        .collect()
}

/// Check if an executable exists by name (consult `$PATH`) or by path.
/// To check by one parameter only, pass `""` as another.
pub fn is_exe(name: &str, path: &str) -> bool {
    (!path.is_empty() && std::path::Path::new(path).exists())
        || (!name.is_empty() && which::which(name).is_ok())
}
