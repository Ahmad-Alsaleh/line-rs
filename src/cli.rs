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

#[cfg(test)]
mod tests {
    // use crate::line_selector::Number;
    //
    // use super::*;
    // use std::error::Error;
    //
    // #[test]
    // fn single_numer() {
    //     let args = Cli::parse_from([".", "--line", "1", "file"]);
    //     assert_eq!(*args.line_nums, [LineSelector::Single(Number::Positive(1))]);
    // }
    //
    // #[test]
    // fn multiple_numbers() {
    //     let args = Cli::parse_from([".", "--line", "1,2:3,4:4", "file"]);
    //     assert_eq!(
    //         *args.line_nums,
    //         [
    //             LineSelector::Single(Number::Positive(1)),
    //             LineSelector::Range(Number::Positive(2), Number::Positive(3)),
    //             LineSelector::Single(Number::Positive(4)),
    //         ]
    //     );
    // }
    //
    // #[test]
    // fn line_number_is_zero() {
    //     let err = Cli::try_parse_from([".", "--line", "0", "file"]).unwrap_err();
    //     // TODO: replace the below with a custom error. e.g.: LineSelectorError::ZeroLine
    //     assert_eq!(
    //         err.source().unwrap().to_string(),
    //         "Line number can't be zero"
    //     );
    // }
    //
    // #[test]
    // fn space_around_comma() {
    //     let args = Cli::parse_from([".", "--line", "1, 2,3 ,4 , 5", "file"]);
    //     assert_eq!(
    //         *args.line_nums,
    //         [
    //             LineSelector::Single(Number::Positive(1)),
    //             LineSelector::Single(Number::Positive(2)),
    //             LineSelector::Single(Number::Positive(3)),
    //             LineSelector::Single(Number::Positive(4)),
    //             LineSelector::Single(Number::Positive(5)),
    //         ]
    //     )
    // }
    //
    // #[test]
    // fn lower_bound_equals_upper_bound() {
    //     let args = Cli::parse_from([".", "--line", "4:4", "file"]);
    //     assert_eq!(*args.line_nums, [LineSelector::Single(Number::Positive(4))])
    // }
    //
    // #[test]
    // fn lower_bound_more_than_upper_bound() {
    //     let err = Cli::try_parse_from([".", "--line", "5:4", "file"]).unwrap_err();
    //     // TODO: replace the below with a custom error. e.g.: LineSelectorError::InvertedRange
    //     assert_eq!(
    //         err.source().unwrap().to_string(),
    //         "Lower bound can't be more than the upper bound: `5:4`"
    //     );
    // }
}
