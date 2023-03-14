#![cfg(unix)]

mod common;
use common::*;

#[test]
#[should_panic(expected = "Failed with pattern `^Package: wget$`")]
fn pkcon_fail() {
    test_dsl! { r##"
        in --using pkcon -Si fish
        ou ^Package: wget$
    "## }
}

#[test]
fn pkcon_q() {
    test_dsl! { r##"
        in --using pkcon -Q
        ou apt
    "## }
}

#[test]
fn pkcon_qs() {
    test_dsl! { r##"
        in --using pkcon -Qs apt
        ou Installed
    "## }
}

#[test]
#[ignore]
fn pkcon_r_s() {
    test_dsl! { r##"
        # Update package databases
        in --using pkcon -Sy

        # Now installation
        in --using pkcon -S fish --yes
        in ! which fish
        ou /bin/fish

        # Now remove the package
        in --using pkcon -R fish --yes
        ou Finished
    "## }
}

#[test]
fn pkcon_si() {
    test_dsl! { r##"
        in --using pkcon -Si wget
        ou retrieves files from the web
    "## }
}
