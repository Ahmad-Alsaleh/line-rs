use crate::cli::Cli;
use crate::line_reader::LineReader;
use anyhow::{Context, Result};
use clap::Parser;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, Write};
use std::path::Path;

mod cli;
mod line_reader;

fn main() -> Result<()> {
    let args = Cli::parse();

    if args.line == 0 {
        anyhow::bail!("Line number can't be zero");
    }

    let file = open_file(&args.file)?;
    let mut file = BufReader::new(file);

    if !args.allow_binary_files {
        let is_binary = is_binary(&mut file).with_context(|| {
            format!("Failed to determine if `{}` is binary", args.file.display())
        })?;

        if is_binary {
            anyhow::bail!(
                "`{}` is a binrary file. Use `--allow-binary-files` to suppress this error",
                args.file.display()
            );
        }
    }

    let line_reader = LineReader::new(file);

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

    match file.metadata() {
        Ok(metadata) => {
            if !metadata.is_file() {
                anyhow::bail!("`{}` is not a file", path.display());
            }
        }
        Err(error) => {
            // TODO: make a `--quiet` flag to suppress warning
            // TODO: color the word `Warning` in yellow
            eprintln!(
                "Warning: couldn't determine if `{}` is a file or a directory: {error}",
                path.display()
            );
        }
    }

    Ok(file)
}

fn read_line(line: usize, mut line_reader: LineReader) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    let line_in_range = line_reader
        // subtracting one since the cli user uses one-index and the code uses zero-index
        .read_specific_line(&mut buf, line - 1)
        .with_context(|| format!("Failed to read line number {line}"))?;

    if !line_in_range {
        anyhow::bail!(
            "Line {line} is out of bound, file has {} line(s) only",
            line_reader.current_line
        );
    }

    Ok(buf)
}

fn is_binary(file: &mut BufReader<File>) -> Result<bool> {
    let mut buf = [0; 64];
    let n = file.read(&mut buf)?;
    let buf = &buf[..n];

    file.rewind()?;

    Ok(content_inspector::inspect(buf).is_binary())
}
