use std::{collections::BTreeMap, ffi::OsString, fmt::Debug, fs, iter, path::Path, str::FromStr};

use anyhow::Context;
use itertools::Itertools;
use prettytable::{Cell, Row, Table};
use proc_macro2::{Span, TokenStream};
use regex::Regex;
use syn::{Error, Result};

const PM_IMPL_DIR: &str = "src/pm/";
const METHODS: &[&str] = &[
    "q", "qc", "qe", "qi", "qk", "ql", "qm", "qo", "qp", "qs", "qu", "r", "rn", "rns", "rs", "rss",
    "s", "sc", "scc", "sccc", "sg", "si", "sii", "sl", "ss", "su", "suy", "sw", "sy", "u",
];

/// Checks the implementation status of `pacman` commands in a specific file
/// (eg. `homebrew.rs`).
fn check_methods(file: &Path) -> anyhow::Result<BTreeMap<String, bool>> {
    let bytes = fs::read(file)?;
    let contents = String::from_utf8(bytes)?;

    METHODS
        .iter()
        .map(|&method| {
            // A function definition (rg. `rs`) is written as follows:
            // `(async) fn rs(..) {..}`
            let found = Regex::new(&format!(r#"fn\s+{}\s*\("#, method))?.is_match(&contents);
            Ok((method.to_owned(), found))
        })
        .try_collect()
}

fn make_table() -> anyhow::Result<String> {
    let paths: Vec<fs::DirEntry> = fs::read_dir(PM_IMPL_DIR)
        .context("Failed while reading PM_IMPL_DIR")?
        .map(|entry| entry.context("Error while reading path"))
        .try_collect()?;

    let excluded_names = ["mod.rs", "unknown.rs"];
    let impls: BTreeMap<OsString, BTreeMap<String, bool>> = paths
        .iter()
        .filter(|entry| !excluded_names.iter().any(|&ex| ex == entry.file_name()))
        .map(|entry| check_methods(&entry.path()).map(|impl_| (entry.file_name(), impl_)))
        .try_collect()?;

    let make_row = |name: &str, data: &[&str]| {
        let row = iter::once(&name)
            .chain(data)
            .map(|&s| Cell::new(s))
            .collect();
        Row::new(row)
    };

    // First row: `row!["", "q", "qc", "qe", ..]`
    let head = Ok(make_row("", METHODS));
    let tail = impls.iter().map(|(file, items)| {
        let data = METHODS
            .iter()
            .map(|&method| {
                items
                    .get(method)
                    .expect("Implementation details not registered")
                    .then(|| "*")
                    .unwrap_or("")
            })
            .collect_vec();

        file.to_str()
            .context("Failed to convert `file: OsString` to `&str`")
            .map(|file| make_row(file, &data))
    });
    let mut table: Table = iter::once(head).chain(tail).try_collect()?;
    table.set_format(*prettytable::format::consts::FORMAT_CLEAN);

    let res = format!("```txt\n{}```\n", table);
    Ok(res)
}

pub(crate) fn compat_table_impl() -> Result<TokenStream> {
    fn throw(e: &dyn Debug) -> Error {
        let msg = format!("{:?}", e);
        Error::new(Span::call_site(), msg)
    }

    let table = make_table().map_err(|e| throw(&e as _))?;
    let comments = format!(r##"r#"{}"#"##, table);
    let res = TokenStream::from_str(&comments)?;
    Ok(res)
}
