mod compat_table;
mod test_dsl;

use anyhow::Result;
use itertools::Itertools;
use litrs::StringLit;
use proc_macro::TokenStream;
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
/// A comment in this DSL starts with a `#`.
///
/// # Examples
///
/// ```no_run
/// #[test]
/// #[ignore]
/// fn apt_r_s() {
///    test_dsl! { r##"
///        # Refresh with `pacaptr -Sy`.
///        in -Sy
///
///        # Install `screen`.
///        in -S screen --yes
///
///        # Verify installation.
///        in ! which screen
///        ou ^/usr/bin/screen
///
///        # Remove `screen` and verify the removal.
///        in -R screen --yes
///        in -Qi screen
///        ou ^Status: deinstall
///    "## }
/// }
/// ```
#[proc_macro]
pub fn test_dsl(input: TokenStream) -> TokenStream {
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

    res_token_stream(test_dsl_impl(string_lit.value()))
}

/// Generates the compatibility table as a docstring on the top of given input.
#[proc_macro]
pub fn compat_table(input: TokenStream) -> TokenStream {
    let res =
        compat_table_impl().map(|docstring| TokenStream::from_iter([docstring.into(), input]));
    res_token_stream(res)
}

fn res_token_stream(res: Result<impl Into<TokenStream>, syn::Error>) -> TokenStream {
    res.map_or_else(|e| e.to_compile_error().into(), Into::into)
}
