use crate::line_selector::RawLineSelector;
use clap::Parser;
use std::path::PathBuf;

// TODO: consider using https://github.com/Canop/clap-help
#[derive(Parser, Debug)]
#[command(version, about="Extract lines without hacks", author, long_about = None, next_line_help = true)]
pub(crate) struct Cli {
    /// Line number(s) to extract
    #[arg(short = 'n', long = "line", value_name = "LINE-SELECTORS", value_parser = RawLineSelector::from_str, value_delimiter = ',', required = true)]
    pub(crate) raw_line_selectors: Vec<RawLineSelector>,

    /// Treat binary files as text files
    #[arg(long)]
    pub(crate) allow_binary_files: bool,

    ///Only show plain style, no decorations or line numbers
    #[arg(short, long)]
    pub(crate) plain: bool,

    /// Show N lines before the selected line
    #[arg(long, short, value_name = "N", default_value_t = 0)]
    pub(crate) before: usize,

    /// Show N lines after the selected line
    #[arg(long, short, value_name = "N", default_value_t = 0)]
    pub(crate) after: usize,

    /// Show context lines around the selected line
    /// Equivalent to setting both --after and --before to the same value
    #[arg(
        long,
        short,
        default_value_t = 0,
        conflicts_with = "before",
        conflicts_with = "after",
        value_name = "N",
        verbatim_doc_comment
    )]
    pub(crate) context: usize,

    /// File to extract line(s) from. Use a dash ('-') or no argument to read from standard input
    pub(crate) file: PathBuf,
}
