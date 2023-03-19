#![cfg(feature = "test")]

mod common;
use common::*;

#[test]
#[should_panic(expected = "Failed with pattern `Here comes the sun`")]
fn pip_fail() {
    test_dsl! { r##"
        in --using pip -Qi wheel
        ou Here comes the sun
    "## }
}

#[test]
fn pip_q() {
    test_dsl! { r##"
        in --using pip -Q
        ou wheel
        in --using pip -Q wheel
        ou wheel
    "## }
}

#[test]
fn pip_qi() {
    test_dsl! { r##"
        in --using pip -Qi wheel
        ou Summary: A built-package format for Python
    "## }
}

#[test]
fn pip_qs() {
    test_dsl! { r##"
        in --using pip -Qs wheel
        ou wheel
    "## }
}

#[test]
#[ignore]
fn pip_r_s() {
    test_dsl! { r##"
        in --using pip -S sphinx --yes
        ou Successfully installed
        in --using pip -Q
        ou [Ss]phinx
        in --using pip -R sphinx --yes
        ou Successfully uninstalled
    "## }
}
