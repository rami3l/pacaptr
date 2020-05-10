pub use self::utils::Test;

mod utils;

#[cfg(target_os = "macos")]
mod homebrew {
    use super::Test;

    #[test]
    fn si_ok() {
        Test::new()
            .pacaptr(&["-Si", "curl"])
            .output(&["curl is keg-only"])
            .run(false)
    }

    #[test]
    #[should_panic(expected = "Failed with pattern `curl is not keg-only`")]
    fn si_fail() {
        Test::new()
            .pacaptr(&["-Si", "curl"])
            .output(&["curl is not keg-only"])
            .run(false)
    }

    #[test]
    fn s_auto_cask() {
        Test::new()
            .pacaptr(&["-S", "curl", "gimp", "--dryrun"])
            .output(&["brew (re)?install curl", "brew cask (re)?install gimp"])
            .run(false)
    }

    #[test]
    fn s_force_cask() {
        Test::new()
            .pacaptr(&["-S", "docker", "--dryrun"])
            .output(&["brew (re)?install docker"])
            .pacaptr(&["-S", "docker", "--cask", "--dryrun"])
            .output(&["brew cask (re)?install docker"])
            .run(false)
    }

    #[test]
    fn r_cask() {
        Test::new()
            .pacaptr(&["-R", "curl", "gimp", "--dryrun"])
            .output(&["brew uninstall curl", "brew cask uninstall gimp"])
            .run(false)
    }

    #[test]
    #[ignore]
    fn r() {
        Test::new()
            .pacaptr(&["-S", "wget", "--yes"])
            .output(&["brew (re)?install wget"])
            .exec("wget", &["-V"], &[])
            .output(&["GNU Wget"])
            .pacaptr(&["-R", "wget", "--yes"])
            .output(&["brew uninstall wget", "Uninstalling /usr/local/Cellar/wget"])
            .run(false)
    }
}

#[cfg(target_os = "windows")]
mod chocolatey {
    use super::Test;

    #[test]
    fn si_ok() {
        Test::new()
            .pacaptr(&["-Si", "wget"])
            .output(&["GNU Wget is a free software package"])
            .run(false)
    }

    #[test]
    #[should_panic(expected = "Failed with pattern `GNU Wget is not a free software package`")]
    fn si_fail() {
        Test::new()
            .pacaptr(&["-Si", "wget"])
            .output(&["GNU Wget is not a free software package"])
            .run(false)
    }

    #[test]
    #[ignore]
    fn r() {
        Test::new()
            .pacaptr(&["-S", "wget", "--yes"])
            .output(&["The install of wget was successful."])
            .pacaptr(&["-R", "wget", "--yes"])
            .output(&["Wget has been successfully uninstalled."])
            .run(false)
    }
}

#[cfg(target_os = "linux")]
mod dpkg {
    use super::Test;

    #[test]
    fn si_ok() {
        Test::new()
            .pacaptr(&["-Si", "screen"])
            .output(&["Package: screen"])
            .run(false)
    }

    #[test]
    #[should_panic(expected = "Failed with pattern `Package: wget`")]
    fn si_fail() {
        Test::new()
            .pacaptr(&["-Si", "screen"])
            .output(&["Package: wget"])
            .run(false)
    }

    #[test]
    #[ignore]
    fn r() {
        Test::new()
            .pacaptr(&["-S", "screen", "--yes"])
            .output(&["apt-get install --yes screen"])
            .pacaptr(&["-Qi", "screen"])
            .output(&["Status: install"])
            .pacaptr(&["-R", "screen", "--yes"])
            .output(&["apt-get remove --yes screen"])
            .pacaptr(&["-Qi", "screen"])
            .output(&["Status: deinstall"])
            .run(false)
    }
}

#[cfg(target_os = "linux")]
mod apk {
    use super::Test;

    #[test]
    fn si_ok() {
        Test::new()
            .pacaptr(&["-Si", "wget"])
            .output(&["A network utility to retrieve files from the Web"])
            .run(false)
    }

    #[test]
    #[should_panic(expected = "Failed with pattern `Why not use curl instead?`")]
    fn si_fail() {
        Test::new()
            .pacaptr(&["-Si", "wget"])
            .output(&["Why not use curl instead?"])
            .run(false)
    }

    #[test]
    #[ignore]
    fn r() {
        Test::new()
            .pacaptr(&["-S", "wget", "--yes"])
            .output(&["Installing wget"])
            .exec("wget", &["-V"], &[])
            .output(&["GNU Wget"])
            .pacaptr(&["-R", "wget", "--yes"])
            .output(&["Purging wget"])
            .run(false)
    }
}
