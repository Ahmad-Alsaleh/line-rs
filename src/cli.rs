use crate::line_selector::RawLineSelector;
use clap::{ArgAction, Parser, ValueEnum};
use std::path::PathBuf;

// TODO: consider using https://github.com/Canop/clap-help
#[derive(Parser, Debug)]
#[command(
    version, 
    author, 
    next_line_help = true,
    about="Extract specific lines from text files with powerful indexing",
    long_about = "A fast, flexible tool for extracting lines from text files using Python-like \
    indexing.\nSupports ranges, steps, and backward counting.",
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

    /// Process binary files as text
    #[arg(long, help_heading = "Input")]
    pub(crate) allow_binary_files: bool,

    // TODO: respect NO_COLOR env var, and update the doc below
    /// Specify when to use colored output. `auto` turns colors on when an interactive terminal is
    /// detected, and off when a pipe is detected. `always` turns colors on all the time, even if a
    /// pipe is detected.
    #[arg(long, value_enum, help_heading = "Output", default_value_t = When::Auto)]
    pub(crate) color: When,

    // TODO: respect PAGING and LINE_PAGING env vars, and update the doc below
    /// Specify when to use paging. `auto` uses paging when an interactive terminal is detected and
    /// the output is too long, and off when a pipe is detected. `always` uses paging all the time,
    /// even if a pipe is detected.
    #[arg(long, value_enum, help_heading = "Output", default_value_t = When::Auto)]
    pub(crate) paging: When,

    // /// Only show plain style, no decorations (e.g.: headers and line numbers).  When '-p' is used
    // /// twice, it also disables automatic paging. This option doesn't affect colors, you can use
    // /// `--color=never` to turn colored output off.
    // #[arg(short, long, help_heading = "Output", action = ArgAction::Count)]
    // pub(crate) plain: u8,

    /// Only show plain style, no decorations (e.g.: headers and line numbers). This option doesn't
    /// affect colors, you can use `--color=never` to turn colored output off.
    #[arg(short, long, help_heading = "Output", action = ArgAction::SetTrue)]
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

    // TODO: support stdin
    /// Input file (omit or use '-' for stdin)
    #[arg(value_name = "FILE")]
    pub(crate) file: PathBuf,
}

#[derive(ValueEnum, Clone, Debug)]
pub(crate) enum When {
    Auto,
    Always,
    Never,
}
