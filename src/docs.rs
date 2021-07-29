#[macro_export]
#[doc(hidden)]
macro_rules! docs_errors_exec {
    () => {
        indoc! {"
            # Errors
            This function might return one of the following errors:

            - [`Error::CmdJoinError`]
            - [`Error::CmdNoHandleError`]
            - [`Error::CmdSpawnError`]
            - [`Error::CmdWaitError`]
        "}
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! docs_errors_grep {
    () => {
        indoc! {"
            # Errors
            Returns an [`Error::OtherError`] when any of the
            regex patterns is ill-formed.
        "}
    };
}
