use anyhow::Context;
use std::fmt::{Debug, Display};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ParsedLineSelector {
    /// Stores a single line selector.
    /// Note that the line number is zero-based.
    Single(usize),

    /// Stores the lower bound, the upper bound, and the step of a line selector.
    /// Note that lower and upper bounds are zero-based and both ends are inclusive.
    ///
    /// Examples
    ///
    /// `Range(1, 5, 2)` represents the lines 1, 3, and 5.
    /// `Range(8, 2, -3)` represents the lines 8, 5, and 2.
    Range(usize, usize, isize),
}

impl ParsedLineSelector {
    /// Parses `original` as a zero-based line number, converting negative line numbers and
    /// unbounded ranges.
    ///
    /// `n_lines` is the number of lines in a file, it will be used to convert negative numbers
    /// and unbounded ranges and to check if the parsed line number is out of bound.
    ///
    /// Notes:
    ///
    /// Ranges with steps will be internally stored in a tightened format. For example, `1:6:2`
    /// represents the numbers 1, 3, and 5. Thus, `1:6:2` will be represented as `Range(1, 5, 2)`
    /// instead of `Range(1, 6, 2)`.
    ///
    /// Errors:
    ///
    /// This method returns an error if:
    /// 1. `original` contains a zero (`original` is one-based and can't contain zero)
    /// 2. `original` contains a number that's beyond the limits of the file (i.e.: not between -n_lines and n_lines)
    /// 3. `original` is a range and the lower bound is larger than the upper bound (e.g.: `5:3`)
    pub(crate) fn from_original(
        original: OriginalLineSelector,
        n_lines: usize,
    ) -> anyhow::Result<Self> {
        let to_positive_one_based = |num: isize| {
            if num == 0 {
                anyhow::bail!("Line number can't be zero");
            }

            if num.unsigned_abs() > n_lines {
                anyhow::bail!("Line {num} is out of bound, input has {n_lines} line(s) only",);
            }

            let num = if num < 0 {
                n_lines - -num as usize
            } else {
                num as usize - 1
            };

            Ok(num)
        };
        match original {
            OriginalLineSelector::Single(line_num) => {
                let line_num = to_positive_one_based(line_num)?;
                Ok(Self::Single(line_num))
            }
            OriginalLineSelector::Range(lower, upper) => {
                let lower = lower.map(to_positive_one_based).unwrap_or(Ok(0))?;
                let upper = upper
                    .map(to_positive_one_based)
                    .unwrap_or(Ok(n_lines - 1))?;

                if lower > upper {
                    anyhow::bail!(
                        "The start of the range can't be more than it's end when the step is positive"
                    );
                }

                if lower == upper {
                    Ok(Self::Single(lower))
                } else {
                    Ok(Self::Range(lower, upper, 1))
                }
            }
            OriginalLineSelector::RangeWithStep(lower, upper, step) => {
                if step == Some(0) {
                    anyhow::bail!("Step can't be zero");
                }

                let lower = lower.map(to_positive_one_based).unwrap_or(Ok(0))?;
                let upper = upper
                    .map(to_positive_one_based)
                    .unwrap_or(Ok(n_lines - 1))?;
                let step = step.unwrap_or(1);

                if step > 0 && lower > upper {
                    anyhow::bail!(
                        "The start of the range can't be more than it's end when the step is positive"
                    );
                }
                if step < 0 && lower < upper {
                    anyhow::bail!(
                        "The start of the range can't be less than it's end when the step is negative"
                    );
                }

                let abs_step = step.unsigned_abs();

                // TODO: benchmark whether using `upper -/+ upper.abs_diff(lower) % abs_step` is
                // more effecient than `lower +/- upper.abs_diff(lower) / abs_step * abs_step`
                match lower.cmp(&upper) {
                    std::cmp::Ordering::Equal => Ok(Self::Single(lower)),
                    std::cmp::Ordering::Less => {
                        // tighten the upper bound. eg: 0:5:2 becomes 0:4:2
                        let upper = lower + upper.abs_diff(lower) / abs_step * abs_step;
                        Ok(Self::Range(lower, upper, step))
                    }
                    std::cmp::Ordering::Greater => {
                        // tighten the upper bound. eg: 5:0:-2 becomes 5:1:-2
                        let upper = lower - upper.abs_diff(lower) / abs_step * abs_step;
                        // TODO: rename every lower/upper to start/end since lower can be greater
                        // than upper when step is negative, which can confusing :(
                        Ok(Self::Range(lower, upper, step))
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
            ParsedLineSelector::Range(lower, upper, _) => lower.min(upper),
        };
        let b = match other {
            ParsedLineSelector::Single(line_num) => line_num,
            ParsedLineSelector::Range(lower, upper, _) => lower.min(upper),
        };
        a.cmp(b)
    }
}

impl PartialOrd for ParsedLineSelector {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OriginalLineSelector {
    Single(isize),
    Range(Option<isize>, Option<isize>),
    RangeWithStep(Option<isize>, Option<isize>, Option<isize>),
}

impl OriginalLineSelector {
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
            Ok::<_, anyhow::Error>(Some(num))
        };

        let mut parts = s.splitn(3, ':');
        match (parts.next(), parts.next(), parts.next()) {
            (Some(line_num), None, None) => {
                let line_num = parse(line_num)?.expect("We already checked that `s` is not empty");
                Ok(Self::Single(line_num))
            }
            (Some(lower), Some(upper), None) => {
                let lower = parse(lower)?;
                let upper = parse(upper)?;
                Ok(Self::Range(lower, upper))
            }
            (Some(lower), Some(upper), Some(step)) => {
                let lower = parse(lower)?;
                let upper = parse(upper)?;
                let step = parse(step)?;
                Ok(Self::RangeWithStep(lower, upper, step))
            }
            _ => unreachable!(),
        }
    }
}

impl Display for OriginalLineSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OriginalLineSelector::Single(line_num) => write!(f, "{line_num}"),
            OriginalLineSelector::Range(lower, upper) => match (lower, upper) {
                (None, None) => write!(f, ":"),
                (None, Some(upper)) => write!(f, ":{upper}"),
                (Some(lower), None) => write!(f, "{lower}:"),
                (Some(lower), Some(upper)) => write!(f, "{lower}:{upper}"),
            },
            OriginalLineSelector::RangeWithStep(lower, upper, step) => match (lower, upper, step) {
                (None, None, None) => write!(f, "::"),
                (None, None, Some(step)) => write!(f, "::{step}"),
                (None, Some(upper), None) => write!(f, ":{upper}:"),
                (None, Some(upper), Some(step)) => write!(f, ":{upper}:{step}"),
                (Some(lower), None, None) => write!(f, "{lower}::"),
                (Some(lower), None, Some(step)) => write!(f, "{lower}::{step}"),
                (Some(lower), Some(upper), None) => write!(f, "{lower}:{upper}:"),
                (Some(lower), Some(upper), Some(step)) => write!(f, "{lower}:{upper}:{step}"),
            },
        }
    }
}

// TODO: test the step feature of Range
// and test all possible combinations of
// lower:upper:step
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

    macro_rules! create_parsed_line_selector {
        ($s: literal, $n_lines: literal) => {{
            let original = OriginalLineSelector::from_str($s).unwrap();
            ParsedLineSelector::from_original(original, $n_lines)
        }};
    }

    mod create_parsed_line_selector {
        use super::*;

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
            assert!(create_parsed_line_selector!("0", 42).is_err());
        }

        #[test]
        fn lower_bound_equals_upper_bound() {
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
        fn lower_bound_more_than_upper_bound() {
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
        fn with_srounding_whitespace() {
            assert_eq!(
                create_parsed_line_selector!("   1:5 ", 5).unwrap(),
                ParsedLineSelector::Range(0, 4, 1)
            );
            assert!(OriginalLineSelector::from_str("1: 5").is_err());
            assert!(OriginalLineSelector::from_str("1 :5").is_err());
            assert!(OriginalLineSelector::from_str("1 : 5").is_err());
        }

        #[test]
        fn not_parsable() {
            assert!(OriginalLineSelector::from_str("a").is_err());
        }
    }
}
