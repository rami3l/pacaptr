use anyhow::{Context, Result};
use itertools::Itertools;
use prettytable::{Cell, Row, Table};
use regex::Regex;
use std::{collections::BTreeMap, ffi::OsString, fs, io::Write, iter, path::Path};

const PM_IMPL_DIR: &str = "src/pm/";
const COMPAT_TABLE_PATH: &str = "docs/compatibility_table.md";
const METHODS: &[&str] = &[
    "q", "qc", "qe", "qi", "qk", "ql", "qm", "qo", "qp", "qs", "qu", "r", "rn", "rns", "rs", "rss",
    "s", "sc", "scc", "sccc", "sg", "si", "sii", "sl", "ss", "su", "suy", "sw", "sy", "u",
];

/// Checks the implementation status of `pacman` commands in a specific file (eg. `homebrew.rs`).
fn check_methods(file: &Path) -> Result<BTreeMap<String, bool>> {
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

fn main() -> Result<()> {
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo:rerun-if-changed={}", PM_IMPL_DIR);

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

    let mut file =
        fs::File::create(COMPAT_TABLE_PATH).context("Failed while creating compatibility table")?;
    file.write_all("```txt\n".as_bytes())?;
    table
        .print(&mut file)
        .context("Failed while writing compatibility table")?;
    file.write_all("```\n".as_bytes())?;

    Ok(())
}
