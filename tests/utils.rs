use pacaptr::exec::{self, Mode};
use regex;

static EXE: &str = "cargo";
static SUBCMD: &[&str] = &["run", "--"];

pub struct Test {
    sequence: Vec<(Vec<String>, Vec<String>)>,
    cmd: Option<Vec<String>>,
}

impl Test {
    pub fn new() -> Self {
        Test {
            sequence: Vec::new(),
            cmd: None,
        }
    }

    pub fn input(mut self, cmd: &[&str]) -> Self {
        // Guard against `self.input().input()`.
        if let Some(_) = self.cmd {
            panic!("Unexpected consecutive input")
        } else {
            self.cmd = Some(cmd.iter().map(|s| s.to_string()).collect());
        }
        self
    }

    pub fn output(mut self, out: &[&str]) -> Self {
        // Guard against `self.output()` without `self.cmd` being set.
        if self.cmd.is_none() {
            panic!("Expect an input before an output")
        }

        let cmd = std::mem::replace(&mut self.cmd, None).unwrap();
        self.sequence
            .push((cmd, out.iter().map(|s| s.to_string()).collect()));
        self
    }

    pub fn run(&self, verbose: bool) {
        let try_match = |out: &str, patterns: &[String]| {
            patterns
                .iter()
                .map(|p| (p, regex::Regex::new(p).unwrap()))
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

        for (kws, patterns) in &self.sequence {
            // got = cmd.run()
            // if not matches_all(got, patterns):
            //     raise MatchError(some_msg)
            let mode = if verbose { Mode::CheckAll } else { Mode::Mute };
            let kws: Vec<&str> = kws.iter().map(|s| s.as_ref()).collect();
            let got_bytes = exec::exec(EXE, SUBCMD, &kws, mode).unwrap();
            let got = String::from_utf8(got_bytes).unwrap();
            try_match(&got, &patterns);
        }
    }
}
