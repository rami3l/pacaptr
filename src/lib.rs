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
#![forbid(unsafe_code)]
#![warn(
    clippy::doc_markdown,
    clippy::nursery,
    clippy::pedantic,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    rustdoc::broken_intra_doc_links,
    trivial_numeric_casts,
    unused_allocation
)]

pub mod config;
pub mod error;
pub mod exec;
pub mod pm;
pub mod print;
