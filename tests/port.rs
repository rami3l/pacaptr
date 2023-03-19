//! A big part of these tests are copied from https://guide.macports.org/#using.port.installed.

#![cfg(all(target_os = "macos", feature = "test"))]

mod common;
use common::*;

#[test]
#[should_panic(expected = "Failed with pattern `curl is not keg-only`")]
fn port_fail() {
    test_dsl! { r##"
        in --using port -Si curl
        ou curl is not keg-only
    "## }
}

#[test]
fn port_q() {
    test_dsl! { r##"
        in --using port -Q
        ou The following ports are currently installed:
        in --using port -Q wget
        ou The following ports are currently installed:
        ou wget @
    "## }
}

#[test]
fn port_qi() {
    test_dsl! { r##"
        in --using port -Qi yubico-pam
        ou The Yubico PAM module provides an easy way
    "## }
}

#[test]
fn port_ql() {
    test_dsl! { r##"
        in --using port -Ql wget
        ou .*/bin/wget
    "## }
}

#[test]
fn port_qs() {
    test_dsl! { r##"
        in --using port -Qs wget
        ou The following ports are currently installed:
        ou wget @
    "## }
}

#[test]
#[ignore]
fn port_r_s() {
    test_dsl! { r##"
        in --using port -S curl --yes
        ou --->  Installing curl
        in ! curl -V
        ou libcurl
        ou Protocols:
        in --using port -R curl --yes
        ou Uninstalling curl
    "## }
}

#[test]
fn port_si() {
    test_dsl! { r##"
        in --using port -Si yubico-pam
        ou The Yubico PAM module provides an easy way
    "## }
}

#[test]
fn port_ss() {
    test_dsl! { r##"
        in --using port -Ss wget
        ou wget
    "## }
}
