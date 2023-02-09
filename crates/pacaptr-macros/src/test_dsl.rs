use itertools::Itertools;
use proc_macro2::{Literal, Span, TokenStream};
use quote::quote;
use syn::{Error, Result};

enum TestDslItem {
    Im(Vec<String>),
    In(Vec<String>),
    InBang(Vec<String>),
    Ou(String),
}

impl TestDslItem {
    fn try_from_line(ln: &str) -> Result<Self> {
        let in_bang = "in ! ";
        let im = "im ";
        let in_ = "in ";
        let ou = "ou ";
        let tokenize = |s: &str| s.split_whitespace().map_into().collect();
        #[allow(clippy::option_if_let_else)]
        if let Some(rest) = ln.strip_prefix(in_bang) {
            Ok(Self::InBang(tokenize(rest)))
        } else if let Some(rest) = ln.strip_prefix(in_) {
            Ok(Self::In(tokenize(rest)))
        } else if let Some(rest) = ln.strip_prefix(im) {
            Ok(Self::Im(tokenize(rest)))
        } else if let Some(rest) = ln.strip_prefix(ou) {
            Ok(Self::Ou(rest.into()))
        } else {
            let msg = format!(
                "Item must start with one of the following: {}, found `{}`",
                [in_bang, in_, ou, im]
                    .iter()
                    .map(|s| format!("`{}`", s.trim_end()))
                    .join(", "),
                ln,
            );
            Err(Error::new(Span::call_site(), msg))
        }
    }

    fn build(&self) -> Result<TokenStream> {
        match self {
            Self::In(i) => {
                let i = i.iter().map(|s| Literal::string(s)).collect_vec();
                Ok(quote! { .pacaptr(&[ #(#i),* ], &[]) })
            }
            Self::InBang(i) => {
                let i = i.iter().map(|s| Literal::string(s)).collect_vec();
                Ok(quote! { .exec(&[ #(#i),* ], &[]) })
            }
            Self::Ou(o) => {
                let o = Literal::string(o);
                Ok(quote! { .output(&[ #o ]) })
            }
            Self::Im(_) => {
                let msg = "`im` items are not yet supported";
                Err(Error::new(Span::call_site(), msg))
            }
        }
    }
}

#[allow(clippy::module_name_repetitions)]
pub fn test_dsl_impl(input: &str) -> Result<TokenStream> {
    let items: Vec<TokenStream> = input
        .lines()
        .map(|ln| ln.trim_start().trim_end())
        // Filter out comments and empty lines.
        .filter(|ln| !(ln.is_empty() || ln.starts_with('#')))
        .map(|ln| TestDslItem::try_from_line(ln).and_then(|item| item.build()))
        .try_collect()?;
    Ok(quote! { Test::new() #(#items)* .run()})
}
