pub use self::utils::Test;

mod utils;

mod homebrew {
    use super::Test;

    #[test]
    #[cfg(target_os = "macos")]
    fn homebrew_working_example() {
        Test::new()
            .input(&["-Si", "curl"])
            .output(&["curl is keg-only"])
            .run(false)
    }

    #[test]
    #[cfg(target_os = "macos")]
    #[should_panic(expected = "Failed with pattern `curl is not keg-only`")]
    fn homebrew_error_example() {
        Test::new()
            .input(&["-Si", "curl"])
            .output(&["curl is not keg-only"])
            .run(false)
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn homebrew_s_cask() {
        Test::new()
            .input(&["-S", "curl", "gimp", "--dryrun"])
            .output(&["brew install curl", "brew cask install gimp"])
            .run(false)
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn homebrew_r_cask() {
        Test::new()
            .input(&["-R", "curl", "gimp", "--dryrun"])
            .output(&["brew uninstall curl", "brew cask uninstall gimp"])
            .run(false)
    }
}
