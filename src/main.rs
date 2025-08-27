use crate::cli::Cli;
use crate::line_reader::LineReader;
use crate::line_selector::{LineSelector, ParsedLineSelector};
use anyhow::{Context, Result};
use clap::Parser;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
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

    // check if the file is binary and binary files are not allowed
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

    if n_lines == 0 {
        if !args.plain {
            // TODO: use pretty printing here
            println!("--- EMPTY FILE ---");
        }
        return Ok(());
    }

    // parse line selectors from cli args
    let line_selectors: anyhow::Result<Box<[_]>> = args
        .original_line_selectors
        .into_iter()
        .map(|original_line_selector| {
            LineSelector::from_original(original_line_selector, n_lines)
                .with_context(|| format!("Invalid line selector: {original_line_selector}"))
        })
        .collect();
    let line_selectors = line_selectors?;

    let mut sorted_line_selectors = line_selectors.clone();
    sorted_line_selectors.sort_unstable();

    let mut line_reader = LineReader::new(file);

    // TODO: benchmark to check if using a Vec + binary search is better than using a hash map
    // read and store selected lines
    let mut lines: HashMap<usize, Vec<u8>> = HashMap::new();
    for line_selector in sorted_line_selectors {
        match line_selector.parsed {
            ParsedLineSelector::Single(line_num) => {
                if let Entry::Vacant(e) = lines.entry(line_num) {
                    let mut line_buf = Vec::new();
                    read_line(&mut line_buf, line_num, &mut line_reader)?;
                    e.insert(line_buf);
                }
            }
            ParsedLineSelector::Range(lower, upper) => {
                for line_num in lower..=upper {
                    if let Entry::Vacant(e) = lines.entry(line_num) {
                        let mut line_buf = Vec::new();
                        read_line(&mut line_buf, line_num, &mut line_reader)?;
                        e.insert(line_buf);
                    }
                }
            }
        }
    }

    // print selected lines
    for line_selector in line_selectors {
        if !args.plain {
            println!("{}", line_selector.original);
        }
        match line_selector.parsed {
            ParsedLineSelector::Single(line_num) => {
                let line = &lines[&line_num];
                print_line(line)?;
            }
            ParsedLineSelector::Range(lower, upper) => {
                for line_num in lower..=upper {
                    let line = &lines[&line_num];
                    print_line(line)?;
                }
            }
        }
    }

    Ok(())
}

fn print_line(line: &[u8]) -> anyhow::Result<()> {
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
                "Warning: couldn't determine if `{}` is a file or a directory from its metadata, \
                treating it as a file. Reason: {error}",
                path.display()
            );
        }
    }

    Ok(file)
}

/// Note: `line_num` should be zero-based
fn read_line<R: BufRead>(
    buf: &mut Vec<u8>,
    line_num: usize,
    line_reader: &mut LineReader<R>,
) -> anyhow::Result<()> {
    line_reader
        .read_specific_line(buf, line_num)
        .with_context(|| format!("Failed to read line number {line_num}"))
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

#[cfg(test)]
mod tests {
    use super::*;

    mod n_selected_lines {
        use super::*;
        use crate::line_selector::OriginalLineSelector;

        macro_rules! create_line_selector {
            ($s: literal, $n_lines: literal) => {{
                let original = OriginalLineSelector::from_str($s).unwrap();
                LineSelector::from_original(original, $n_lines)
            }};
        }

        #[test]
        fn b_lower_is_a_lower() {
            let a = create_line_selector!("2:7", 42).unwrap();
            let b = create_line_selector!("2", 42).unwrap();
            let mut v = [a, b];
            v.sort();
            assert_eq!(n_selected_lines(&v), 6);

            let a = create_line_selector!("2:7", 42).unwrap();
            let b = create_line_selector!("2:5", 42).unwrap();
            let mut v = [a, b];
            v.sort();
            assert_eq!(n_selected_lines(&v), 6);

            let a = create_line_selector!("2:7", 42).unwrap();
            let b = create_line_selector!("2:7", 42).unwrap();
            let mut v = [a, b];
            v.sort();
            assert_eq!(n_selected_lines(&v), 6);

            let a = create_line_selector!("2:7", 42).unwrap();
            let b = create_line_selector!("2:9", 42).unwrap();
            let mut v = [a, b];
            v.sort();
            assert_eq!(n_selected_lines(&v), 8);

            let a = create_line_selector!("3", 42).unwrap();
            let b = create_line_selector!("3", 42).unwrap();
            let mut v = [a, b];
            v.sort();
            assert_eq!(n_selected_lines(&v), 1);

            let a = create_line_selector!("3", 42).unwrap();
            let b = create_line_selector!("3:5", 42).unwrap();
            let mut v = [a, b];
            v.sort();
            assert_eq!(n_selected_lines(&v), 3);
        }

        #[test]
        fn b_lower_is_inside_a() {
            let a = create_line_selector!("2:7", 42).unwrap();
            let b = create_line_selector!("4", 42).unwrap();
            let mut v = [a, b];
            v.sort();
            assert_eq!(n_selected_lines(&v), 6);

            let a = create_line_selector!("2:7", 42).unwrap();
            let b = create_line_selector!("4:6", 42).unwrap();
            let mut v = [a, b];
            v.sort();
            assert_eq!(n_selected_lines(&v), 6);

            let a = create_line_selector!("2:7", 42).unwrap();
            let b = create_line_selector!("4:7", 42).unwrap();
            let mut v = [a, b];
            v.sort();
            assert_eq!(n_selected_lines(&v), 6);

            let a = create_line_selector!("2:7", 42).unwrap();
            let b = create_line_selector!("4:9", 42).unwrap();
            let mut v = [a, b];
            v.sort();
            assert_eq!(n_selected_lines(&v), 8);
        }

        #[test]
        fn b_lower_is_a_upper() {
            let a = create_line_selector!("2:6", 42).unwrap();
            let b = create_line_selector!("6", 42).unwrap();
            let mut v = [a, b];
            v.sort();
            assert_eq!(n_selected_lines(&v), 5);

            let a = create_line_selector!("2:6", 42).unwrap();
            let b = create_line_selector!("6:8", 42).unwrap();
            let mut v = [a, b];
            v.sort();
            assert_eq!(n_selected_lines(&v), 7);
        }

        #[test]
        fn b_lower_is_outside_a() {
            let a = create_line_selector!("2:6", 42).unwrap();
            let b = create_line_selector!("7", 42).unwrap();
            let mut v = [a, b];
            v.sort();
            assert_eq!(n_selected_lines(&v), 6);

            let a = create_line_selector!("2:6", 42).unwrap();
            let b = create_line_selector!("7:9", 42).unwrap();
            let mut v = [a, b];
            v.sort();
            assert_eq!(n_selected_lines(&v), 8);

            let a = create_line_selector!("3", 42).unwrap();
            let b = create_line_selector!("5", 42).unwrap();
            let mut v = [a, b];
            v.sort();
            assert_eq!(n_selected_lines(&v), 2);

            let a = create_line_selector!("3", 42).unwrap();
            let b = create_line_selector!("5:7", 42).unwrap();
            let mut v = [a, b];
            v.sort();
            assert_eq!(n_selected_lines(&v), 4);
        }
    }
}
