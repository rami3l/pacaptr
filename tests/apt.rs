#![cfg(all(unix, feature = "test"))]

mod common;
use common::*;

#[test]
#[should_panic(expected = "failed with pattern `^Package: wget$`")]
fn apt_fail() {
    test_dsl! { r##"
        in -Si fish
        ou ^Package: wget$
    "## }
}

#[test]
fn apt_q() {
    test_dsl! { r##"
        in -Q
        ou apt
    "## }
}

#[cfg(target_os = "linux")]
#[test]
fn apt_qc() {
    test_dsl! { r##"
        in -Qc wget
        ou ubuntu\.com
    "## }
}

#[test]
fn apt_qe() {
    test_dsl! { r##"
        in -Qe
        ou ^apt$
        in -Qe apt
        ou ^apt$
    "## }
}

#[test]
fn apt_qi() {
    test_dsl! { r##"
        in -Qi apt
        ou ^Package: apt$
        ou ^Status: install ok installed$
        ou ^Priority: (important|required)$
    "## }
}

#[test]
fn apt_qo() {
    test_dsl! { r##"
        in -Qo apt.8
        ou /share/man/man8/apt
    "## }
}

#[cfg(target_os = "linux")]
#[test]
fn apt_qp_sw() {
    test_dsl! { r##"
        in -Sw fortunes-min --yes
        ou download only mode
    "## }
}

#[cfg(target_os = "linux")]
#[test]
fn apt_qs() {
    test_dsl! { r##"
        in -Qs shell
        ou Bourne Again SHell
    "## }
}

#[test]
#[ignore]
fn apt_r_s() {
    test_dsl! { r##"
        # Update package databases
        in -Sy

        # Now installation
        in -S fish --yes
        in ! which fish
        ou /bin/fish

        # Now remove the package
        in -R fish --yes
        ou ^Removing fish
    "## }
}

#[test]
fn apt_si() {
    test_dsl! { r##"
        in -Si fish
        ou ^Package: fish$
    "## }
}

#[test]
fn apt_sii() {
    test_dsl! { r##"
        in -Sii fish
        ou ^Reverse Depends:
    "## }
}

#[test]
fn apt_ss() {
    test_dsl! { r##"
        in -Ss apt
        ou apt
        ou commandline package manager
    "## }
}
