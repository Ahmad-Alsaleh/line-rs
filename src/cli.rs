use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about="Extract lines without hacks", author, long_about = None)]
pub(crate) struct Cli {
    /// line number to extract
    #[arg(short = 'n', long)]
    pub(crate) line: usize,

    pub(crate) file: String,
}
