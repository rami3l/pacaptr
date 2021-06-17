#![cfg(target_os = "windows")]

mod common;
use common::*;

#[test]
fn scoop_si_ok() {
    Test::new()
            .pacaptr(&["--using", "scoop", "-Si", "wget"], &[])
            .output(&["Description: A command-line utility for retrieving files using HTTP, HTTPS, FTP, and FTPS protocols."])
            .run()
}

#[test]
#[should_panic(expected = "Failed with pattern `GNU Wget is not a free software package`")]
fn scoop_si_fail() {
    Test::new()
        .pacaptr(&["--using", "scoop", "-Si", "wget"], &[])
        .output(&["GNU Wget is not a free software package"])
        .run()
}

#[test]
#[ignore]
fn scoop_r() {
    Test::new()
        .pacaptr(&["--using", "scoop", "-S", "wget", "--yes"], &[])
        .output(&["wget", "was installed successfully!"])
        .pacaptr(&["--using", "scoop", "-Q"], &[])
        .output(&["wget", "[main]"])
        .pacaptr(&["--using", "scoop", "-R", "wget", "--yes"], &[])
        .output(&["wget", "was uninstalled."])
        .run()
}
