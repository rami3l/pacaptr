#![cfg(target_os = "linux")]

mod common;
use common::*;

#[test]
fn dnf_si_ok() {
    test_dsl! { r##"
        in -Si wget
        ou A utility for retrieving files using the HTTP or FTP protocols
    "## }
}

#[test]
#[should_panic(expected = "Failed with pattern `Why not use curl instead?`")]
fn dnf_si_fail() {
    test_dsl! { r##"
        in -Si wget
        ou Why not use curl instead?
    "## }
}

#[test]
#[ignore]
fn dnf_r() {
    test_dsl! { r##"
        in -S wget --yes
        ou Installed:
        ou Complete!
        in ! wget -V
        ou GNU Wget
        in -R wget --yes
        ou Removed:
        ou Complete!
    "## }
}
