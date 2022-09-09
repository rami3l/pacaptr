//! `pacaptr` is a `pacman`-like syntax wrapper for many package managers.
//!
//! # Compatibility Table
//!
//! Currently, `pacaptr` supports the following operations:
#![doc = pacaptr_macros::compat_table!()]
//! Note: Some flags are "translated" so are not shown in this table, eg. `-p`
//! in `-Sp`.

#![forbid(unsafe_code)]
#![warn(
    clippy::pedantic,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    rustdoc::broken_intra_doc_links,
    trivial_numeric_casts,
    unused_allocation
)]

pub mod dispatch;
pub mod error;
mod exec;
mod pm;
pub mod print;
