#![cfg(all(target_os = "linux", feature = "test"))]

mod common;
use common::*;

#[test]
#[should_panic(expected = "failed with pattern `^pkgname: wget$`")]
fn xbps_fail() {
    test_dsl! {"
        in -Si fish-shell
        ou ^pkgname: wget$
    "}
}

#[test]
fn xbps_q() {
    test_dsl! {"
        in -Q
        ou ^ii xbps
    "}
}

#[test]
fn xbps_qe() {
    test_dsl! {"
        in -Qe
        ou ^base
    "}
}

#[test]
fn xbps_qi() {
    test_dsl! {"
        in -Qi xbps
        ou ^pkgname: xbps
        ou ^state: installed
    "}
}

#[test]
fn xbps_r_s() {
    test_dsl! {"
        # Update package databases
        in -Sy

        # Now installation
        in -S fish-shell --yes
        in ! which fish
        ou bin/fish

        # Now remove the package
        in -R fish-shell --yes
        ou Removing `fish
    "}
}

#[test]
fn xbps_si() {
    test_dsl! {"
        in -Si fish-shell
        ou ^pkgname: fish-shell
    "}
}

#[test]
fn xbps_sii() {
    test_dsl! {"
        in -Sii xbps
        ou base-system
    "}
}

#[test]
fn xbps_ss() {
    test_dsl! {"
        in -Ss xbps
        ou xbps
        ou XBPS package system
    "}
}
