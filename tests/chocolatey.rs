#![cfg(target_os = "windows")]

mod common;
use common::*;

#[test]
fn chocolatey_si_ok() {
    Test::new()
        .pacaptr(&["-Si", "wget"], &[])
        .output(&["GNU Wget is a free software package"])
        .run_verbose()
}

#[test]
#[should_panic(expected = "Failed with pattern `GNU Wget is not a free software package`")]
fn chocolatey_si_fail() {
    Test::new()
        .pacaptr(&["-Si", "wget"], &[])
        .output(&["GNU Wget is not a free software package"])
        .run_verbose()
}

#[test]
#[ignore]
fn chocolatey_r() {
    Test::new()
        .pacaptr(&["-S", "wget", "--yes"], &[])
        .output(&["The install of wget was successful."])
        .pacaptr(&["-R", "wget", "--yes"], &[])
        .output(&["Wget has been successfully uninstalled."])
        .run_verbose()
}
