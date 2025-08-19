use crate::cli::Cli;
use crate::line_reader::LineReader;
use crate::line_selector::{LineSelector, ParsedLineSelector};
use anyhow::{Context, Result};
use clap::Parser;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, Write};
use std::path::Path;

mod cli;
mod line_reader;
mod line_selector;

fn main() -> Result<()> {
    let args = Cli::parse();

    let file = open_file(&args.file)?;
    let mut file = BufReader::new(file);

    if !args.allow_binary_files {
        match is_binary(&mut file) {
            Ok(is_binary) => {
                if is_binary {
                    anyhow::bail!(
                        "`{}` is a binrary file. Use `--allow-binary-files` to suppress this error",
                        args.file.display()
                    )
                }
            }
            Err(err) => eprintln!(
                "Warning: Failed to determine if `{}` is binary. \
                Use `--allow-binary-files` to suppress this warning. Reason: {err}",
                args.file.display()
            ),
        }
    }

    let n_lines = count_lines_and_rewind(&mut file)?;

    let line_selectors: anyhow::Result<Box<[_]>> = args
        .line_selectors
        .iter()
        .map(|s| {
            LineSelector::new(s.trim(), n_lines)
                .with_context(|| format!("Invalid line selector: `{s}`"))
        })
        .collect();
    let line_selectors = line_selectors?;

    let mut sorted_line_selectors = line_selectors.clone();
    sorted_line_selectors.sort_unstable();

    let mut line_reader = LineReader::new(file);

    // continue from here
    // TODO: keep the original order
    // TODO: handel duplicates (maybe cache a line inside LineReader, but this is not effecient)
    // and cases like '1,2:4,3' (note that this is sorted but the current_line will need to move
    // back to 3 after 4)
    for line_selector in sorted_line_selectors {
        let LineSelector { parsed, .. } = line_selector;
        match parsed {
            ParsedLineSelector::Single(line_num) => {
                let line = read_line(line_num, &mut line_reader)?;
                print_line(&line)?;
            }
            ParsedLineSelector::Range(lower, upper) => {
                for line_num in lower..=upper {
                    let line = read_line(line_num, &mut line_reader)?;
                    print_line(&line)?;
                }
            }
        }
    }

    // 2,1,3:5,4 (before sort)
    // 1,2,3:5,4 (after sort)

    Ok(())
}

fn print_line(line: &[u8]) -> Result<(), anyhow::Error> {
    std::io::stdout()
        .lock()
        .write_all(line)
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
            eprintln!(
                "Warning: couldn't determine if `{}` is a file or a directory from its metadata, treating it as a file: {error}",
                path.display()
            );
        }
    }

    Ok(file)
}

/// Note: `line_num` should be zero-indexed
fn read_line<R: BufRead>(line_num: usize, line_reader: &mut LineReader<R>) -> Result<Vec<u8>> {
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
