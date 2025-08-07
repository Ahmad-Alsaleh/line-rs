use crate::cli::Cli;
use anyhow::{Context, Result};
use clap::Parser;
use content_inspector::ContentType;
use std::any;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};

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

    let mut lines = file.lines();
    // TODO: handel non UTF-8 lines
    // TODO: calling nth allocates a string args.line - 1 times, find a better way to skip lines
    // ig a simple way is to read chars until '\n' or '\r\n' is found
    if let Some(line) = lines.nth(args.line - 1) {
        let line = line.context("Failed to read line")?;
        println!("{line}");
    } else {
        anyhow::bail!("Line {} is out of bound", args.line);
    }

    Ok(())
}

struct FileReader {
    first_line: Vec<u8>,
    reader: BufReader<File>,
    content_type: Option<ContentType>,
}

impl FileReader {
    fn new(mut reader: BufReader<File>) -> Result<Self> {
        let mut first_line = Vec::new();
        reader
            .read_until(b'\n', &mut first_line)
            .context("Failed to read first line from file")?;

        let content_type = if first_line.is_empty() {
            None
        } else {
            Some(content_inspector::inspect(&first_line))
        };

        Ok(Self {
            first_line,
            reader,
            content_type,
        })
    }

    fn read_line(&mut self, buf: &mut Vec<u8>) -> anyhow::Result<bool> {
        if !self.first_line.is_empty() {
            buf.append(&mut self.first_line);
            return Ok(true);
        };

        let bytes_read = self.reader.read_until(b'\n', buf).map(|n| n > 0)?;
        Ok(bytes_read)
    }

    fn skip_n_lines(&mut self, n: usize) -> anyhow::Result<bool> {
        if n == 0 {
            return Ok(true);
        }

        if !self.first_line.is_empty() {
            std::mem::take(&mut self.first_line.clear());
        }
        for _ in 1..n {
            let n = self.reader.skip_until(b'\n')?;
            if n == 0 {
                return Ok(false);
            }
        }

        Ok(true)
    }
}
