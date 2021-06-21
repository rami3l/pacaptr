#![cfg(target_os = "linux")]

mod common;
use common::*;

#[test]
fn apk_si_ok() {
    test_dsl! { r##"
        in -Si wget
        ou Network utility to retrieve files from the Web
    "## }
}

#[test]
#[should_panic(expected = "Failed with pattern `Why not use curl instead?`")]
fn apk_si_ok() {
    test_dsl! { r##"
        in -Si wget
        ou Why not use curl instead?
    "## }
}

#[test]
#[ignore]
fn apk_r() {
    test_dsl! { r##"
        in -S wget --yes
        ou Installing wget
        in ! wget -V
        ou GNU Wget
        in -R wget --yes
        ou Purging wget
    "## }
}
