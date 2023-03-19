#![cfg(all(target_os = "linux", feature = "test"))]

mod common;
use common::*;

#[test]
#[should_panic(expected = "Failed with pattern `Why not use curl instead?`")]
fn emerge_fail() {
    test_dsl! { r##"
        in -Si wget
        ou Why not use curl instead?
    "## }
}

#[test]
fn emerge_q() {
    test_dsl! { r##"
        in -Q
        ou net-misc/wget
        in -Q wget
        ou net-misc/wget
    "## }
}

#[test]
fn emerge_ql() {
    test_dsl! { r##"
        in -Ql wget
        ou /usr/bin/wget
    "## }
}

#[test]
fn emerge_qo() {
    test_dsl! { r##"
        in -Qo /usr/bin/wget
        ou net-misc/wget
    "## }
}

#[test]
fn emerge_qs() {
    test_dsl! { r##"
        in -Qs wget
        ou net-misc/wget
    "## }
}

#[test]
#[ignore]
fn emerge_r_s() {
    test_dsl! { r##"
        in -S screenfetch --yes
        ou >>> Installing \(.* of .*\) app-misc/screenfetch-
        in ! screenfetch -V
        ou <kittykatt@kittykatt.us>
        in -R screenfetch --yes
        ou >>> Unmerging \(.* of .*\) app-misc/screenfetch-
    "## }
}

#[test]
fn emerge_si() {
    test_dsl! { r##"
        in -Si wget
        ou net-misc/wget
        ou Network utility to retrieve files from the WWW
    "## }
}

#[test]
fn emerge_ss() {
    test_dsl! { r##"
        in -Ss wget
        ou net-misc/wget: Network utility to retrieve files from the WWW
    "## }
}
