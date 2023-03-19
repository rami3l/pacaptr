#![cfg(all(target_os = "linux", feature = "test"))]

mod common;
use common::*;

#[test]
#[should_panic(expected = "Failed with pattern `Why not use curl instead?`")]
fn dnf_fail() {
    test_dsl! { r##"
        in -Si wget
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
        ou redhat.com
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
        in -Sw wget --yes
        ou The downloaded packages were saved in cache
        in -Qp /var/cache/dnf/*/packages/wget-*.rpm
        ou wget
        ou A utility for retrieving files using the HTTP or FTP protocols
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

#[test]
fn dnf_si() {
    test_dsl! { r##"
        in -Si wget
        ou A utility for retrieving files using the HTTP or FTP protocols
    "## }
}

#[test]
fn dnf_sii() {
    test_dsl! { r##"
        in -Sii wget
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
        in -Sl wget
        ou Available Packages
        ou wget
    "## }
}

#[test]
fn dnf_ss() {
    test_dsl! { r##"
        in -Ss wget
        ou Name Exactly Matched: wget
        ou A utility for retrieving files using the HTTP or FTP protocols
    "## }
}
