use anyhow::Context;
use std::fmt::{Debug, Display};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ParsedLineSelector {
    /// Single line number (zero-based)
    Single(usize),

    /// Stores the start, the end, and the step of a line selector.
    /// Note that start and end are zero-based and both ends are inclusive.
    ///
    /// # Examples
    ///
    /// `Range(1, 5, 2)` represents the lines 1, 3, and 5.
    /// `Range(8, 2, -3)` represents the lines 8, 5, and 2.
    Range(usize, usize, isize),
}

impl ParsedLineSelector {
    /// Parses `raw` as a zero-based line number, normalizing negative line numbers and
    /// unbounded ranges.
    ///
    /// `n_lines` is the number of lines in a file, it will be used to convert negative numbers
    /// and unbounded ranges and to check if the parsed line number is out of bound.
    ///
    /// # Notes:
    ///
    /// Ranges with steps will be internally stored in a tightened format. For example, `1:6:2`
    /// represents the numbers 1, 3, and 5. Thus, `1:6:2` will be represented as `Range(1, 5, 2)`
    /// instead of `Range(1, 6, 2)`.
    ///
    /// # Errors:
    ///
    /// This method returns an error if:
    /// 1. `raw` contains a zero (`raw` is one-based so it can't be zero)
    /// 2. `raw` contains a number that's beyond the limits of the file (i.e.: not between -n_lines and n_lines)
    /// 3. `raw` is a range and the start is larger than the end (e.g.: `5:3` or `3:5:-1`)
    pub(crate) fn from_raw(raw: RawLineSelector, n_lines: usize) -> anyhow::Result<Self> {
        let to_positive_one_based = |num: isize| {
            if num.unsigned_abs() > n_lines {
                anyhow::bail!("Line {num} is out of range (input has only {n_lines} line(s))");
            }

            let num = if num < 0 {
                n_lines - -num as usize
            } else {
                num as usize - 1
            };

            Ok(num)
        };
        match raw {
            RawLineSelector::Single(line_num) => {
                let line_num = to_positive_one_based(line_num)?;
                Ok(Self::Single(line_num))
            }
            RawLineSelector::Range(start, end) => {
                let start = start.map(to_positive_one_based).unwrap_or(Ok(0))?;
                let end = end.map(to_positive_one_based).unwrap_or(Ok(n_lines - 1))?;

                if start > end {
                    anyhow::bail!(
                        "The start of the range can't be more than its end when the step is positive"
                    );
                }

                if start == end {
                    Ok(Self::Single(start))
                } else {
                    Ok(Self::Range(start, end, 1))
                }
            }
            RawLineSelector::RangeWithStep(start, end, step) => {
                let start = start.map(to_positive_one_based).unwrap_or(Ok(0))?;
                let end = end.map(to_positive_one_based).unwrap_or(Ok(n_lines - 1))?;
                let step = step.unwrap_or(1);

                if step > 0 && start > end {
                    anyhow::bail!(
                        "The start of the range can't be more than its end when the step is positive"
                    );
                }
                if step < 0 && start < end {
                    anyhow::bail!(
                        "The start of the range can't be less than its end when the step is negative"
                    );
                }

                // TODO: benchmark whether using `end -/+ end.abs_diff(start) % abs_step` is
                // more efficient than `start +/- end.abs_diff(start) / abs_step * abs_step`
                match start.cmp(&end) {
                    std::cmp::Ordering::Equal => Ok(Self::Single(start)),
                    std::cmp::Ordering::Less => {
                        // tighten the end bound. eg: 0:5:2 becomes 0:4:2
                        let abs_step = step.unsigned_abs();
                        let end = start + end.abs_diff(start) / abs_step * abs_step;
                        Ok(Self::Range(start, end, step))
                    }
                    std::cmp::Ordering::Greater => {
                        // tighten the end bound. eg: 5:0:-2 becomes 5:1:-2
                        let abs_step = step.unsigned_abs();
                        let end = start - end.abs_diff(start) / abs_step * abs_step;
                        Ok(Self::Range(start, end, step))
                    }
                }
            }
        }
    }
}

impl Ord for ParsedLineSelector {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let a = match self {
            ParsedLineSelector::Single(line_num) => line_num,
            ParsedLineSelector::Range(start, end, _) => start.min(end),
        };
        let b = match other {
            ParsedLineSelector::Single(line_num) => line_num,
            ParsedLineSelector::Range(start, end, _) => start.min(end),
        };
        a.cmp(b)
    }
}

impl PartialOrd for ParsedLineSelector {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Represents a line selector as parsed from user input, before validation
///
/// # Examples:
///
/// `-4` is represented as Single(-4)
/// `:5` is represented as Range(None, Some(5))
/// `3:7:2` is represented as RangeWithStep(Some(3), Some(7), Some(2))
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RawLineSelector {
    /// Single line number (1-based)
    Single(isize),

    /// Range with optional bounds (1-based, inclusive)
    Range(Option<isize>, Option<isize>),

    /// Range with step (1-based, inclusive)
    RangeWithStep(Option<isize>, Option<isize>, Option<isize>),
}

impl RawLineSelector {
    /// Parses `s` into single and range line selectors without validation (e.g. if the number is
    /// out of bound) or further processing (e.g. converting negative numbers and unbounded ranges).
    /// Thus, the numbers are stored as one-based.
    ///
    /// Errors:
    ///
    /// This method returns an error if: `s` can't be parsed into a number
    pub(crate) fn from_str(s: &str) -> anyhow::Result<Self> {
        let s = s.trim();
        if s.is_empty() {
            anyhow::bail!("Line number can't be empty");
        }

        let parse = |s: &str| {
            if s.is_empty() {
                return Ok(None);
            }
            let num: isize = s
                .parse()
                .with_context(|| format!("Value `{s}` is not a number"))?;
            if num == 0 {
                anyhow::bail!(
                    "Zero is not allowed. Use positive numbers (1, 2, ...) or negative numbers \
                    (-1, -2, ...) for backward counting"
                );
            }
            Ok(Some(num))
        };

        let mut parts = s.splitn(3, ':');
        match (parts.next(), parts.next(), parts.next()) {
            (Some(line_num), None, None) => {
                let line_num = parse(line_num)?.expect("We already checked that `s` is not empty");
                Ok(Self::Single(line_num))
            }
            (Some(start), Some(end), None) => {
                let start = parse(start)?;
                let end = parse(end)?;
                Ok(Self::Range(start, end))
            }
            (Some(start), Some(end), Some(step)) => {
                let start = parse(start)?;
                let end = parse(end)?;
                let step = parse(step)?;
                Ok(Self::RangeWithStep(start, end, step))
            }
            _ => unreachable!(),
        }
    }
}

impl Display for RawLineSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RawLineSelector::Single(line_num) => write!(f, "{line_num}"),
            RawLineSelector::Range(start, end) => match (start, end) {
                (None, None) => write!(f, ":"),
                (None, Some(end)) => write!(f, ":{end}"),
                (Some(start), None) => write!(f, "{start}:"),
                (Some(start), Some(end)) => write!(f, "{start}:{end}"),
            },
            RawLineSelector::RangeWithStep(start, end, step) => match (start, end, step) {
                (None, None, None) => write!(f, "::"),
                (None, None, Some(step)) => write!(f, "::{step}"),
                (None, Some(end), None) => write!(f, ":{end}:"),
                (None, Some(end), Some(step)) => write!(f, ":{end}:{step}"),
                (Some(start), None, None) => write!(f, "{start}::"),
                (Some(start), None, Some(step)) => write!(f, "{start}::{step}"),
                (Some(start), Some(end), None) => write!(f, "{start}:{end}:"),
                (Some(start), Some(end), Some(step)) => write!(f, "{start}:{end}:{step}"),
            },
        }
    }
}

// TODO: test the step feature of Range
// and test all possible combinations of
// start:end:step
// '1'
//
// ':'
// ':2'
// '1:'
// '1:2'
//
// '::'
// '::3'
// ':2:'
// ':2:3'
// '1::'
// '1::3'
// '1:2:'
// '1:2:3'

#[cfg(test)]
mod tests {
    use super::*;

    mod create_parsed_line_selector {
        use super::*;

        macro_rules! create_parsed_line_selector {
            ($s: literal, $n_lines: literal) => {{
                let raw = RawLineSelector::from_str($s).unwrap();
                ParsedLineSelector::from_raw(raw, $n_lines)
            }};
        }

        #[test]
        fn single_number() {
            assert_eq!(
                create_parsed_line_selector!("2", 2).unwrap(),
                ParsedLineSelector::Single(1)
            );
            assert_eq!(
                create_parsed_line_selector!("-2", 2).unwrap(),
                ParsedLineSelector::Single(0)
            );
        }

        #[test]
        fn line_number_is_zero() {
            // TODO: replace all `.is_err` with `matches!(CORRECT_ERROR_TYPE)`
            // once custom errors are created
            assert!(RawLineSelector::from_str("0").is_err());
        }

        #[test]
        fn start_equals_end() {
            assert_eq!(
                create_parsed_line_selector!("2:2", 2).unwrap(),
                ParsedLineSelector::Single(1)
            );
            assert_eq!(
                create_parsed_line_selector!("2:-4", 5).unwrap(),
                ParsedLineSelector::Single(1)
            );
        }

        #[test]
        fn out_of_bounds() {
            assert!(create_parsed_line_selector!("-3", 2).is_err());
            assert!(create_parsed_line_selector!("3", 2).is_err());
        }

        #[test]
        fn start_more_than_end() {
            assert!(create_parsed_line_selector!("3:2", 42).is_err());
        }

        #[test]
        fn bounded_range() {
            assert_eq!(
                create_parsed_line_selector!("-5:2", 5).unwrap(),
                ParsedLineSelector::Range(0, 1, 1)
            );
            assert_eq!(
                create_parsed_line_selector!("2:-1", 5).unwrap(),
                ParsedLineSelector::Range(1, 4, 1)
            );
            assert_eq!(
                create_parsed_line_selector!("2:5", 5).unwrap(),
                ParsedLineSelector::Range(1, 4, 1)
            );
            assert_eq!(
                create_parsed_line_selector!("-5:-1", 5).unwrap(),
                ParsedLineSelector::Range(0, 4, 1)
            );
        }

        #[test]
        fn unbounded_range() {
            assert_eq!(
                create_parsed_line_selector!("1:", 5).unwrap(),
                ParsedLineSelector::Range(0, 4, 1)
            );
            assert_eq!(
                create_parsed_line_selector!(":5", 5).unwrap(),
                ParsedLineSelector::Range(0, 4, 1)
            );
            assert_eq!(
                create_parsed_line_selector!(":", 5).unwrap(),
                ParsedLineSelector::Range(0, 4, 1)
            );
        }

        #[test]
        fn with_surrounding_whitespace() {
            assert_eq!(
                create_parsed_line_selector!("   1:5 ", 5).unwrap(),
                ParsedLineSelector::Range(0, 4, 1)
            );
            assert!(RawLineSelector::from_str("1: 5").is_err());
            assert!(RawLineSelector::from_str("1 :5").is_err());
            assert!(RawLineSelector::from_str("1 : 5").is_err());
        }

        #[test]
        fn not_parsable() {
            assert!(RawLineSelector::from_str("a").is_err());
            assert!(RawLineSelector::from_str("a:2").is_err());
            assert!(RawLineSelector::from_str("1:a").is_err());
            assert!(RawLineSelector::from_str("a:2:3").is_err());
            assert!(RawLineSelector::from_str("1:a:3").is_err());
            assert!(RawLineSelector::from_str("1:2:a").is_err());
        }
    }

    mod display_raw_line_selector {
        use super::*;

        #[test]
        fn single() {
            let line_selector = RawLineSelector::from_str("1").unwrap();
            assert_eq!(line_selector.to_string(), "1");
        }

        #[test]
        fn range() {
            for s in [":", ":2", "1:", "1:2"] {
                let line_selector = RawLineSelector::from_str(s).unwrap();
                assert_eq!(line_selector.to_string(), s);
            }
        }

        #[test]
        fn range_with_step() {
            for s in ["::", "::3", ":2:", ":2:3", "1::", "1::3", "1:2:", "1:2:3"] {
                let line_selector = RawLineSelector::from_str(s).unwrap();
                assert_eq!(line_selector.to_string(), s);
            }
        }
    }
}
