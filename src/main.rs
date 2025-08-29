use crate::cli::Cli;
use crate::line_reader::LineReader;
use crate::line_selector::{LineSelector, ParsedLineSelector};
use anyhow::{Context, Result};
use clap::Parser;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
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

    let mut n_lines = 0;
    if !args.allow_binary_files {
        // binary files aren't allowed, check if the file is binary
        let mut first_few_bytes = [0; 128];
        let n = file
            .read(&mut first_few_bytes)
            .context("Failed to read from file")?;

        // file is empty, return early
        if n == 0 {
            if !args.plain {
                println!("--- EMPTY FILE ---");
            }
            return Ok(());
        }

        let mut first_few_bytes = &first_few_bytes[..n];
        if content_inspector::inspect(first_few_bytes).is_binary() {
            anyhow::bail!(
                "`{}` is a binrary file, use `--allow-binary-files` to suppress this error",
                args.file.display()
            );
        }

        // count the number of lines in the first few bytes
        while first_few_bytes
            .skip_until(b'\n')
            .context("Failed to read from file")?
            > 0
        {
            n_lines += 1;
        }
    }
    // count the number of lines in the remainder of the file
    while file.skip_until(b'\n').context("Failed to read from file")? > 0 {
        n_lines += 1;
    }
    // TODO: support seek for stdin https://github.com/rust-lang/rust/issues/72802#issuecomment-1101996578
    // and https://github.com/uutils/coreutils/pull/4189/files#diff-bd7f28594a45798eed07dea6767fc2bb5cb29e2d2855366ba65b126248bfd4b9R128-R132
    file.rewind().context("Failed to rewind file")?;

    // parse line selectors
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
                if let Entry::Vacant(entry) = lines.entry(line_num) {
                    let line = read_line(&mut line_reader, line_num)?;
                    entry.insert(line);
                }
            }
            ParsedLineSelector::Range(lower, upper, step) => {
                let line_nums = if step > 0 {
                    (lower..=upper).step_by(step.unsigned_abs())
                } else {
                    (upper..=lower).step_by(step.unsigned_abs())
                };

                for line_num in line_nums {
                    if let Entry::Vacant(entry) = lines.entry(line_num) {
                        let line = read_line(&mut line_reader, line_num)?;
                        entry.insert(line);
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
                print_line(&lines[&line_num])?;
            }
            ParsedLineSelector::Range(lower, upper, step) => {
                let abs_step = step.unsigned_abs();
                let mut curr = lower;
                if step > 0 {
                    while curr <= upper {
                        print_line(&lines[&curr])?;
                        curr += abs_step;
                    }
                } else {
                    while curr >= upper {
                        print_line(&lines[&curr])?;
                        curr -= abs_step;
                    }
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
    line_reader: &mut LineReader<R>,
    line_num: usize,
) -> anyhow::Result<Vec<u8>> {
    let mut lin_buf = Vec::new();
    line_reader
        .read_specific_line(&mut lin_buf, line_num)
        .with_context(|| format!("Failed to read line number {line_num}"))?;
    Ok(lin_buf)
}

#[cfg(test)]
mod tests {
    // use super::*;
    //
    // mod n_selected_lines {
    //     use super::*;
    //     use crate::line_selector::OriginalLineSelector;
    //
    //     macro_rules! create_line_selector {
    //         ($s: literal, $n_lines: literal) => {{
    //             let original = OriginalLineSelector::from_str($s).unwrap();
    //             LineSelector::from_original(original, $n_lines)
    //         }};
    //     }
    //
    //     #[test]
    //     fn b_lower_is_a_lower() {
    //         let a = create_line_selector!("2:7", 42).unwrap();
    //         let b = create_line_selector!("2", 42).unwrap();
    //         let mut v = [a, b];
    //         v.sort();
    //         assert_eq!(n_selected_lines(&v), 6);
    //
    //         let a = create_line_selector!("2:7", 42).unwrap();
    //         let b = create_line_selector!("2:5", 42).unwrap();
    //         let mut v = [a, b];
    //         v.sort();
    //         assert_eq!(n_selected_lines(&v), 6);
    //
    //         let a = create_line_selector!("2:7", 42).unwrap();
    //         let b = create_line_selector!("2:7", 42).unwrap();
    //         let mut v = [a, b];
    //         v.sort();
    //         assert_eq!(n_selected_lines(&v), 6);
    //
    //         let a = create_line_selector!("2:7", 42).unwrap();
    //         let b = create_line_selector!("2:9", 42).unwrap();
    //         let mut v = [a, b];
    //         v.sort();
    //         assert_eq!(n_selected_lines(&v), 8);
    //
    //         let a = create_line_selector!("3", 42).unwrap();
    //         let b = create_line_selector!("3", 42).unwrap();
    //         let mut v = [a, b];
    //         v.sort();
    //         assert_eq!(n_selected_lines(&v), 1);
    //
    //         let a = create_line_selector!("3", 42).unwrap();
    //         let b = create_line_selector!("3:5", 42).unwrap();
    //         let mut v = [a, b];
    //         v.sort();
    //         assert_eq!(n_selected_lines(&v), 3);
    //     }
    //
    //     #[test]
    //     fn b_lower_is_inside_a() {
    //         let a = create_line_selector!("2:7", 42).unwrap();
    //         let b = create_line_selector!("4", 42).unwrap();
    //         let mut v = [a, b];
    //         v.sort();
    //         assert_eq!(n_selected_lines(&v), 6);
    //
    //         let a = create_line_selector!("2:7", 42).unwrap();
    //         let b = create_line_selector!("4:6", 42).unwrap();
    //         let mut v = [a, b];
    //         v.sort();
    //         assert_eq!(n_selected_lines(&v), 6);
    //
    //         let a = create_line_selector!("2:7", 42).unwrap();
    //         let b = create_line_selector!("4:7", 42).unwrap();
    //         let mut v = [a, b];
    //         v.sort();
    //         assert_eq!(n_selected_lines(&v), 6);
    //
    //         let a = create_line_selector!("2:7", 42).unwrap();
    //         let b = create_line_selector!("4:9", 42).unwrap();
    //         let mut v = [a, b];
    //         v.sort();
    //         assert_eq!(n_selected_lines(&v), 8);
    //     }
    //
    //     #[test]
    //     fn b_lower_is_a_upper() {
    //         let a = create_line_selector!("2:6", 42).unwrap();
    //         let b = create_line_selector!("6", 42).unwrap();
    //         let mut v = [a, b];
    //         v.sort();
    //         assert_eq!(n_selected_lines(&v), 5);
    //
    //         let a = create_line_selector!("2:6", 42).unwrap();
    //         let b = create_line_selector!("6:8", 42).unwrap();
    //         let mut v = [a, b];
    //         v.sort();
    //         assert_eq!(n_selected_lines(&v), 7);
    //     }
    //
    //     #[test]
    //     fn b_lower_is_outside_a() {
    //         let a = create_line_selector!("2:6", 42).unwrap();
    //         let b = create_line_selector!("7", 42).unwrap();
    //         let mut v = [a, b];
    //         v.sort();
    //         assert_eq!(n_selected_lines(&v), 6);
    //
    //         let a = create_line_selector!("2:6", 42).unwrap();
    //         let b = create_line_selector!("7:9", 42).unwrap();
    //         let mut v = [a, b];
    //         v.sort();
    //         assert_eq!(n_selected_lines(&v), 8);
    //
    //         let a = create_line_selector!("3", 42).unwrap();
    //         let b = create_line_selector!("5", 42).unwrap();
    //         let mut v = [a, b];
    //         v.sort();
    //         assert_eq!(n_selected_lines(&v), 2);
    //
    //         let a = create_line_selector!("3", 42).unwrap();
    //         let b = create_line_selector!("5:7", 42).unwrap();
    //         let mut v = [a, b];
    //         v.sort();
    //         assert_eq!(n_selected_lines(&v), 4);
    //     }
    // }
}
