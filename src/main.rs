use crate::cli::Cli;
use crate::line_reader::LineReader;
use crate::line_selector::{ParsedLineSelector, RawLineSelector};
use crate::output::Line;
use anyhow::{Context, Result};
use clap::Parser;
use std::collections::{HashMap, hash_map::Entry};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, IsTerminal, Read, Seek};
use std::path::Path;

mod cli;
mod line_reader;
mod line_selector;
mod output;

fn main() -> Result<()> {
    let mut args = Cli::parse();

    let file = open_file(&args.file)?;
    let mut file = BufReader::new(file);

    if !args.allow_binary_files {
        bail_if_binrary(&mut file, &args.file)?;
    }

    let n_lines = count_lines(&mut file)?;
    let line_selectors = parse_line_selectors(&args.raw_line_selectors, n_lines)?;

    let mut sorted_line_selectors = line_selectors.clone();
    sorted_line_selectors.sort_unstable();

    // if `--context` is set (i.e. not 0), then `--context=N` is equivalent
    // to `--before=N --after=N`
    if args.context != 0 {
        args.before = args.context;
        args.after = args.context;
    }

    let mut line_reader = LineReader::new(file);

    // TODO: benchmark to check if using a Vec + binary search is better than using a hash map
    // read and store selected lines
    let mut lines: HashMap<usize, Vec<u8>> = HashMap::new();
    for line_selector in sorted_line_selectors {
        match line_selector {
            ParsedLineSelector::Single(selected_line_num) => {
                read_line_with_context(
                    &mut line_reader,
                    &mut lines,
                    selected_line_num,
                    args.before,
                    args.after,
                    n_lines,
                )?;
            }
            ParsedLineSelector::Range(start, end, step) => {
                let selected_line_nums = if step > 0 {
                    (start..=end).step_by(step.unsigned_abs())
                } else {
                    (end..=start).step_by(step.unsigned_abs())
                };

                for selected_line_num in selected_line_nums {
                    // TODO: optimize this: when you have a range, say, 4:10 with -c=2, you don't
                    // need an inner loop for the lines 5..=9, you can read the lines 1..=7 then
                    // read two lines before 4 and two lines after 10. this will reduce the number
                    // of hashes. but watch out when the step is not 1.
                    read_line_with_context(
                        &mut line_reader,
                        &mut lines,
                        selected_line_num,
                        args.before,
                        args.after,
                        n_lines,
                    )?;
                }
            }
        }
    }

    let stdout = std::io::stdout().lock();
    let is_terminal = stdout.is_terminal();
    let stdout = BufWriter::new(stdout);
    let mut output = output::get_output_writer(stdout, args.color, args.plain, is_terminal);

    // print selected lines
    for line_selector in line_selectors {
        output
            .print_line_selector_header(&line_selector)
            .context("Failed to output header")?;
        match line_selector {
            ParsedLineSelector::Single(selected_line_num) => {
                let line_nums =
                    get_line_nums_with_context(selected_line_num, args.before, args.after, n_lines);

                for line_num in line_nums {
                    let line = &lines[&line_num];
                    let line = if line_num == selected_line_num {
                        Line::Selected { line_num, line }
                    } else {
                        Line::Context { line_num, line }
                    };
                    output
                        .print_line(line)
                        .with_context(|| format!("Failed to output line {}", line_num + 1))?;
                }
            }
            ParsedLineSelector::Range(start, end, step) => {
                let update_fn = if step > 0 {
                    std::ops::AddAssign::add_assign
                } else {
                    std::ops::SubAssign::sub_assign
                };

                let step_abs = step.unsigned_abs();

                // TODO: handel cases when args.before != args.after
                let mut line_num = start;
                loop {
                    // TODO: maybe `get_line_nums_with_context` can be used to get the context lines

                    // print context lines (before)
                    for line_num in line_num.saturating_sub(args.before)..line_num {
                        output
                            .print_line(Line::Context {
                                line_num,
                                line: &lines[&line_num],
                            })
                            .with_context(|| format!("Failed to output line {}", line_num + 1))?;
                    }

                    // print the selected line
                    output
                        .print_line(Line::Selected {
                            line_num,
                            line: &lines[&line_num],
                        })
                        .with_context(|| format!("Failed to output line {}", line_num + 1))?;

                    // print context lines (after)
                    for line_num in (line_num + 1)..=(line_num + args.after).min(n_lines) {
                        output
                            .print_line(Line::Context {
                                line_num,
                                line: &lines[&line_num],
                            })
                            .with_context(|| format!("Failed to output line {}", line_num + 1))?;
                    }

                    if line_num == end {
                        break;
                    }
                    if args.context != 0 {
                        writeln!(output)?;
                    }
                    update_fn(&mut line_num, step_abs);
                }
            }
        }
        writeln!(output)?;
    }

    Ok(())
}

/// Reads the line `selected_line_num` and it's context line, storing the line in `lines`. If the
/// line is already in `lines`, then the line will not be read.
fn read_line_with_context(
    line_reader: &mut LineReader<BufReader<File>>,
    lines: &mut HashMap<usize, Vec<u8>>,
    selected_line_num: usize,
    before: usize,
    after: usize,
    n_lines: usize,
) -> anyhow::Result<()> {
    let context_line_nums = get_line_nums_with_context(selected_line_num, before, after, n_lines);

    // read context lines
    for context_line_num in context_line_nums {
        if let Entry::Vacant(entry) = lines.entry(context_line_num) {
            let mut line = Vec::new();
            line_reader
                .read_specific_line(&mut line, context_line_num)
                .with_context(|| format!("Failed to read line number {context_line_num}"))?;
            entry.insert(line);
        }
    }

    Ok(())
}

/// Parses a slice of `RawLineSelector` into a slice of `ParsedLineSelector`
fn parse_line_selectors(
    raw_line_selectors: &[RawLineSelector],
    n_lines: usize,
) -> anyhow::Result<Box<[ParsedLineSelector]>> {
    raw_line_selectors
        .iter()
        .map(|&raw_line_selector| {
            ParsedLineSelector::from_raw(raw_line_selector, n_lines)
                .with_context(|| format!("Invalid line selector: {raw_line_selector}"))
        })
        .collect()
}

/// Opens a file and bails if the file is a directory or empty
fn open_file(path: &Path) -> anyhow::Result<File> {
    let file =
        File::open(path).with_context(|| format!("Couldn't open file `{}`", path.display()))?;

    let metadata = file
        .metadata()
        .with_context(|| format!("Couldn't read file metadata of `{}`", path.display()))?;

    if !metadata.is_file() {
        anyhow::bail!("`{}` is not a file", path.display());
    } else if metadata.len() == 0 {
        anyhow::bail!("`{}` is an empty file", path.display());
    }

    Ok(file)
}

/// Counts the number of lines in the file then rewinds to the begining of the file
fn count_lines(file: &mut BufReader<File>) -> anyhow::Result<usize> {
    let mut n_lines = 0;
    while file.skip_until(b'\n').context("Failed to read from file")? > 0 {
        n_lines += 1;
    }
    file.rewind().context("Failed to rewind file")?;
    Ok(n_lines)
}

/// Checks if `file` is binary by inspecing the first few bytes, then bails if it is
fn bail_if_binrary(file: &mut BufReader<File>, path: &Path) -> anyhow::Result<()> {
    let mut first_few_bytes = [0; 64];
    let n = file
        .read(&mut first_few_bytes)
        .context("Failed to read from file")?;
    let first_few_bytes = &first_few_bytes[..n];

    if content_inspector::inspect(first_few_bytes).is_binary() {
        anyhow::bail!(
            "file '{}' appears to be a binary file (use --allow-binary-files to override)",
            path.display()
        );
    }

    // we read a small amount of bytes, so rewinding shouldn't be expensive due to caching
    file.rewind().context("Failed to rewind file")?;

    Ok(())
}

/// Returns `selected_line_num` along with its context line numbers, that is: all line numbers
/// between `selected_line_num - before` and `selected_line_num + after`, capped between 0 and
/// n_lines - 1.
fn get_line_nums_with_context(
    selected_line_num: usize,
    before: usize,
    after: usize,
    n_lines: usize,
) -> impl Iterator<Item = usize> {
    debug_assert!(n_lines > 0); // ensures n_lines - 1 is safe
    let first_context_line = selected_line_num.saturating_sub(before);
    let last_context_line = selected_line_num.saturating_add(after).min(n_lines - 1);
    first_context_line..=last_context_line
}
