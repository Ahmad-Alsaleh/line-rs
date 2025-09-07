use crate::line_selector::RawLineSelector;
use clap::Parser;
use std::path::PathBuf;

// TODO: consider using https://github.com/Canop/clap-help
#[derive(Parser, Debug)]
#[command(
    version, 
    about="Extract specific lines from text files with powerful indexing",
    author, 
    long_about = "A fast, flexible tool for extracting lines from text files using Python-like \
                 indexing.\nSupports ranges, steps, and backward counting.",
    next_line_help = true
)]
pub(crate) struct Cli {
    /// Line number(s) to extract. Supports ranges (1:5), ranges with steps (1:10:2),
    /// unbound ranges (5:), negative indices for backward counting, and combinations (1,5:3:-1,:7)
    #[arg(
        short = 'n', 
        long = "line", 
        value_name = "LINE_SELECTORS", 
        value_parser = RawLineSelector::from_str, 
        value_delimiter = ',', 
        required = true,
        help_heading = "Selection"
    )]
    pub(crate) raw_line_selectors: Vec<RawLineSelector>,

    /// Process binary files as text (default: reject binary files)
    #[arg(long, help_heading = "Input")]
    pub(crate) allow_binary_files: bool,

    // TODO: make this an enum Color {On, Off, Auto}, default should be auto, which turns colors
    // off if the output is not a terminal or when an env var is set
    /// Do not output colors
    #[arg(long, help_heading = "Output")]
    pub(crate) no_color: bool,

    /// Output plain text without any decorations or line numbers
    #[arg(short, long, help_heading = "Output")]
    pub(crate) plain: bool,

    /// Show N lines before each selected line
    #[arg(long, short, value_name = "N", default_value_t = 0, help_heading = "Context")]
    pub(crate) before: usize,

    /// Show N lines after each selected line  
    #[arg(long, short, value_name = "N", default_value_t = 0, help_heading = "Context")]
    pub(crate) after: usize,

    /// Show N context lines around each selected line (equivalent to --before=N --after=N)
    #[arg(
        long,
        short,
        default_value_t = 0,
        conflicts_with_all = ["before", "after"],
        value_name = "N",
        help_heading = "Context"
    )]
    pub(crate) context: usize,

    /// Input file (use '-' for stdin)
    #[arg(value_name = "FILE")]
    pub(crate) file: PathBuf,
}
