use anyhow::Context;
use clap::Parser;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// line number to extract
    #[arg(short = 'n', long)]
    line: usize,

    file: String,
}

// TODO: support negative indexing (like in Python)
// TODO: support a flag `--force` to print last line in case
// N is beyond the max lines and first line if line is negative
// TODO: add a flag to print to stderr instead of stdout
// TODO: use pretty print (consider using olive!), something similar to `bat` style, but add a flag
// to make it plain, and make it plain by default if redirection is detected (check how bat does
// that)
// TODO: add a flag to switch zero/one index
// TODO: write test cases
// TODO: add syntax for range (eg: -n 1:4)
// TODO: add syntax for multiple lines (eg: -n 1,4)

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // TODO: handle ~ and symbolic links
    let file =
        File::open(&args.file).with_context(|| format!("Failed to open file `{}`", args.file))?;
    let file = BufReader::new(file);
    if let Some(line) = file.lines().nth(args.line - 1) {
        let line = line.context("Failed to read line")?;
        println!("{line}");
    } else {
        anyhow::bail!("Line {} is too large", args.line);
    }

    Ok(())
}
