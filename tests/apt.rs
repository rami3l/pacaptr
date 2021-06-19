#![cfg(target_os = "linux")]

mod common;
use common::*;
use pacaptr_macros::test_dsl;

#[test]
fn apt_si_ok() {
    Test::new()
        .pacaptr(&["-Si", "screen"], &[])
        .output(&["Package: screen"])
        .run()
}

#[test]
#[should_panic(expected = "Failed with pattern `Package: wget`")]
fn apt_si_fail() {
    Test::new()
        .pacaptr(&["-Si", "screen"], &[])
        .output(&["Package: wget"])
        .run()
}

/*
#[test]
#[ignore]
fn apt_r() {
    Test::new()
        .pacaptr(&["-S", "screen", "--yes"], &[])
        .output(&["apt(-get)? install", "--reinstall", "--yes", "screen"])
        .pacaptr(&["-Qi", "screen"], &[])
        .output(&["Status: install"])
        .pacaptr(&["-R", "screen", "--yes"], &[])
        .output(&["apt(-get)? remove", "--yes", "screen"])
        .pacaptr(&["-Qi", "screen"], &[])
        .output(&["Status: deinstall"])
        .run()
}
*/

#[test]
#[ignore]
fn apt_long() {
    // The following does not work because macros are lazy (for now)...
    // test_dsl!(include_str!("./tests/data/apt.txt"))
    test_dsl! {
        r##"
        # Update package databases
        in -Sy

        # Simple query that lists all packages
        # in -Q
        # ou ^apt$

        # Information of `apt`
        in -Qi apt
        ou ^Package: apt$
        ou ^Status: install ok installed$
        ou ^Priority: (important|required)$

        # Install and Deinstall a package

        # Display remote package information
        in -Si screen
        ou ^Package: screen

        in -Sii screen
        ou ^Reverse Depends:

        # Now installation
        in -S screen --yes
        in ! which screen
        ou ^/usr/bin/screen

        # Now remove the package
        in -R screen --yes
        in -Qi screen
        ou ^Status: deinstall
        "##
    }
}
