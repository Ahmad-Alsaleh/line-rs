use crate::cli::Cli;
use crate::line_reader::LineReader;
use anyhow::{Context, Result};
use clap::Parser;
use std::fs::File;
use std::io::Write;
use std::path::Path;

mod cli;
mod line_reader;

fn main() -> Result<()> {
    let mut args = Cli::parse();

    if args.line == 0 {
        anyhow::bail!("Line number can't be zero");
    }

    // the cli user uses one-index but the code uses zero-index
    args.line -= 1;

    let file = open_file(&args.file)?;
    let mut line_reader = LineReader::new(file).context("Failed to create file reader")?;

    let mut line = Vec::new();
    let line_in_range = line_reader
        .read_specific_line(&mut line, args.line)
        .with_context(|| format!("Failed to read line {}", args.line + 1))?;
    if !line_in_range {
        anyhow::bail!(
            "Line {} is out of bound, file has {} line only",
            args.line + 1,
            line_reader.current_line
        );
    }

    std::io::stdout()
        .write_all(&line)
        .context("Failed to write line to stdout")?;

    Ok(())
}

fn open_file(path: &Path) -> Result<File> {
    let file =
        File::open(path).with_context(|| format!("Failed to open file `{}`", path.display()))?;
    if !file
        .metadata()
        .context("Failed to extract file metadata")?
        .is_file()
    {
        anyhow::bail!("{} is not a file", path.display());
    }
    Ok(file)
}
