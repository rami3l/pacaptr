#![cfg(target_os = "linux")]

mod common;
use common::*;

#[test]
fn apk_si_ok() {
    Test::new()
        .pacaptr(&["-Si", "wget"], &[])
        .output(&["Network utility to retrieve files from the Web"])
        .run()
}

#[test]
#[should_panic(expected = "Failed with pattern `Why not use curl instead?`")]
fn apk_si_fail() {
    Test::new()
        .pacaptr(&["-Si", "wget"], &[])
        .output(&["Why not use curl instead?"])
        .run()
}

#[test]
#[ignore]
fn apk_r() {
    Test::new()
        .pacaptr(&["-S", "wget", "--yes"], &[])
        .output(&["Installing wget"])
        .exec(&["wget", "-V"], &[])
        .output(&["GNU Wget"])
        .pacaptr(&["-R", "wget", "--yes"], &[])
        .output(&["Purging wget"])
        .run()
}
