use anyhow::Context;
use clap::Parser;
use std::{path::PathBuf, str::FromStr};

#[derive(Parser, Debug)]
#[command(version, about="Extract lines without hacks", author, long_about = None)]
pub(crate) struct Cli {
    /// Line number to extract
    #[arg(short = 'n', long = "line", value_parser = line_num_parser)]
    pub(crate) line_nums: Box<[LineSelector]>,
    /// File to extract line(s) from. Use a dash ('-') or no argument to read from standard input
    pub(crate) file: PathBuf,
    /// Treat binary files as text files
    #[arg(long)]
    pub(crate) allow_binary_files: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum LineSelector {
    Single(isize),
    Range(isize, isize),
}

impl FromStr for LineSelector {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parse_num = |s: &str| {
            let num = s
                .parse()
                .with_context(|| format!("Value `{s}` is not a number"))?;
            if num == 0 {
                anyhow::bail!("Line number can't be zero");
            }
            Ok(num)
        };

        match s.split_once(':') {
            Some((lower, upper)) => {
                let (lower, upper) = (parse_num(lower)?, parse_num(upper)?);
                if lower > upper {
                    anyhow::bail!("Lower bound can't be more than the upper bound: `{s}`");
                } else if lower == upper {
                    Ok(LineSelector::Single(lower))
                } else {
                    Ok(LineSelector::Range(lower, upper))
                }
            }
            None => {
                let num = parse_num(s)?;
                Ok(LineSelector::Single(num))
            }
        }
    }
}

fn line_num_parser(s: &str) -> anyhow::Result<Box<[LineSelector]>> {
    s.split(',').map(|part| part.trim().parse()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn single_numer() {
        let args = Cli::parse_from([".", "--line", "1", "file"]);
        assert_eq!(*args.line_nums, [LineSelector::Single(1)]);
    }

    #[test]
    fn multiple_numbers() {
        let args = Cli::parse_from([".", "--line", "1,2:3,4:4", "file"]);
        assert_eq!(
            *args.line_nums,
            [
                LineSelector::Single(1),
                LineSelector::Range(2, 3),
                LineSelector::Single(4)
            ]
        );
    }

    #[test]
    fn line_number_is_zero() {
        let err = Cli::try_parse_from([".", "--line", "0", "file"]).unwrap_err();
        // TODO: replace the below with a custom error. e.g.: LineSelectorError::ZeroLine
        assert_eq!(
            err.source().unwrap().to_string(),
            "Line number can't be zero"
        );
    }

    #[test]
    fn space_around_comma() {
        let args = Cli::parse_from([".", "--line", "1, 2,3 ,4 , 5", "file"]);
        assert_eq!(
            *args.line_nums,
            [
                LineSelector::Single(1),
                LineSelector::Single(2),
                LineSelector::Single(3),
                LineSelector::Single(4),
                LineSelector::Single(5),
            ]
        )
    }

    #[test]
    fn lower_bound_equals_upper_bound() {
        let args = Cli::parse_from([".", "--line", "4:4", "file"]);
        assert_eq!(*args.line_nums, [LineSelector::Single(4)])
    }

    #[test]
    fn lower_bound_more_than_upper_bound() {
        let err = Cli::try_parse_from([".", "--line", "5:4", "file"]).unwrap_err();
        // TODO: replace the below with a custom error. e.g.: LineSelectorError::InvertedRange
        assert_eq!(
            err.source().unwrap().to_string(),
            "Lower bound can't be more than the upper bound: `5:4`"
        );
    }
}
