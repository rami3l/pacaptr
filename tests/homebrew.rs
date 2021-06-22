#![cfg(target_os = "macos")]

mod common;
use common::*;

#[test]
fn homebrew_si_ok() {
    test_dsl! { r##"
        in -Si curl
        ou curl is keg-only
    "## }
}

#[test]
#[should_panic(expected = "Failed with pattern `curl is not keg-only`")]
fn homebrew_si_fail() {
    test_dsl! { r##"
        in -Si curl
        ou curl is not keg-only
    "## }
}

#[test]
#[ignore]
fn homebrew_r() {
    test_dsl! { r##"
        in -S wget --yes
        ou brew (re)?install wget
        in ! wget -V
        ou GNU Wget
        in -R wget --yes
        ou brew uninstall wget
        ou Uninstalling /usr/local/Cellar/wget
    "## }
}
