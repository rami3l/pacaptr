#![cfg(unix)]

mod common;
use common::*;

#[test]
#[should_panic(expected = "Failed with pattern `^Package: wget$`")]
fn apt_fail() {
    test_dsl! { r##"
        in -Si screen
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

#[test]
fn apt_qc() {
    test_dsl! { r##"
        in -Qc wget
        ou @ubuntu.com
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

#[test]
fn apt_qp_sw() {
    test_dsl! { r##"
        in -Sw screenfetch --yes
        ou download only mode
        in -Qp /var/cache/apt/archives/screenfetch_*.deb
        ou Package: screenfetch
    "## }
}

#[test]
#[ignore]
fn apt_r_s() {
    test_dsl! { r##"
        # Update package databases
        in -Sy

        # Now installation
        in -S screen --yes
        in ! which screen
        ou ^/usr/bin/screen

        # Now remove the package
        in -R screen --yes
        in -Qi screen
        ou ^Status: deinstall
    "## }
}

#[cfg(target_os = "linux")]
#[test]
fn apt_sg() {
    test_dsl! { r##"
        in -S --noconfirm tasksel
        in -Sg
        ou ^u ubuntu-desktop
        in -Sg ubuntu-desktop
        ou ubuntu-desktop\^
    "## }
}

#[test]
fn apt_si() {
    test_dsl! { r##"
        in -Si screen
        ou ^Package: screen$
    "## }
}

#[test]
fn apt_sii() {
    test_dsl! { r##"
        in -Sii screen
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
