#![cfg(target_os = "linux")]

mod common;
use common::*;

#[test]
fn apt_si_ok() {
    test_dsl! { r##"
        # Information of `screen`
        in -Si screen
        ou ^Package: screen$
    "## }
}

#[test]
#[should_panic(expected = "Failed with pattern `^Package: wget$`")]
fn apt_si_fail() {
    test_dsl! { r##"
        # Information of `screen`
        in -Si screen
        ou ^Package: wget$
    "## }
}

#[test]
fn apt_qi() {
    test_dsl! { r##"
        # Information of `apt`
        in -Qi apt
        ou ^Package: apt$
        ou ^Status: install ok installed$
        ou ^Priority: (important|required)$
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
fn apt_q() {
    test_dsl! { r##"
        # Simple query that lists all packages
        in -Q
        ou apt
    "## }
}

#[test]
#[ignore]
fn apt_r() {
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
