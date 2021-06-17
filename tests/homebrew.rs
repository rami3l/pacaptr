#![cfg(target_os = "macos")]

mod common;
use common::*;

#[test]
fn homebrew_si_ok() {
    Test::new()
        .pacaptr(&["-Si", "curl"], &[])
        .output(&["curl is keg-only"])
        .run()
}

#[test]
#[should_panic(expected = "Failed with pattern `curl is not keg-only`")]
fn homebrew_si_fail() {
    Test::new()
        .pacaptr(&["-Si", "curl"], &[])
        .output(&["curl is not keg-only"])
        .run()
}

#[test]
#[ignore]
fn homebrew_r() {
    Test::new()
        .pacaptr(&["-S", "wget", "--yes"], &[])
        .output(&["brew (re)?install wget"])
        .exec(&["wget", "-V"], &[])
        .output(&["GNU Wget"])
        .pacaptr(&["-R", "wget", "--yes"], &[])
        .output(&["brew uninstall wget", "Uninstalling /usr/local/Cellar/wget"])
        .run()
}
