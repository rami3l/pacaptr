//! `pacaptr` is a `pacman`-like syntax wrapper for many package managers.
#![cfg_attr(
    doc,
    doc = indoc::indoc!{r##"
        # Compatibility Table

        Currently, `pacaptr` supports the following operations:
    "##}
)]
#![cfg_attr(doc, doc = pacaptr_macros::compat_table!())]
#![cfg_attr(
    doc,
    doc = indoc::indoc!{r##"
        Note: Some flags are "translated" so are not shown in this table, eg. `-p`
        in `-Sp`.
    "##}
)]
#![warn(missing_docs)]

pub mod config;
pub mod error;
pub mod exec;
pub mod pm;
pub mod print;
