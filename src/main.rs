use crate::cli::Cli;
use crate::line_reader::LineReader;
use anyhow::{Context, Result};
use clap::Parser;
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::path::Path;

mod cli;
mod line_reader;

fn main() -> Result<()> {
    let args = Cli::parse();

    if args.line_num == 0 {
        anyhow::bail!("Line number can't be zero");
    }

    let mut file = open_file(&args.file)?;

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

    let line_reader = LineReader::new(file)?;

    if args.line_num.unsigned_abs() > line_reader.n_lines {
        anyhow::bail!(
            "Line {} is out of bound, input has {} line(s) only",
            args.line_num,
            line_reader.n_lines
        );
    }

    let line_num = if args.line_num < 0 {
        line_reader.n_lines - -args.line_num as usize
    } else {
        // subtracte one to convert to zero-index
        args.line_num as usize - 1
    };

    let line = read_line(line_num, line_reader)?;

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
                "Warning: couldn't determine if `{}` is a file or a directory from its metadata, treating it as a file: {error}",
                path.display()
            );
        }
    }

    Ok(file)
}

/// Note: `line_num` should be zero-indexed
fn read_line(line_num: usize, mut line_reader: LineReader) -> Result<Vec<u8>> {
    let mut line_buf = Vec::new();
    line_reader
        .read_specific_line(&mut line_buf, line_num)
        .with_context(|| format!("Failed to read line number {line_num}"))?;
    Ok(line_buf)
}

/// Note: this funciton rewinds to the begginsing of the file after doing the necesary
/// operatoins, i.e., it assumes no lines were read from the file before calling this function
fn is_binary(file: &mut File) -> Result<bool> {
    let mut buf = [0; 64];
    let n = file.read(&mut buf)?;
    let buf = &buf[..n];

    file.rewind()?;

    Ok(content_inspector::inspect(buf).is_binary())
}
