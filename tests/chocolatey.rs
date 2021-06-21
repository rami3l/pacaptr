#![cfg(target_os = "windows")]

mod common;
use common::*;

#[test]
fn chocolatey_si_ok() {
    test_dsl! { r##"
        in -Si wget
        ou GNU Wget is a free software package
    "## }
}

#[test]
#[should_panic(expected = "Failed with pattern `GNU Wget is not a free software package`")]
fn chocolatey_si_fail() {
    test_dsl! { r##"
        in -Si wget
        ou GNU Wget is not a free software package
    "## }
}

#[test]
#[ignore]
fn chocolatey_r() {
    test_dsl! { r##"
        in -S wget --yes
        ou The install of wget was successful.
        in -R wget --yes
        ou Wget has been successfully uninstalled.
    "## }
}
