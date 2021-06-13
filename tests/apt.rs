#![cfg(target_os = "linux")]

mod common;
use common::*;

#[test]
fn apt_si_ok() {
    Test::new()
        .pacaptr(&["-Si", "screen"], &[])
        .output(&["Package: screen"])
        .run_verbose()
}

#[test]
#[should_panic(expected = "Failed with pattern `Package: wget`")]
fn apt_si_fail() {
    Test::new()
        .pacaptr(&["-Si", "screen"], &[])
        .output(&["Package: wget"])
        .run_verbose()
}

#[test]
#[ignore]
fn apt_r() {
    Test::new()
        .pacaptr(&["-S", "screen", "--yes"], &[])
        .output(&["apt(-get)? install", "--reinstall", "--yes", "screen"])
        .pacaptr(&["-Qi", "screen"], &[])
        .output(&["Status: install"])
        .pacaptr(&["-R", "screen", "--yes"], &[])
        .output(&["apt(-get)? remove", "--yes", "screen"])
        .pacaptr(&["-Qi", "screen"], &[])
        .output(&["Status: deinstall"])
        .run_verbose()
}
