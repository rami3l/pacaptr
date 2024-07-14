#![cfg(all(target_os = "macos", feature = "test"))]

mod common;
use common::*;

#[test]
#[should_panic(expected = "failed with pattern `curl is not keg-only`")]
fn brew_fail() {
    test_dsl! { r##"
        in --using brew -Si curl
        ou curl is not keg-only
    "## }
}

#[test]
fn brew_q() {
    test_dsl! { r##"
        in --using brew -Q
        ou wget
        in --using brew -Q wget
        ou wget
    "## }
}

#[test]
fn brew_qc() {
    test_dsl! { r##"
        in --using brew -Qc curl
        ou github\.com
    "## }
}

#[test]
fn brew_qi() {
    test_dsl! { r##"
        in --using brew -Qi curl
        ou Get a file from an HTTP, HTTPS or FTP server
    "## }
}

#[test]
fn brew_ql() {
    test_dsl! { r##"
        in --using brew -Ql wget
        ou Cellar/wget/.*/bin/wget
    "## }
}

#[test]
fn brew_qs() {
    test_dsl! { r##"
        in --using brew -Qs wget
        ou wget
    "## }
}

#[test]
#[ignore]
fn brew_r_s() {
    test_dsl! { r##"
        in --using brew -S screenfetch --yes
        ou Cellar/screenfetch
        in ! screenfetch --version
        ou Created by and licensed to Brett Bohnenkamper <kittykatt@kittykatt.us>
        in --using brew -R screenfetch --yes
        ou Uninstalling
        ou Cellar/screenfetch
    "## }
}

#[test]
fn brew_si() {
    test_dsl! { r##"
        in --using brew -Si curl
        ou Get a file from an HTTP, HTTPS or FTP server
    "## }
}

#[test]
fn brew_ss() {
    test_dsl! { r##"
        in --using brew -Ss wget
        ou wget
    "## }
}
