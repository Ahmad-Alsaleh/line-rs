use crate::cli::Cli;
use crate::line_reader::LineReader;
use crate::line_selector::ParsedLineSelector;
use anyhow::{Context, Result};
use clap::Parser;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, StdoutLock, Write};
use std::path::Path;

mod cli;
mod line_reader;
mod line_selector;

struct Line {
    content: Vec<u8>,
    color: bool,
}

fn main() -> Result<()> {
    let mut args = Cli::parse();

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
                // TODO: use pretty pretty printing
                println!("--- EMPTY FILE ---");
            }
            return Ok(());
        }

        let mut first_few_bytes = &first_few_bytes[..n];
        if content_inspector::inspect(first_few_bytes).is_binary() {
            anyhow::bail!(
                "file '{}' appears to be a binary file (use --allow-binary-files to override)",
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
        .raw_line_selectors
        .into_iter()
        .map(|raw_line_selector| {
            ParsedLineSelector::from_raw(raw_line_selector, n_lines)
                .with_context(|| format!("Invalid line selector: {raw_line_selector}"))
        })
        .collect();
    let line_selectors = line_selectors?;

    let mut sorted_line_selectors = line_selectors.clone();
    sorted_line_selectors.sort_unstable();

    if args.context != 0 {
        args.before = args.context;
        args.after = args.context;
    }

    let mut line_reader = LineReader::new(file);

    // TODO: benchmark to check if using a Vec + binary search is better than using a hash map
    // read and store selected lines
    let mut lines: HashMap<usize, Line> = HashMap::new();
    for line_selector in sorted_line_selectors {
        match line_selector {
            ParsedLineSelector::Single(selected_line_num) => {
                // read context lines
                let first_context_line = selected_line_num.saturating_sub(args.before);
                let last_context_line = selected_line_num.saturating_add(args.after).min(n_lines);
                for context_line_num in first_context_line..=last_context_line {
                    if let Entry::Vacant(entry) = lines.entry(context_line_num) {
                        let line = read_line(&mut line_reader, context_line_num)?;
                        entry.insert(Line {
                            content: line,
                            color: false,
                        });
                    }
                }
                // change color of selected line
                lines
                    .entry(selected_line_num)
                    .and_modify(|line| line.color = true);
            }
            ParsedLineSelector::Range(start, end, step) => {
                let selected_line_nums = if step > 0 {
                    (start..=end).step_by(step.unsigned_abs())
                } else {
                    (end..=start).step_by(step.unsigned_abs())
                };

                for selected_line_num in selected_line_nums {
                    // read context lines
                    let first_context_line = selected_line_num.saturating_sub(args.before);
                    let last_context_line =
                        selected_line_num.saturating_add(args.after).min(n_lines);
                    // TODO: optimize this: when you have a range, say, 4:10 with -c=2, you don't
                    // need an inner loop for the lines 5..=9, you can read the lines 1..=7 then
                    // read two lines before 4 and two lines after 10. but watch out when the step
                    // is not 1.
                    for context_line_num in first_context_line..=last_context_line {
                        if let Entry::Vacant(entry) = lines.entry(context_line_num) {
                            let line = read_line(&mut line_reader, context_line_num)?;
                            entry.insert(Line {
                                content: line,
                                color: false,
                            });
                        }
                    }
                    // change color of selected line
                    lines
                        .entry(selected_line_num)
                        .and_modify(|line| line.color = true);
                }
            }
        }
    }

    // print selected lines
    let mut stdout = std::io::stdout().lock();
    for line_selector in line_selectors {
        // TODO: implement Display for ParsedLineSelector. Maybe print Range(a, b, c) as `a:b:c (a, a + c, ..., b)`
        // but make sure to use one-based for the line numbers
        writeln!(stdout, "\n------ Line: {line_selector:?} ------")?;
        match line_selector {
            ParsedLineSelector::Single(selected_line_num) => {
                let first_line = selected_line_num.saturating_sub(args.before);
                let last_line = selected_line_num.saturating_add(args.after).min(n_lines);
                for line_num in first_line..=last_line {
                    print_line(&lines[&line_num], line_num, &mut stdout)?;
                }
            }
            ParsedLineSelector::Range(start, end, step) => {
                let update_fn = if step > 0 {
                    std::ops::AddAssign::add_assign
                } else {
                    std::ops::SubAssign::sub_assign
                };

                // TODO: print context lines
                let abs_step = step.unsigned_abs();
                let mut curr_line_num = start;
                loop {
                    print_line(&lines[&curr_line_num], curr_line_num, &mut stdout)?;
                    if curr_line_num == end {
                        break;
                    }
                    update_fn(&mut curr_line_num, abs_step);
                }
            }
        }
    }

    Ok(())
}

fn print_line(line: &Line, line_num: usize, stdout: &mut StdoutLock) -> anyhow::Result<()> {
    // TODO (FIXME): handle SIGPIPE, eg: `line -n=: large_file.txt | head -n1`
    write!(stdout, "{}: ", line_num + 1)?;
    if line.color {
        // TODO: make this cross-platform
        write!(stdout, "\x1b[31m")?;
    }
    stdout.write_all(&line.content)?;
    if line.color {
        // TODO: make this cross-platform
        write!(stdout, "\x1b[0m")?;
    }
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
