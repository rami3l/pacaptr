#![cfg(target_os = "linux")]

mod common;
use common::*;

#[test]
fn linuxbrew_si_ok() {
    test_dsl! { r##"
        in -Si curl
        ou Get a file from an HTTP, HTTPS or FTP server
    "## }
}

#[test]
#[ignore]
fn linuxbrew_r() {
    test_dsl! { r##"
        in -S wget --yes
        ou brew (re)?install wget
        in ! wget -V
        ou GNU Wget
        in -R wget --yes
        ou brew uninstall wget
    "## }
}
