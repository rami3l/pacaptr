use anyhow::Result;
use xshell::{cmd, read_file};

fn main() -> Result<()> {
    let name = "Julia";
    let output = cmd!("echo hello {name}!").read()?;
    assert_eq!(output, "hello Julia!");

    let err = read_file("feeling-lucky.txt").unwrap_err();
    assert_eq!(
        err.to_string(),
        "`feeling-lucky.txt`: no such file or directory (os error 2)",
    );

    Ok(())
}
