/// The helper macro to generate a batch of different doc macros.
#[macro_export]
#[doc(hidden)]
macro_rules! docs_factory {
    ( $( $name:ident => $res:expr ),* $(,)? ) => {
        $( #[doc(hidden)]
        macro_rules! $name {
            () => {
                $res
            };
        } )*
    };
}
