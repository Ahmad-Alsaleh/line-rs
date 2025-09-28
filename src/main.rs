use crate::cli::Cli;
use crate::line_reader::LineReader;
use crate::line_selector::{LineSelector, ParsedLineSelector, RawLineSelector};
use crate::output::{Line, OutputWriter};
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

    // store the line numbers of all lines to be read (selected lines and context lines)
    let mut lines: HashMap<usize, Vec<u8>> = HashMap::new();
    for line_selector in &line_selectors {
        for selected_line_num in line_selector.iter() {
            let (first_context_line, last_context_line) =
                get_context_lines_endpoints(selected_line_num, args.before, args.after, n_lines);
            for line_num in first_context_line..=last_context_line {
                lines.entry(line_num).or_default();
            }
        }
    }
    let mut line_nums_to_read: Box<[usize]> = lines.keys().copied().collect();
    line_nums_to_read.sort_unstable();

    // TODO: optimize this: when you have a range, say 4:10 with -c=2, you don't need an inner
    // loop, you can read the lines 4..=10 then read two lines before 4 and two lines after 10. no
    // need to read line 4 and it's context (2..=6) then line 5 and it's context (3..=7), etc. as
    // this will lead to many redundancy and will increse the number of hashes. this optimization
    // can be applied when there is an overalp, which happens when `2 * context > step - 1`.

    // read selected lines
    let mut line_reader = LineReader::new(file);
    for line_num in line_nums_to_read {
        let line_buf = lines
            .get_mut(&line_num)
            .expect("we already inserted all line numbers into the hash map");
        line_reader
            .read_specific_line(line_buf, line_num)
            .with_context(|| format!("Failed to read line number {}", line_num + 1))?;
    }

    // print selected lines
    let stdout = std::io::stdout().lock();
    let is_terminal = stdout.is_terminal();
    let stdout = BufWriter::new(stdout);
    let mut output = output::get_output_writer(stdout, args.color, args.plain, is_terminal);

    let mut is_first = true;
    for line_selector in line_selectors {
        output
            .print_line_selector_header(&line_selector, is_first)
            .context("Failed to output header")?;
        is_first = false;

        let (start, end, step) = match line_selector.parsed {
            ParsedLineSelector::Single(line_num) => (line_num, line_num, 1),
            ParsedLineSelector::Range(start, end, step) => (start, end, step),
        };

        let update_fn = if step > 0 {
            std::ops::AddAssign::add_assign
        } else {
            std::ops::SubAssign::sub_assign
        };
        let step_abs = step.unsigned_abs();

        // TODO: handel cases when args.before != args.after
        let mut selected_line_num = start;
        loop {
            print_line_and_its_context(
                selected_line_num,
                args.before,
                args.after,
                n_lines,
                &lines,
                &mut output,
            )?;
            if selected_line_num == end {
                break;
            }
            if args.context != 0 {
                writeln!(output)?;
            }
            update_fn(&mut selected_line_num, step_abs);
        }
    }

    Ok(())
}

fn print_line_and_its_context(
    selected_line_num: usize,
    before: usize,
    after: usize,
    n_lines: usize,
    lines: &HashMap<usize, Vec<u8>>,
    output: &mut Box<dyn OutputWriter>,
) -> Result<(), anyhow::Error> {
    fn print_context_lines(
        context_line_nums: impl Iterator<Item = usize>,
        lines: &HashMap<usize, Vec<u8>>,
        output: &mut Box<dyn OutputWriter>,
    ) -> anyhow::Result<()> {
        for line_num in context_line_nums {
            let line = Line::Context {
                line_num,
                line: &lines[&line_num],
            };
            output
                .print_line(line)
                .with_context(|| format!("Failed to output line {}", line_num + 1))?;
        }
        Ok(())
    }

    let (context_before, context_after) =
        get_context_lines(selected_line_num, before, after, n_lines);

    print_context_lines(context_before, lines, output)?;

    let line = Line::Selected {
        line_num: selected_line_num,
        line: &lines[&selected_line_num],
    };
    output
        .print_line(line)
        .with_context(|| format!("Failed to output line {}", selected_line_num + 1))?;

    print_context_lines(context_after, lines, output)?;

    Ok(())
}

/// Parses a slice of `RawLineSelector`s into a slice of `LineSelector`
fn parse_line_selectors(
    raw_line_selectors: &[RawLineSelector],
    n_lines: usize,
) -> anyhow::Result<Box<[LineSelector]>> {
    raw_line_selectors
        .iter()
        .map(|&raw_line_selector| {
            let parsed_line_selector = ParsedLineSelector::from_raw(raw_line_selector, n_lines)
                .with_context(|| format!("Invalid line selector: {raw_line_selector}"))?;

            Ok(LineSelector {
                parsed: parsed_line_selector,
                raw: raw_line_selector,
            })
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

/// Returns the context lines before and after the `selected_line_num` as iterators, capped
/// between 0 and n_lines - 1.
fn get_context_lines(
    selected_line_num: usize,
    before: usize,
    after: usize,
    n_lines: usize,
) -> (impl Iterator<Item = usize>, impl Iterator<Item = usize>) {
    let (first_context_line, last_context_line) =
        get_context_lines_endpoints(selected_line_num, before, after, n_lines);

    let before = first_context_line..selected_line_num;
    let after = (selected_line_num + 1)..=last_context_line;

    (before, after)
}

/// Returns the first and last context lines of `selected_line_num`, capped between 0 and
/// n_lines - 1.
fn get_context_lines_endpoints(
    selected_line_num: usize,
    before: usize,
    after: usize,
    n_lines: usize,
) -> (usize, usize) {
    debug_assert!(n_lines > 0); // ensures `n_lines - 1` doesn't panic/underflow
    let first_context_line = selected_line_num.saturating_sub(before);
    let last_context_line = selected_line_num.saturating_add(after).min(n_lines - 1);
    (first_context_line, last_context_line)
}
