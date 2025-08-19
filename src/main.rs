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

    if args.line_num == 0 {
        anyhow::bail!("Line number can't be zero");
    }

    let file = open_file(&args.file)?;
    let mut file = BufReader::new(file);

    if !args.allow_binary_files {
        match is_binary(&mut file) {
            Ok(true) => anyhow::bail!(
                "`{}` is a binrary file. Use `--allow-binary-files` to suppress this error",
                args.file.display()
            ),
            Ok(false) => {}
            Err(err) => eprintln!(
                "Warning: Failed to determine if `{}` is binary. Use `--allow-binary-files` to suppress this warning. Reason: {err}",
                args.file.display()
            ),
        }
    }

    let n_lines = count_lines_and_rewind(&mut file)?;

    if args.line_num.unsigned_abs() > n_lines {
        anyhow::bail!(
            "Line {} is out of bound, input has {} line(s)",
            args.line_num,
            n_lines
        );
    }
    let line_num = to_pisitive_zero_index(args.line_num, n_lines);

    let line_reader = LineReader::new(file);
    let line = read_line(line_num, line_reader)?;

    std::io::stdout()
        .lock()
        .write_all(&line)
        .context("Failed to write line to stdout")?;

    Ok(())
}

/// Converts negative line numbers to possitve and converts one-index to zero-index
fn to_pisitive_zero_index(line_num: isize, n_lines: usize) -> usize {
    if line_num < 0 {
        n_lines - -line_num as usize
    } else {
        // subtracte one to convert to zero-index
        line_num as usize - 1
    }
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
            eprintln!(
                "Warning: couldn't determine if `{}` is a file or a directory from its metadata, treating it as a file: {error}",
                path.display()
            );
        }
    }

    Ok(file)
}

/// Note: `line_num` should be zero-indexed
fn read_line<R: BufRead>(line_num: usize, mut line_reader: LineReader<R>) -> Result<Vec<u8>> {
    let mut line_buf = Vec::new();
    line_reader
        .read_specific_line(&mut line_buf, line_num)
        .with_context(|| format!("Failed to read line number {line_num}"))?;
    Ok(line_buf)
}

fn is_binary(file: &mut BufReader<File>) -> Result<bool> {
    let mut buf = [0; 64];
    let n = file.read(&mut buf)?;
    let buf = &buf[..n];

    file.rewind()?;

    Ok(content_inspector::inspect(buf).is_binary())
}

// TODO: support seek for stdin https://github.com/rust-lang/rust/issues/72802#issuecomment-1101996578
// and https://github.com/uutils/coreutils/pull/4189/files#diff-bd7f28594a45798eed07dea6767fc2bb5cb29e2d2855366ba65b126248bfd4b9R128-R132
/// Counts the number of lines in a reader and rewinds it
pub(crate) fn count_lines_and_rewind<R: BufRead + Seek>(reader: &mut R) -> anyhow::Result<usize> {
    let mut n_lines = 0;
    while reader.skip_until(b'\n')? > 0 {
        n_lines += 1;
    }
    reader.rewind()?;
    Ok(n_lines)
}
