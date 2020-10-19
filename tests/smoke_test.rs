pub use self::utils::Test;

mod utils;

#[cfg(target_os = "macos")]
mod homebrew {
    use super::Test;
    use tokio::test;

    #[test]
    async fn si_ok() {
        Test::new()
            .pacaptr(&["-Si", "curl"], &[])
            .output(&["curl is keg-only"])
            .run(false)
            .await
    }

    #[test]
    #[should_panic(expected = "Failed with pattern `curl is not keg-only`")]
    async fn si_fail() {
        Test::new()
            .pacaptr(&["-Si", "curl"], &[])
            .output(&["curl is not keg-only"])
            .run(false)
            .await
    }

    #[test]
    async fn s_auto_cask() {
        Test::new()
            .pacaptr(&["-S", "curl", "gimp", "--dryrun"], &[])
            .output(&["brew (re)?install curl", "brew cask (re)?install gimp"])
            .run(false)
            .await
    }

    #[test]
    async fn s_force_cask() {
        Test::new()
            .pacaptr(&["-S", "docker", "--dryrun"], &[])
            .output(&["brew (re)?install docker"])
            .pacaptr(&["-S", "docker", "--cask", "--dryrun"], &[])
            .output(&["brew cask (re)?install docker"])
            .run(false)
            .await
    }

    #[test]
    async fn r_cask() {
        Test::new()
            .pacaptr(&["-R", "curl", "gimp", "--dryrun"], &[])
            .output(&["brew uninstall curl", "brew cask uninstall gimp"])
            .run(false)
            .await
    }

    #[test]
    #[ignore]
    async fn r() {
        Test::new()
            .pacaptr(&["-S", "wget", "--yes"], &[])
            .output(&["brew (re)?install wget"])
            .exec(&["wget", "-V"], &[])
            .output(&["GNU Wget"])
            .pacaptr(&["-R", "wget", "--yes"], &[])
            .output(&["brew uninstall wget", "Uninstalling /usr/local/Cellar/wget"])
            .run(false)
            .await
    }
}

#[cfg(target_os = "windows")]
mod chocolatey {
    use super::Test;
    use tokio::test;

    #[test]
    async fn si_ok() {
        Test::new()
            .pacaptr(&["-Si", "wget"], &[])
            .output(&["GNU Wget is a free software package"])
            .run(false)
            .await
    }

    #[test]
    #[should_panic(expected = "Failed with pattern `GNU Wget is not a free software package`")]
    async fn si_fail() {
        Test::new()
            .pacaptr(&["-Si", "wget"], &[])
            .output(&["GNU Wget is not a free software package"])
            .run(false)
            .await
    }

    #[test]
    #[ignore]
    async fn r() {
        Test::new()
            .pacaptr(&["-S", "wget", "--yes"], &[])
            .output(&["The install of wget was successful."])
            .pacaptr(&["-R", "wget", "--yes"], &[])
            .output(&["Wget has been successfully uninstalled."])
            .run(false)
            .await
    }
}

#[cfg(target_os = "linux")]
mod linuxbrew {
    use super::Test;
    use tokio::test;

    #[test]
    async fn si_ok() {
        Test::new()
            .pacaptr(&["-Si", "curl"], &[])
            .output(&["Get a file from an HTTP, HTTPS or FTP server"])
            .run(false)
            .await
    }

    #[test]
    #[ignore]
    async fn r() {
        Test::new()
            .pacaptr(&["-S", "wget", "--yes"], &[])
            .output(&["brew (re)?install wget"])
            .exec(&["wget", "-V"], &[])
            .output(&["GNU Wget"])
            .pacaptr(&["-R", "wget", "--yes"], &[])
            .output(&["brew uninstall wget"])
            .run(false)
            .await
    }
}

#[cfg(target_os = "linux")]
mod apt {
    use super::Test;
    use tokio::test;

    #[test]
    async fn si_ok() {
        Test::new()
            .pacaptr(&["-Si", "screen"], &[])
            .output(&["Package: screen"])
            .run(false)
            .await
    }

    #[test]
    #[should_panic(expected = "Failed with pattern `Package: wget`")]
    async fn si_fail() {
        Test::new()
            .pacaptr(&["-Si", "screen"], &[])
            .output(&["Package: wget"])
            .run(false)
            .await
    }

    #[test]
    #[ignore]
    async fn r() {
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
            .await
    }
}

#[cfg(target_os = "linux")]
mod apk {
    use super::Test;
    use tokio::test;

    #[test]
    async fn si_ok() {
        Test::new()
            .pacaptr(&["-Si", "wget"], &[])
            .output(&["A network utility to retrieve files from the Web"])
            .run(false)
            .await
    }

    #[test]
    #[should_panic(expected = "Failed with pattern `Why not use curl instead?`")]
    async fn si_fail() {
        Test::new()
            .pacaptr(&["-Si", "wget"], &[])
            .output(&["Why not use curl instead?"])
            .run(false)
            .await
    }

    #[test]
    #[ignore]
    async fn r() {
        Test::new()
            .pacaptr(&["-S", "wget", "--yes"], &[])
            .output(&["Installing wget"])
            .exec(&["wget", "-V"], &[])
            .output(&["GNU Wget"])
            .pacaptr(&["-R", "wget", "--yes"], &[])
            .output(&["Purging wget"])
            .run(false)
            .await
    }
}

#[cfg(target_os = "linux")]
mod dnf {
    use super::Test;
    use tokio::test;

    #[test]
    async fn si_ok() {
        Test::new()
            .pacaptr(&["-Si", "curl"], &[])
            .output(&["A utility for getting files from remote servers"])
            .run(false)
            .await
    }

    #[test]
    #[should_panic(expected = "Failed with pattern `Why not use curl instead?`")]
    async fn si_fail() {
        Test::new()
            .pacaptr(&["-Si", "wget"], &[])
            .output(&["Why not use curl instead?"])
            .run(false)
            .await
    }

    #[test]
    #[ignore]
    async fn r() {
        Test::new()
            .pacaptr(&["-S", "wget", "--yes"], &[])
            .output(&["dnf install", "-y", "wget", "Installed:", "Complete!"])
            .exec(&["wget", "-V"], &[])
            .output(&["GNU Wget"])
            .pacaptr(&["-R", "wget", "--yes"], &[])
            .output(&["dnf remove", "-y", "wget", "Removed:", "Complete!"])
            .run(false)
            .await
    }
}

#[cfg(target_os = "linux")]
mod zypper {
    use super::Test;
    use tokio::test;

    #[test]
    async fn si_ok() {
        Test::new()
            .pacaptr(&["-Si", "curl"], &[])
            .output(&["A Tool for Transferring Data from URLs"])
            .run(false)
            .await
    }

    #[test]
    #[should_panic(expected = "Failed with pattern `Why not use curl instead?`")]
    async fn si_fail() {
        Test::new()
            .pacaptr(&["-Si", "wget"], &[])
            .output(&["Why not use curl instead?"])
            .run(false)
            .await
    }

    #[test]
    #[ignore]
    async fn r() {
        Test::new()
            .pacaptr(&["-S", "wget", "--yes"], &[])
            .output(&["zypper install", "-y", "wget", "Installing: wget"])
            .exec(&["wget", "-V"], &[])
            .output(&["GNU Wget"])
            .pacaptr(&["-R", "wget", "--yes"], &[])
            .output(&["zypper remove", "-y", "wget", "Removing wget"])
            .run(false)
            .await
    }
}
