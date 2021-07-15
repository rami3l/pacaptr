mod compat_table;
mod test_dsl;

use std::{convert::TryFrom, iter::FromIterator};

use itertools::Itertools;
use litrs::StringLit;
use proc_macro2::TokenStream;
use quote::quote;

use crate::{compat_table::compat_table_impl, test_dsl::test_dsl_impl};

/// A DSL (Domain-Specific Language) embedded in Rust, in order to simplify the
/// form of smoke tests.
///
/// This macro accepts the source of the Test DSL in a **string literal**.
/// In this DSL, each line is called an `item`. We now support the following
/// item types:
/// - `in` item: Run command on `pacaptr`.
/// - `in !` item: Run command with the system shell (`sh` on Unix,`powershell`
///   on Windows).
/// - `ou` item: Check the output of the **last** `in` or `in !` item above
///   against a **regex** pattern.
///
/// # Examples
///
/// ```no_run
/// #[test]
/// #[ignore]
/// fn apt_r_s() {
///    test_dsl! { r##"
///        in -Sy               # Refresh with `pacaptr -Sy`.
///        in -S screen --yes   # Install `screen`.
///        in ! which screen    # Verify installation.
///        ou ^/usr/bin/screen
///
///        in -R screen --yes   # Remove `screen`.
///        in -Qi screen        # Verify removal.
///        ou ^Status: deinstall
///    "## }
/// }
/// ```
#[proc_macro]
pub fn test_dsl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = TokenStream::from(input);

    let input = input.into_iter().collect_vec();
    if input.len() != 1 {
        let msg = format!(
            "argument must be a single string literal, but got {} tokens",
            input.len()
        );
        return quote! { compile_error!(#msg) }.into();
    }

    let string_lit = match StringLit::try_from(&input[0]) {
        // Error if the token is not a string literal
        Err(e) => return e.to_compile_error(),
        Ok(lit) => lit,
    };

    let res = match test_dsl_impl(string_lit.value()) {
        Err(e) => return e.to_compile_error().into(),
        Ok(r) => r,
    };

    proc_macro::TokenStream::from(res)
}

#[proc_macro]
pub fn compat_table(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let comments = match compat_table_impl() {
        Err(e) => return e.to_compile_error().into(),
        Ok(r) => proc_macro::TokenStream::from(r),
    };
    proc_macro::TokenStream::from_iter(vec![comments, input])
}
