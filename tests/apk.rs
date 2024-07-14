#![cfg(all(target_os = "linux", feature = "test"))]

mod common;
use common::*;

#[test]
#[should_panic(expected = "failed with pattern `Why not use curl instead?`")]
fn apk_fail() {
    test_dsl! { r##"
        in -Si wget
        ou Why not use curl instead?
    "## }
}

#[test]
fn apk_q() {
    test_dsl! { r##"
        in -Q
        ou busybox
    "## }
}

#[test]
fn apk_ql() {
    test_dsl! { r##"
        in -Ql busybox
        ou bin/busybox
    "## }
}

#[test]
fn apk_qo() {
    test_dsl! { r##"
        in -Qo /usr/bin/vi
        ou symlink target is owned by busybox
    "## }
}

#[test]
fn apk_qs() {
    test_dsl! { r##"
        in -Qs busybox
        ou busybox
        # ou Size optimized toolbox of many common UNIX utilities
    "## }
}

#[test]
#[ignore]
fn apk_r_s() {
    test_dsl! { r##"
        in -S wget --yes
        ou Installing wget
        in ! wget -V
        ou GNU Wget
        in -R wget --yes
        ou Purging wget
    "## }
}

#[test]
fn apk_si() {
    test_dsl! { r##"
        in -Si wget
        ou Network utility to retrieve files from the Web
    "## }
}

#[test]
fn apk_ss() {
    test_dsl! { r##"
        in -Ss wget
        # `wget-1.21.1-r1`
        ou wget-.*-r
    "## }
}
