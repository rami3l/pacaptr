use itertools::Itertools;
use litrs::StringLit;
use proc_macro2::{Literal, Span, TokenStream};
use quote::quote;
use std::convert::TryFrom;
use syn::{Error, Result};

// ! TODO: Make TestDsl a test-only module.
// ! TODO: Implement pipe with Rust.
enum TestDslItem {
    In(Vec<String>),
    InBang(Vec<String>),
    Ou(String),
}

impl TestDslItem {
    fn try_from_line(ln: &str) -> Result<Self> {
        let in_bang = "in ! ";
        let in_ = "in ";
        let ou = "ou ";
        if let Some(rest) = ln.strip_prefix(in_bang) {
            Ok(TestDslItem::InBang(
                rest.split_whitespace().map_into().collect(),
            ))
        } else if let Some(rest) = ln.strip_prefix(in_) {
            Ok(TestDslItem::In(
                rest.split_whitespace().map_into().collect(),
            ))
        } else if let Some(rest) = ln.strip_prefix(ou) {
            Ok(TestDslItem::Ou(rest.into()))
        } else {
            let msg = format!(
                "An item must start with `{}`, `{}`, or `{}`, found `{}`",
                in_bang, in_, ou, ln,
            );
            Err(Error::new(Span::call_site(), msg))
        }
    }

    fn build(&self) -> TokenStream {
        match self {
            TestDslItem::In(i) => {
                let i = i.iter().map(|s| Literal::string(s)).collect::<Vec<_>>();
                quote! { .pacaptr(&[ #(#i),* ], &[]) }
            }
            TestDslItem::InBang(i) => {
                let i = i.iter().map(|s| Literal::string(s)).collect::<Vec<_>>();
                quote! { .exec(&[ #(#i),* ], &[]) }
            }
            TestDslItem::Ou(o) => {
                let o = Literal::string(o);
                quote! { .output(&[ #o ]) }
            }
        }
    }
}

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

fn test_dsl_impl(input: &str) -> Result<TokenStream> {
    input
        .lines()
        // Filter out comments and empty lines
        .map(|ln| ln.trim_start().trim_end())
        .filter(|ln| !(ln.is_empty() || ln.starts_with('#')))
        .map(|ln| {
            let item = TestDslItem::try_from_line(ln)?;
            Ok(item.build())
        })
        .collect::<Result<Vec<_>>>()
        .map(|items| {
            quote! {
                Test::new()
                    #(#items)*
                    .run()
            }
        })
}
