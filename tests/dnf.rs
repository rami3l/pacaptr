#![cfg(target_os = "linux")]

mod common;
use common::*;

#[test]
fn dnf_si_ok() {
    Test::new()
        .pacaptr(&["-Si", "curl"], &[])
        .output(&["A utility for getting files from remote servers"])
        .run()
}

#[test]
#[should_panic(expected = "Failed with pattern `Why not use curl instead?`")]
fn dnf_si_fail() {
    Test::new()
        .pacaptr(&["-Si", "wget"], &[])
        .output(&["Why not use curl instead?"])
        .run()
}

#[test]
#[ignore]
fn dnf_r() {
    Test::new()
        .pacaptr(&["-S", "wget", "--yes"], &[])
        .output(&["dnf install", "-y", "wget", "Installed:", "Complete!"])
        .exec(&["wget", "-V"], &[])
        .output(&["GNU Wget"])
        .pacaptr(&["-R", "wget", "--yes"], &[])
        .output(&["dnf remove", "-y", "wget", "Removed:", "Complete!"])
        .run()
}
