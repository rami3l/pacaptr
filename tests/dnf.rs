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
        in -Qc dnf5
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
        in -Qi dnf5
        ou Command-line package manager
    "## }
}

#[test]
fn dnf_ql() {
    test_dsl! { r##"
        in -Ql dnf5
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
        ou wget2
        in -Qp /var/cache/*dnf*/*/packages/wget2-*.rpm
        ou wget
        ou file and recursive website downloader
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
#[ignore = "heavy test"]
fn dnf_r_s() {
    test_dsl! { r##"
        in -S wget2 --yes
        ou Installing
        in ! wget2 -V
        ou GNU Wget
        in -R wget2 --yes
        ou Removing
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
        in -Sii curl
        ou rpm
    "## }
}

#[test]
fn dnf_sg() {
    test_dsl! { r##"
        in -Sg
        ou LibreOffice
        in -Sg LibreOffice
        ou LibreOffice Productivity Suite
        ou libreoffice-writer
    "## }
}

#[test]
fn dnf_sl() {
    test_dsl! { r##"
        in -Sl wget2
        ou Available packages
        ou wget
    "## }
}

#[test]
fn dnf_ss() {
    test_dsl! { r##"
        in -Ss wget
        ou An advanced file and recursive website downloader
    "## }
}
