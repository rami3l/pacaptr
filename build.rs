use anyhow::{Context, Result};
use prettytable::{Cell, Row, Table};
use regex::Regex;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

const PM_IMPL_DIR: &str = "src/package_manager/";
const COMPAT_TABLE_PATH: &str = "docs/compatibility_table.txt";
const METHODS: &[&str] = &[
    "q", "qc", "qe", "qi", "qk", "ql", "qm", "qo", "qp", "qs", "qu", "r", "rn", "rns", "rs", "rss",
    "s", "sc", "scc", "sccc", "sg", "si", "sii", "sl", "ss", "su", "suy", "sw", "sy", "u",
];

/// Check the implementation status of `pacman` commands in a specific file (eg. `homebrew.rs`).
fn check_methods(file: &Path) -> Result<BTreeMap<String, bool>> {
    let bytes = fs::read(file)?;
    let contents = String::from_utf8(bytes)?;

    let mut res = BTreeMap::new();
    for &method in METHODS {
        // ! Here we depend implicitly on `rustfmt`: a function definition is written as follows:
        // ! `(async) fn rs(..) {..}`, so we match `fn rs(` in order distinguish `rss` from `rs`.
        let found = Regex::new(&format!(r#"fn\s{}\("#, method))?.is_match(&contents);
        res.insert(method.to_owned(), found);
    }

    Ok(res)
}

fn main() -> Result<()> {
    // Tell Cargo that if the given file changes, to rerun this build script.

    println!("cargo:rerun-if-changed={}", PM_IMPL_DIR);

    let paths = fs::read_dir(PM_IMPL_DIR).context("Failed while reading PM_IMPL_DIR")?;

    let excluded_names = ["mod.rs", "unknown.rs"];

    let mut impls = BTreeMap::new();
    for path in paths {
        let entry = path.context("Error while reading path")?;
        if excluded_names.iter().any(|&ex| ex == entry.file_name()) {
            continue;
        }
        impls.insert(entry.file_name(), check_methods(entry.path().as_ref())?);
    }

    let mut table = Table::new();
    let make_row = |row_name: &str, data: &[&str]| {
        Row::new({
            let mut row = vec![row_name];
            row.extend(data);
            row.into_iter().map(Cell::new).collect()
        })
    };

    // Add first row:
    // `row!["", "q", "qc", "qe", ..]`
    table.add_row(make_row("", METHODS));

    for (file, items) in &impls {
        let data = METHODS
            .iter()
            .map(|&method| {
                let has_impl = items
                    .get(method)
                    .expect("Implementation details not registered");
                if *has_impl {
                    "*"
                } else {
                    ""
                }
            })
            .collect::<Vec<_>>();

        table.add_row(make_row(
            file.to_str()
                .context("Failed to convert `file: OsString` to `&str`")?,
            &data,
        ));
    }

    table.set_format(*prettytable::format::consts::FORMAT_CLEAN);

    let mut file =
        fs::File::create(COMPAT_TABLE_PATH).context("Failed while creating compatibility table")?;
    table
        .print(&mut file)
        .context("Failed while writing compatibility table")?;

    Ok(())
}
