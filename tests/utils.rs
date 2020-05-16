use pacaptr::exec::{self, Mode};
use regex;

static CARGO: &str = "cargo";
static RUN: &[&str] = &["run", "--"];

#[derive(Debug)]
enum Input<'i> {
    Pacaptr {
        args: &'i [&'i str],
        flags: &'i [&'i str],
    },
    Exec {
        cmd: &'i str,
        subcmd: &'i [&'i str],
        kws: &'i [&'i str],
    },
}

pub struct Test<'t> {
    sequence: Vec<(Input<'t>, &'t [&'t str])>,
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
        if let Some(_) = self.pending_input {
            panic!("Unexpected consecutive inputs")
        } else {
            self.pending_input = Some(Input::Pacaptr { args, flags });
        }
        self
    }

    pub fn exec(mut self, cmd: &'t str, subcmd: &'t [&str], kws: &'t [&str]) -> Self {
        // Guard against consecutive inputs without calling `self.output()`.
        if let Some(_) = self.pending_input {
            panic!("Unexpected consecutive inputs")
        } else {
            self.pending_input = Some(Input::Exec { cmd, subcmd, kws });
        }
        self
    }

    pub fn output(mut self, out: &'t [&str]) -> Self {
        // Guard against `self.output()` without `self.pending_input` being set.
        let cmd = std::mem::replace(&mut self.pending_input, None)
            .expect("Expect an input before an output");
        self.sequence.push((cmd, out));
        self
    }

    pub fn run(&self, verbose: bool) {
        let try_match = |out: &str, patterns: &[&str]| {
            patterns
                .iter()
                .map(|&p| (p, regex::Regex::new(p).unwrap()))
                .for_each(|(p, re)| {
                    assert!(
                        re.find(out).is_some(),
                        "Failed with pattern `{}`, got `{}`",
                        p,
                        out
                    )
                })
        };

        // Prevent running the test before `self.sequence` is configured.
        if self.sequence.is_empty() {
            panic!("Test sequence not yet configured")
        }

        for (input, patterns) in &self.sequence {
            // got = cmd.run()
            // if not matches_all(got, patterns):
            //     raise MatchError(some_msg)
            let mode = if verbose { Mode::CheckAll } else { Mode::Mute };
            let got_bytes: Vec<u8> = match input {
                &Input::Pacaptr { args, flags } => {
                    exec::exec(CARGO, RUN, args, flags, mode).unwrap()
                }
                &Input::Exec { cmd, subcmd, kws } => {
                    exec::exec(cmd, subcmd, kws, &[], mode).unwrap()
                }
            };
            let got = String::from_utf8(got_bytes).unwrap();
            try_match(&got, *patterns);
        }
    }
}
