use itertools::Itertools;
use proc_macro2::{Literal, Span, TokenStream};
use quote::quote;
use syn::{Error, Result};

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
                let i = i.iter().map(|s| Literal::string(s)).collect_vec();
                quote! { .pacaptr(&[ #(#i),* ], &[]) }
            }
            TestDslItem::InBang(i) => {
                let i = i.iter().map(|s| Literal::string(s)).collect_vec();
                quote! { .exec(&[ #(#i),* ], &[]) }
            }
            TestDslItem::Ou(o) => {
                let o = Literal::string(o);
                quote! { .output(&[ #o ]) }
            }
        }
    }
}

pub(crate) fn test_dsl_impl(input: &str) -> Result<TokenStream> {
    let items = input
        .lines()
        .map(|ln| ln.trim_start().trim_end())
        // Filter out comments and empty lines.
        .filter(|ln| !(ln.is_empty() || ln.starts_with('#')))
        .map(|ln| TestDslItem::try_from_line(ln).map(|item| item.build()))
        .collect::<Result<Vec<_>>>()?;
    Ok(quote! {
        Test::new()
        #(#items)*
        .run()
    })
}
