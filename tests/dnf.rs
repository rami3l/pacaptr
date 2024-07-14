#![cfg(all(target_os = "linux", feature = "test"))]

mod common;
use common::*;

#[test]
#[should_panic(expected = "failed with pattern `Why not use curl instead?`")]
fn dnf_fail() {
    test_dsl! { r##"
        in -Si wget2
        ou Why not use curl instead?
    "## }
}

#[test]
fn dnf_q() {
    test_dsl! { r##"
        in -Q
        ou dnf
    "## }
}

#[test]
fn dnf_qc() {
    test_dsl! { r##"
        in -Qc dnf
        ou redhat\.com
    "## }
}

#[test]
fn dnf_qe() {
    test_dsl! { r##"
        in -Qe
        ou dnf
    "## }
}

#[test]
fn dnf_qi() {
    test_dsl! { r##"
        in -Qi dnf
        ou Utility that allows users to manage packages on their systems.
    "## }
}

#[test]
fn dnf_ql() {
    test_dsl! { r##"
        in -Ql dnf
        ou /usr/share/man/man8/dnf.8.gz
    "## }
}

#[test]
fn dnf_qo() {
    test_dsl! { r##"
        in -Qo /usr/bin/dnf
        ou dnf
    "## }
}

#[test]
fn dnf_qp_sw() {
    test_dsl! { r##"
        in -Sw wget2 --yes
        ou The downloaded packages were saved in cache
        in -Qp /var/cache/dnf/*/packages/wget2-*.rpm
        ou wget
        ou An advanced file and recursive website downloader
    "## }
}

#[test]
fn dnf_qs() {
    test_dsl! { r##"
        in -Qs dnf
        ou dnf
    "## }
}

#[test]
#[ignore]
fn dnf_r_s() {
    test_dsl! { r##"
        in -S wget2 --yes
        ou Installed:
        ou Complete!
        in ! wget2 -V
        ou GNU Wget
        in -R wget2 --yes
        ou Removed:
        ou Complete!
    "## }
}

#[test]
fn dnf_si() {
    test_dsl! { r##"
        in -Si wget2
        ou An advanced file and recursive website downloader
    "## }
}

#[test]
fn dnf_sii() {
    test_dsl! { r##"
        in -Sii wget2
        ou package: wget
        ou dependency: libc.so
    "## }
}

#[test]
fn dnf_sg() {
    test_dsl! { r##"
        in -Sg
        ou Available Groups:
        ou LibreOffice
        in -Sg LibreOffice
        ou Group: LibreOffice
        ou Description: LibreOffice Productivity Suite
        ou libreoffice-writer
    "## }
}

#[test]
fn dnf_sl() {
    test_dsl! { r##"
        in -Sl wget2
        ou Available Packages
        ou wget
    "## }
}

#[test]
fn dnf_ss() {
    test_dsl! { r##"
        in -Ss wget
        ou Matched: wget
        ou An advanced file and recursive website downloader
    "## }
}
