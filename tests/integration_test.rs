pub use self::utils::Test;

mod utils;

#[cfg(target_os = "macos")]
mod homebrew {
    use super::Test;

    #[test]
    fn working_example() {
        Test::new()
            .input(&["-Si", "curl"])
            .output(&["curl is keg-only"])
            .run(false)
    }

    #[test]
    #[should_panic(expected = "Failed with pattern `curl is not keg-only`")]
    fn error_example() {
        Test::new()
            .input(&["-Si", "curl"])
            .output(&["curl is not keg-only"])
            .run(false)
    }

    #[test]
    fn s_auto_cask() {
        Test::new()
            .input(&["-S", "curl", "gimp", "--dryrun"])
            .output(&["brew install curl", "brew cask install gimp"])
            .run(false)
    }

    #[test]
    fn s_force_cask() {
        Test::new()
            .input(&["-S", "docker", "--dryrun"])
            .output(&["brew install docker"])
            .input(&["-S", "docker", "--cask", "--dryrun"])
            .output(&["brew cask install docker"])
            .run(false)
    }

    #[test]
    fn r_cask() {
        Test::new()
            .input(&["-R", "curl", "gimp", "--dryrun"])
            .output(&["brew uninstall curl", "brew cask uninstall gimp"])
            .run(false)
    }

    #[test]
    #[ignore]
    fn install_uninstall() {
        Test::new()
            .input(&["-S", "wget"])
            .output(&["brew install wget"])
            .input(&["-S", "wget"])
            .output(&["brew install wget", "is already installed"])
            .input(&["-R", "wget"])
            .output(&["brew uninstall wget", "Uninstalling /usr/local/Cellar/wget"])
            .run(false)
    }
}

#[cfg(target_os = "windows")]
mod chocolatey {
    use super::Test;

    #[test]
    fn working_example() {
        Test::new()
            .input(&["-Si", "wget"])
            .output(&["GNU Wget is a free software package"])
            .run(false)
    }

    #[test]
    #[should_panic(expected = "Failed with pattern `GNU Wget is not a free software package`")]
    fn error_example() {
        Test::new()
            .input(&["-Si", "wget"])
            .output(&["GNU Wget is not a free software package"])
            .run(false)
    }

    #[test]
    #[ignore]
    fn install_uninstall() {
        Test::new()
            .input(&["-S", "wget", "--yes"])
            .output(&["The install of wget was successful."])
            .input(&["-R", "wget", "--yes"])
            .output(&["Wget has been successfully uninstalled."])
            .run(false)
    }
}
