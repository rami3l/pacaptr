mod test_dsl;

use crate::test_dsl::test_dsl_impl;
use litrs::StringLit;
use proc_macro2::TokenStream;
use quote::quote;
use std::convert::TryFrom;

#[proc_macro]
pub fn test_dsl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = TokenStream::from(input);

    let input = input.into_iter().collect::<Vec<_>>();
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
