use crate::docs_factory;

docs_factory! {
    docs_errors_exec => ::indoc::indoc! {"
        # Errors
        This function might return one of the following errors:
        - [`Error::CmdJoinError`]
        - [`Error::CmdNoHandleError`]
        - [`Error::CmdSpawnError`]
        - [`Error::CmdWaitError`]
    "},

    docs_errors_grep => ::indoc::indoc! {"
        # Errors
        Returns an [`Error::OtherError`] when any of the
        regex patterns is ill-formed.
    "},
}
