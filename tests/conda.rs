mod common;
use common::*;

#[test]
#[should_panic(expected = "Failed with pattern `Why not use curl instead?`")]
fn conda_fail() {
    test_dsl! { r##"
        in --using conda -Si sympy
        ou Why not use curl instead?
    "## }
}

#[test]
fn conda_q() {
    test_dsl! { r##"
        in --using conda -Q
        ou python
        in --using conda -Q python
        ou python
    "## }
}

#[test]
fn conda_qs() {
    test_dsl! { r##"
        in --using conda -Qs python
        ou python
    "## }
}

#[test]
#[ignore]
fn conda_r_s() {
    test_dsl! { r##"
        in --using conda -S sympy --yes
        ou Executing transaction:
        ou done
        in --using conda -Q
        ou sympy
        in --using conda -R sympy --yes
        ou Executing transaction:
        ou done
    "## }
}

#[test]
fn conda_si() {
    test_dsl! { r##"
        in --using conda -Si sympy
        ou https://repo.anaconda.com/pkgs
    "## }
}

#[test]
fn conda_ss() {
    test_dsl! { r##"
        in --using conda -Ss sympy
        ou pkgs/main
    "## }
}
