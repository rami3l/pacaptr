use self::utils::Test;

mod utils;

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
