#![cfg(feature = "test")]
#![allow(clippy::dbg_macro, clippy::missing_panics_doc)]

use itertools::{Itertools, chain};
pub use pacaptr_macros::test_dsl;
use regex::RegexBuilder;
use xshell::{Shell, cmd};

#[derive(Debug)]
enum Input<'i> {
    Pacaptr {
        args: &'i [&'i str],
        flags: &'i [&'i str],
    },
    #[allow(dead_code)]
    Exec {
        cmd: &'i [&'i str],
        kws: &'i [&'i str],
    },
}

/// Returns the platform specific prefix of calling a command encoded as string.
const fn cmd_prefix() -> (&'static str, &'static [&'static str]) {
    match () {
        () if cfg!(windows) => ("powershell", &["-Command"]),
        () => ("sh", &["-c"]),
    }
}

#[derive(Debug, Default)]
pub struct Test<'t> {
    sequence: Vec<(Input<'t>, Vec<&'t str>)>,
    pending_input: Option<Input<'t>>,
}

impl<'t> Test<'t> {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn pacaptr(mut self, args: &'t [&str], flags: &'t [&str]) -> Self {
        // Guard against consecutive inputs without calling `self.output()`.
        if self.pending_input.is_some() {
            self = self.output(&[]);
        }
        self.pending_input = Some(Input::Pacaptr { args, flags });
        self
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn exec(mut self, cmd: &'t [&str], kws: &'t [&str]) -> Self {
        // Guard against consecutive inputs without calling `self.output()`.
        if self.pending_input.is_some() {
            self = self.output(&[]);
        }
        self.pending_input = Some(Input::Exec { cmd, kws });
        self
    }

    #[must_use]
    pub fn output(mut self, out: &'t [&str]) -> Self {
        if let Some(cmd) = self.pending_input.take() {
            self.sequence.push((cmd, out.into()));
        } else if let Some((_cmd, outs)) = self.sequence.last_mut() {
            outs.extend(out);
        } else {
            panic!("expect an input before an output");
        }
        self
    }

    pub fn run(&self) {
        let try_match = |out: &str, patterns: &[&str]| {
            for &p in patterns {
                let re = RegexBuilder::new(p).multi_line(true).build().unwrap();
                let is_match = re.is_match(out);
                assert!(is_match, "failed with pattern `{p}`, got `{out}`");
            }
        };

        // Prevent running the test before `self.sequence` is configured.
        assert!(
            !self.sequence.is_empty(),
            "Test sequence not yet configured"
        );

        let s = Shell::new().unwrap();
        for (input, patterns) in &self.sequence {
            let (sh, sh_args) = cmd_prefix();
            let cmd = match *input {
                Input::Exec { cmd, kws } => chain!(cmd, kws).join(" "),
                Input::Pacaptr { args, flags } => {
                    format!("cargo run --quiet -- {}", chain!(args, flags).join(" "))
                }
            };
            let cmd = cmd!(s, "{sh}").args(sh_args).arg(dbg!(&cmd));
            let output = cmd.ignore_status().output().unwrap();
            let got = String::from_utf8_lossy(&output.stdout);
            println!("{got}");
            try_match(&got, patterns);
            let code = output.status.code();
            let got_stderr = String::from_utf8_lossy(&output.stderr);
            assert!(
                output.status.success(),
                "failed with exit code {code:?} and the following stderr: {got_stderr}"
            );
        }
    }
}
