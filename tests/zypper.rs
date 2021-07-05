#![cfg(target_os = "linux")]

mod common;
use common::*;

#[test]
#[should_panic(expected = "Failed with pattern `Why not use curl instead?`")]
fn zypper_fail() {
    test_dsl! { r##"
        in -Si wget
        ou Why not use curl instead?
    "## }
}

#[test]
fn zypper_q() {
    test_dsl! { r##"
        in -Q
        ou ^zypper
    "## }
}

#[test]
fn zypper_qc() {
    test_dsl! { r##"
        in -Qc zypper
        ou suse.de
    "## }
}

#[test]
fn zypper_qi() {
    test_dsl! { r##"
        in -Qi curl
        ou A Tool for Transferring Data from URLs
"## }
}

#[test]
fn zypper_ql() {
    test_dsl! { r##"
        in -Ql rpm
        ou /usr/bin/rpm
    "## }
}

#[test]
fn zypper_qo() {
    test_dsl! { r##"
        in -Qo /usr/bin/zypper
        ou zypper
    "## }
}

#[test]
fn zypper_qp_sw() {
    test_dsl! { r##"
        in -Sw wget --yes
        ou Download only.
        in ! ls /var/cache/zypp/packages/*/*
        in -Qp /var/cache/zypp/packages/*/*/libcares2-*.rpm
        ou Library for asynchronous name resolves
    "## }
}

#[test]
fn zypper_qs() {
    test_dsl! { r##"
        in -Qs zypper
        ou zypper
        ou Command line software manager using libzypp
    "## }
}

#[test]
#[ignore]
fn zypper_r_s() {
    test_dsl! { r##"
        in -S wget --yes
        ou Installing: wget
        in ! wget -V
        ou GNU Wget
        in -R wget --yes
        ou Removing wget
    "## }
}

#[test]
fn zypper_sg() {
    test_dsl! { r##"
        in -Sg
        ou ^  | base
        ou ^  | console
        in -Sg console
        ou ^  | patterns-base-console
        ou ^  | tmux
    "## }
}

#[test]
fn zypper_si() {
    test_dsl! { r##"
        in -Si curl
        ou A Tool for Transferring Data from URLs
    "## }
}

#[test]
fn zypper_sl() {
    test_dsl! { r##"
        in -Sl
        ou Main Repository
        ou wget
        in -Sl wget
        ou Main Repository
        ou wget
    "## }
}

#[test]
fn zypper_ss() {
    test_dsl! { r##"
        in -Ss wget
        ou wget
        ou A Tool for Mirroring FTP and HTTP
    "## }
}
