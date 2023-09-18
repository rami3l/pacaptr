#![cfg(all(windows, feature = "test"))]

mod common;
use common::*;

#[test]
#[should_panic(expected = "Failed with pattern `GNU Wget is not a free software package`")]
fn winget_fail() {
    test_dsl! { r##"
        in --using winget -Si wget
        ou GNU Wget is not a free software package
    "## }
}

#[test]
fn winget_qi() {
    test_dsl! { r##"
        in --using winget -Qi wget
        ou utility for retrieving files using HTTP, HTTPS, FTP and FTPS
    "## }
}

#[test]
#[ignore]
fn winget_r_s() {
    test_dsl! { r##"
        in --using winget -S wget --yes
        ou Found Wget
        ou JernejSimoncic\.Wget
        ou Successfully installed
        in --using winget -Q
        ou Wget
        ou JernejSimoncic\.Wget
        in --using winget -R wget --yes
        ou Found Wget
        ou JernejSimoncic\.Wget
        ou Successfully uninstalled
    "## }
}

#[test]
fn winget_si() {
    test_dsl! { r##"
        in --using winget -Si wget
        ou utility for retrieving files using HTTP, HTTPS, FTP and FTPS
    "## }
}

#[test]
fn winget_ss() {
    test_dsl! { r##"
        in --using winget -Ss wget
        ou Wget
        ou JernejSimoncic\.Wget
    "## }
}
