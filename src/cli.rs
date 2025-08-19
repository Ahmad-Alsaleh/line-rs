use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about="Extract lines without hacks", author, long_about = None)]
pub(crate) struct Cli {
    /// Line number(s) to extract
    #[arg(short = 'n', long = "line", value_delimiter = ',', required = true)]
    pub(crate) line_selectors: Vec<String>,

    /// File to extract line(s) from. Use a dash ('-') or no argument to read from standard input
    pub(crate) file: PathBuf,

    /// Treat binary files as text files
    #[arg(long)]
    pub(crate) allow_binary_files: bool,
}
