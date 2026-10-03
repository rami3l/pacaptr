//! `pacaptr` is a `pacman`-like syntax wrapper for many package managers.
//! # Compatibility Table
//!
//! Currently, `pacaptr` supports the following operations:
#![doc = include_str!("pm/compat_table.md")]
//! Note: Some flags are "translated" so are not shown in this table, eg. `-p`
//! in `-Sp`.

#![warn(missing_docs)]
#![cfg_attr(any(test, feature = "test"), allow(clippy::wildcard_imports))]

pub mod config;
pub mod error;
pub mod exec;
pub mod pm;
pub mod print;
