pub use self::utils::Test;

mod utils;

#[cfg(target_os = "macos")]
mod homebrew {
    use super::Test;

    #[test]
    fn si_ok() {
        Test::new()
            .pacaptr(&["-Si", "curl"], &[])
            .output(&["curl is keg-only"])
            .run(false)
    }

    #[test]
    #[should_panic(expected = "Failed with pattern `curl is not keg-only`")]
    fn si_fail() {
        Test::new()
            .pacaptr(&["-Si", "curl"], &[])
            .output(&["curl is not keg-only"])
            .run(false)
    }

    #[test]
    #[ignore]
    fn r() {
        Test::new()
            .pacaptr(&["-S", "wget", "--yes"], &[])
            .output(&["brew (re)?install wget"])
            .exec(&["wget", "-V"], &[])
            .output(&["GNU Wget"])
            .pacaptr(&["-R", "wget", "--yes"], &[])
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
            .pacaptr(&["-Si", "wget"], &[])
            .output(&["GNU Wget is a free software package"])
            .run(false)
    }

    #[test]
    #[should_panic(expected = "Failed with pattern `GNU Wget is not a free software package`")]
    fn si_fail() {
        Test::new()
            .pacaptr(&["-Si", "wget"], &[])
            .output(&["GNU Wget is not a free software package"])
            .run(false)
    }

    #[test]
    #[ignore]
    fn r() {
        Test::new()
            .pacaptr(&["-S", "wget", "--yes"], &[])
            .output(&["The install of wget was successful."])
            .pacaptr(&["-R", "wget", "--yes"], &[])
            .output(&["Wget has been successfully uninstalled."])
            .run(false)
    }
}

#[cfg(target_os = "windows")]
mod scoop {
    use super::Test;

    #[test]
    fn si_ok() {
        Test::new()
            .pacaptr(&["--using", "scoop", "-Si", "wget"], &[])
            .output(&["Description: A command-line utility for retrieving files using HTTP, HTTPS, FTP, and FTPS protocols."])
            .run(false)
    }

    #[test]
    #[should_panic(expected = "Failed with pattern `GNU Wget is not a free software package`")]
    fn si_fail() {
        Test::new()
            .pacaptr(&["--using", "scoop", "-Si", "wget"], &[])
            .output(&["GNU Wget is not a free software package"])
            .run(false)
    }

    #[test]
    #[ignore]
    fn r() {
        Test::new()
            .pacaptr(&["--using", "scoop", "-S", "wget", "--yes"], &[])
            .output(&["wget", "was installed successfully!"])
            .pacaptr(&["--using", "scoop", "-Q"])
            .output(&["wget", "[main]"])
            .pacaptr(&["--using", "scoop", "-R", "wget", "--yes"], &[])
            .output(&["wget", "was uninstalled."])
            .run(false)
    }
}

#[cfg(target_os = "linux")]
mod linuxbrew {
    use super::Test;

    #[test]
    fn si_ok() {
        Test::new()
            .pacaptr(&["-Si", "curl"], &[])
            .output(&["Get a file from an HTTP, HTTPS or FTP server"])
            .run(false)
    }

    #[test]
    #[ignore]
    fn r() {
        Test::new()
            .pacaptr(&["-S", "wget", "--yes"], &[])
            .output(&["brew (re)?install wget"])
            .exec(&["wget", "-V"], &[])
            .output(&["GNU Wget"])
            .pacaptr(&["-R", "wget", "--yes"], &[])
            .output(&["brew uninstall wget"])
            .run(false)
    }
}

#[cfg(target_os = "linux")]
mod apt {
    use super::Test;

    #[test]
    fn si_ok() {
        Test::new()
            .pacaptr(&["-Si", "screen"], &[])
            .output(&["Package: screen"])
            .run(false)
    }

    #[test]
    #[should_panic(expected = "Failed with pattern `Package: wget`")]
    fn si_fail() {
        Test::new()
            .pacaptr(&["-Si", "screen"], &[])
            .output(&["Package: wget"])
            .run(false)
    }

    #[test]
    #[ignore]
    fn r() {
        Test::new()
            .pacaptr(&["-S", "screen", "--yes"], &[])
            .output(&["apt(-get)? install", "--reinstall", "--yes", "screen"])
            .pacaptr(&["-Qi", "screen"], &[])
            .output(&["Status: install"])
            .pacaptr(&["-R", "screen", "--yes"], &[])
            .output(&["apt(-get)? remove", "--yes", "screen"])
            .pacaptr(&["-Qi", "screen"], &[])
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
            .pacaptr(&["-Si", "wget"], &[])
            .output(&["A network utility to retrieve files from the Web"])
            .run(false)
    }

    #[test]
    #[should_panic(expected = "Failed with pattern `Why not use curl instead?`")]
    fn si_fail() {
        Test::new()
            .pacaptr(&["-Si", "wget"], &[])
            .output(&["Why not use curl instead?"])
            .run(false)
    }

    #[test]
    #[ignore]
    fn r() {
        Test::new()
            .pacaptr(&["-S", "wget", "--yes"], &[])
            .output(&["Installing wget"])
            .exec(&["wget", "-V"], &[])
            .output(&["GNU Wget"])
            .pacaptr(&["-R", "wget", "--yes"], &[])
            .output(&["Purging wget"])
            .run(false)
    }
}

#[cfg(target_os = "linux")]
mod dnf {
    use super::Test;

    #[test]
    fn si_ok() {
        Test::new()
            .pacaptr(&["-Si", "curl"], &[])
            .output(&["A utility for getting files from remote servers"])
            .run(false)
    }

    #[test]
    #[should_panic(expected = "Failed with pattern `Why not use curl instead?`")]
    fn si_fail() {
        Test::new()
            .pacaptr(&["-Si", "wget"], &[])
            .output(&["Why not use curl instead?"])
            .run(false)
    }

    #[test]
    #[ignore]
    fn r() {
        Test::new()
            .pacaptr(&["-S", "wget", "--yes"], &[])
            .output(&["dnf install", "-y", "wget", "Installed:", "Complete!"])
            .exec(&["wget", "-V"], &[])
            .output(&["GNU Wget"])
            .pacaptr(&["-R", "wget", "--yes"], &[])
            .output(&["dnf remove", "-y", "wget", "Removed:", "Complete!"])
            .run(false)
    }
}

#[cfg(target_os = "linux")]
mod zypper {
    use super::Test;

    #[test]
    fn si_ok() {
        Test::new()
            .pacaptr(&["-Si", "curl"], &[])
            .output(&["A Tool for Transferring Data from URLs"])
            .run(false)
    }

    #[test]
    #[should_panic(expected = "Failed with pattern `Why not use curl instead?`")]
    fn si_fail() {
        Test::new()
            .pacaptr(&["-Si", "wget"], &[])
            .output(&["Why not use curl instead?"])
            .run(false)
    }

    #[test]
    #[ignore]
    fn r() {
        Test::new()
            .pacaptr(&["-S", "wget", "--yes"], &[])
            .output(&["zypper install", "-y", "wget", "Installing: wget"])
            .exec(&["wget", "-V"], &[])
            .output(&["GNU Wget"])
            .pacaptr(&["-R", "wget", "--yes"], &[])
            .output(&["zypper remove", "-y", "wget", "Removing wget"])
            .run(false)
    }
}
