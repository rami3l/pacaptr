#![cfg(target_os = "linux")]

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
        ou ^apt: /usr/share/man/man8/apt.8.gz$
    "## }
}

#[test]
fn apt_qp_sw() {
    test_dsl! { r##"
        # Information of `apt`
        in -Sw screenfetch --yes
        ou ^apt: /usr/share/man/man8/apt.8.gz$
        in -Qp /var/cache/apt/archives/screenfetch_*.deb
        ou ^Package: screenfetch$
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

#[test]
fn apt_si() {
    test_dsl! { r##"
        # Information of `screen`
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
