// TODO: remove this
#![allow(dead_code)]

use crate::cli::Cli;
use anyhow::{Context, Result};
use clap::Parser;
use content_inspector::ContentType;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek};

mod cli;

fn main() -> Result<()> {
    let args = Cli::parse();

    if args.line == 0 {
        anyhow::bail!("Line number can't be zero");
    }

    let file =
        File::open(&args.file).with_context(|| format!("Failed to open file `{}`", args.file))?;
    if !file
        .metadata()
        .context("Failed to extract metadata of file")?
        .is_file()
    {
        anyhow::bail!("{} is not a file", args.file);
    }
    let file = BufReader::new(file);

    // TODO: handel non UTF-8 lines
    // TODO: calling nth allocates a string args.line - 1 times, find a better way to skip lines
    // ig a simple way is to read chars until '\n' or '\r\n' is found
    let mut lines = file.lines();
    if let Some(line) = lines.nth(args.line - 1) {
        let line = line.context("Failed to read line")?;
        println!("{line}");
    } else {
        anyhow::bail!("Line {} is out of bound", args.line);
    }

    Ok(())
}

struct FileReader {
    reader: BufReader<File>,
    current_line: usize,
    content_type: ContentType,
}

impl FileReader {
    fn new(mut reader: BufReader<File>) -> Result<Self> {
        let mut first_line = Vec::new();
        reader
            .read_until(b'\n', &mut first_line)
            .context("Failed to read first line from file")?;
        reader.rewind().context("Failed to rewind file")?;

        let content_type = content_inspector::inspect(&first_line);

        Ok(Self {
            reader,
            current_line: 0,
            content_type,
        })
    }

    /// Returns `false` if no bytes were read and `true` otherwise.
    fn read_next_line(&mut self, buf: &mut Vec<u8>) -> anyhow::Result<bool> {
        let n = self.reader.read_until(b'\n', buf)?;
        if n == 0 {
            return Ok(false);
        }
        self.current_line += 1;
        Ok(true)
    }

    /// Skips `n` lines.
    /// returns `false` if reached EOF before skipping `n` lines.
    fn skip_lines(&mut self, n: usize) -> anyhow::Result<bool> {
        let mut i = 0;
        while i < n && self.reader.skip_until(b'\n')? > 0 {
            i += 1;
        }
        self.current_line += i;
        Ok(i == n)
    }

    /// `lines_num` should be more than `self.current_line`.
    /// `line_num` is zero-indexed.
    /// returns `true` if `line_num` is beyod EOF.
    fn read_specific_line(&mut self, buf: &mut Vec<u8>, line_num: usize) -> anyhow::Result<bool> {
        if !self.skip_lines(line_num - self.current_line)? {
            return Ok(false);
        }
        self.read_next_line(buf)
    }
}
