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
    let args = Cli::parse();

    if args.line == 0 {
        anyhow::bail!("Line number can't be zero");
    }

    let file = open_file(&args.file)?;
    let line_reader = LineReader::new(file).context("Failed to create file reader")?;

    let line = read_line(args.line, line_reader)?;

    std::io::stdout()
        .lock()
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

fn read_line(line: usize, mut line_reader: LineReader) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    let line_in_range = line_reader
        // subtract one since the cli user uses one-index and the code uses zero-index
        .read_specific_line(&mut buf, line - 1)
        .with_context(|| format!("Failed to read line {line}"))?;

    if !line_in_range {
        anyhow::bail!(
            "Line {line} is out of bound, file has {} line only",
            line_reader.current_line
        );
    }

    Ok(buf)
}
