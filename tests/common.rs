use itertools::{chain, Itertools};
pub use pacaptr_macros::test_dsl;
use regex::RegexBuilder;
use xshell::{cmd, Shell};

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
        _ if cfg!(target_os = "windows") => ("powershell", &["-Command"]),
        _ => ("sh", &["-c"]),
    }
}

pub struct Test<'t> {
    sequence: Vec<(Input<'t>, Vec<&'t str>)>,
    pending_input: Option<Input<'t>>,
}

impl<'t> Test<'t> {
    pub fn new() -> Self {
        Test {
            sequence: Vec::new(),
            pending_input: None,
        }
    }

    pub fn pacaptr(mut self, args: &'t [&str], flags: &'t [&str]) -> Self {
        // Guard against consecutive inputs without calling `self.output()`.
        if self.pending_input.is_some() {
            self = self.output(&[]);
        }
        self.pending_input = Some(Input::Pacaptr { args, flags });
        self
    }

    #[allow(dead_code)]
    pub fn exec(mut self, cmd: &'t [&str], kws: &'t [&str]) -> Self {
        // Guard against consecutive inputs without calling `self.output()`.
        if self.pending_input.is_some() {
            self = self.output(&[]);
        }
        self.pending_input = Some(Input::Exec { cmd, kws });
        self
    }

    pub fn output(mut self, out: &'t [&str]) -> Self {
        if let Some(cmd) = self.pending_input.take() {
            self.sequence.push((cmd, out.into()))
        } else if let Some((_cmd, outs)) = self.sequence.last_mut() {
            outs.extend(out);
        } else {
            panic!("Expect an input before an output");
        }
        self
    }

    pub fn run(&self) {
        let try_match = |out: &str, patterns: &[&str]| {
            patterns.iter().for_each(|p| {
                let re = RegexBuilder::new(p).multi_line(true).build().unwrap();
                let is_match = re.is_match(out);
                assert!(is_match, "Failed with pattern `{p}`, got `{out}`")
            })
        };

        // Prevent running the test before `self.sequence` is configured.
        if self.sequence.is_empty() {
            panic!("Test sequence not yet configured")
        }

        let s = Shell::new().unwrap();
        self.sequence.iter().for_each(|(input, patterns)| {
            // got = cmd.run()
            // if not matches_all(got, patterns):
            //     raise MatchError(some_msg)
            let (sh, sh_args) = cmd_prefix();
            let cmd = match *input {
                Input::Exec { cmd, kws } => chain!(cmd, kws).join(" "),
                Input::Pacaptr { args, flags } => {
                    format!("cargo run -- {}", chain!(args, flags).join(" "))
                }
            };
            let got = cmd!(s, "{sh}")
                .args(sh_args)
                .arg(dbg!(&cmd))
                .read()
                .unwrap();
            println!("{got}");
            try_match(&got, patterns);
        })
    }
}

impl<'t> Default for Test<'t> {
    fn default() -> Self {
        Test::new()
    }
}
