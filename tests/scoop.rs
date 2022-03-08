#![cfg(target_os = "windows")]

mod common;
use common::*;

#[test]
#[should_panic(expected = "Failed with pattern `GNU Wget is not a free software package`")]
fn scoop_fail() {
    test_dsl! { r##"
        in --using scoop -Si wget
        ou GNU Wget is not a free software package
    "## }
}

#[test]
fn scoop_qi() {
    test_dsl! { r##"
        in --using scoop -Qi wget
        ou A command-line utility for retrieving files using HTTP, HTTPS, FTP, and FTPS protocols.
    "## }
}

#[test]
#[ignore]
fn scoop_r_s() {
    test_dsl! { r##"
        in --using scoop -S wget --yes
        ou wget
        ou was installed successfully!
        in --using scoop -Q
        ou wget
        ou [main]
        in --using scoop -R wget --yes
        ou wget
        ou was uninstalled.
    "## }
}

#[test]
fn scoop_si() {
    test_dsl! { r##"
        in --using scoop -Si wget
        ou Description: A command-line utility for retrieving files using HTTP, HTTPS, FTP, and FTPS protocols.
    "## }
}

#[test]
fn scoop_ss() {
    test_dsl! { r##"
        in --using scoop -Ss wget
        ou wget \(.+\)
    "## }
}
