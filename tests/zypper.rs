#![cfg(target_os = "linux")]

mod common;
use common::*;

#[test]
fn zypper_si_ok() {
    Test::new()
        .pacaptr(&["-Si", "curl"], &[])
        .output(&["A Tool for Transferring Data from URLs"])
        .run_verbose()
}

#[test]
#[should_panic(expected = "Failed with pattern `Why not use curl instead?`")]
fn zypper_si_fail() {
    Test::new()
        .pacaptr(&["-Si", "wget"], &[])
        .output(&["Why not use curl instead?"])
        .run_verbose()
}

#[test]
#[ignore]
fn zypper_r() {
    Test::new()
        .pacaptr(&["-S", "wget", "--yes"], &[])
        .output(&["zypper install", "-y", "wget", "Installing: wget"])
        .exec(&["wget", "-V"], &[])
        .output(&["GNU Wget"])
        .pacaptr(&["-R", "wget", "--yes"], &[])
        .output(&["zypper remove", "-y", "wget", "Removing wget"])
        .run_verbose()
}
