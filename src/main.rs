use crate::cli::Cli;
use crate::line_reader::LineReader;
use crate::line_selector::{ParsedLineSelector, RawLineSelector};
use crate::output::Line;
use anyhow::{Context, Result};
use clap::Parser;
use std::collections::HashMap;
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

    // if `--context` is set (i.e. not 0), then `--context=N` is equivalent
    // to `--before=N --after=N`
    if args.context != 0 {
        args.before = args.context;
        args.after = args.context;
    }

    // populate hash map with selected line numbers
    let mut lines: HashMap<usize, Vec<u8>> = HashMap::new();
    for line_selector in &line_selectors {
        for line_num in line_selector.iter() {
            lines.entry(line_num).or_default();
        }
    }
    let mut sorted_line_nums: Box<[usize]> = lines.keys().copied().collect();
    sorted_line_nums.sort_unstable();

    // TODO: optimize this: when you have a range, say 4:10 with -c=2, you don't need an inner
    // loop, you can read the lines 4..=10 then read two lines before 4 and two lines after 10. no
    // need to read line 4 and it's context (2..=6) then line 5 and it's context (3..=7), etc. as
    // this will lead to many redundancy and will increse the number of hashes. this optimization
    // can be applied when there is an overalp, which happens when `2 * context > step - 1`.

    // read selected lines
    let mut line_reader = LineReader::new(file);
    for line_num in sorted_line_nums {
        let line_num_with_context =
            get_line_num_with_context(line_num, args.before, args.after, n_lines);

        for line_num in line_num_with_context {
            let line_buf = lines
                .get_mut(&line_num)
                .expect("we already inserted all line numbers into the hash map");
            line_reader
                .read_specific_line(line_buf, line_num)
                .with_context(|| format!("Failed to read line number {}", line_num + 1))?;
        }
    }

    // print selected lines
    let stdout = std::io::stdout().lock();
    let is_terminal = stdout.is_terminal();
    let stdout = BufWriter::new(stdout);
    let mut output = output::get_output_writer(stdout, args.color, args.plain, is_terminal);

    for (i, line_selector) in line_selectors.iter().enumerate() {
        output
            .print_line_selector_header(line_selector)
            .context("Failed to output header")?;
        match *line_selector {
            ParsedLineSelector::Single(selected_line_num) => {
                let line_nums =
                    get_line_num_with_context(selected_line_num, args.before, args.after, n_lines);

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
                let mut selected_line_num = start;
                loop {
                    let line_nums = get_line_num_with_context(
                        selected_line_num,
                        args.before,
                        args.after,
                        n_lines,
                    );
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

                    if selected_line_num == end {
                        break;
                    }
                    if args.context != 0 {
                        writeln!(output)?;
                    }
                    update_fn(&mut selected_line_num, step_abs);
                }
            }
        }
        if i != line_selectors.len() - 1 {
            writeln!(output)?;
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
fn get_line_num_with_context(
    selected_line_num: usize,
    before: usize,
    after: usize,
    n_lines: usize,
) -> impl Iterator<Item = usize> {
    debug_assert!(n_lines > 0); // ensures n_lines - 1 doesn't panic
    let first_context_line = selected_line_num.saturating_sub(before);
    let last_context_line = selected_line_num.saturating_add(after).min(n_lines - 1);
    first_context_line..=last_context_line
}
