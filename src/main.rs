use anyhow::Context;
use clap::Parser;
use std::fs::File;
use std::io::{BufRead, BufReader};

mod cli;

use cli::Cli;

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    let file =
        File::open(&args.file).with_context(|| format!("Failed to open file `{}`", args.file))?;
    let file = BufReader::new(file);
    if let Some(line) = file.lines().nth(args.line - 1) {
        let line = line.context("Failed to read line")?;
        println!("{line}");
    } else {
        anyhow::bail!("Line {} is too large", args.line);
    }

    Ok(())
}
