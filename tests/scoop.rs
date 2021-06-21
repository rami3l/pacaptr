#![cfg(target_os = "windows")]

mod common;
use common::*;

#[test]
fn scoop_si_ok() {
    test_dsl! { r##"
        in --using scoop -Si wget
        ou Description: A command-line utility for retrieving files using HTTP, HTTPS, FTP, and FTPS protocols.
    "## }
}

#[test]
#[should_panic(expected = "Failed with pattern `GNU Wget is not a free software package`")]
fn scoop_si_fail() {
    test_dsl! { r##"
        in --using scoop -Si wget
        ou GNU Wget is not a free software package
    "## }
}

#[test]
#[ignore]
fn scoop_r() {
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
