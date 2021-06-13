#![cfg(target_os = "linux")]

mod common;
use common::*;

#[test]
fn linuxbrew_si_ok() {
    Test::new()
        .pacaptr(&["-Si", "curl"], &[])
        .output(&["Get a file from an HTTP, HTTPS or FTP server"])
        .run()
}

#[test]
#[ignore]
fn linuxbrew_r() {
    Test::new()
        .pacaptr(&["-S", "wget", "--yes"], &[])
        .output(&["brew (re)?install wget"])
        .exec(&["wget", "-V"], &[])
        .output(&["GNU Wget"])
        .pacaptr(&["-R", "wget", "--yes"], &[])
        .output(&["brew uninstall wget"])
        .run()
}
