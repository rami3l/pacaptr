#![cfg(target_os = "linux")]

mod common;
use common::*;

#[test]
fn zypper_si_ok() {
    test_dsl! { r##"
        in -Si curl
        ou A Tool for Transferring Data from URLs
    "## }
}

#[test]
#[should_panic(expected = "Failed with pattern `Why not use curl instead?`")]
fn zypper_si_fail() {
    test_dsl! { r##"
        in -Si wget
        ou Why not use curl instead?
    "## }
}

#[test]
#[ignore]
fn zypper_r() {
    test_dsl! { r##"
        in -S wget --yes
        ou Installing: wget
        in ! wget -V
        ou GNU Wget
        in -R wget --yes
        ou Removing wget
    "## }
}
