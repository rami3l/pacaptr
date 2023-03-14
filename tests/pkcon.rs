#![cfg(unix)]

mod common;
use common::*;

#[test]
#[should_panic(expected = "Failed with pattern `^Package: wget$`")]
fn pkcon_fail() {
    test_dsl! { r##"
        in -Si fish
        ou ^Package: wget$
    "## }
}

#[test]
fn pkcon_q() {
    test_dsl! { r##"
        in -Q
        ou apt
    "## }
}

#[test]
fn pkcon_qi() {
    test_dsl! { r##"
        in -Qi apt
        ou Installed
    "## }
}

#[test]
#[ignore]
fn pkcon_r_s() {
    test_dsl! { r##"
        # Update package databases
        in -Sy

        # Now installation
        in -S fish --yes
        in ! which fish
        ou /bin/fish

        # Now remove the package
        in -R fish --yes
        ou Finished
    "## }
}

#[test]
fn pkcon_si() {
    test_dsl! { r##"
        in -Si wget
        ou retrieves files from the web
    "## }
}
