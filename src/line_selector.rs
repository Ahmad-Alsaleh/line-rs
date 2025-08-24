use anyhow::Context;
use core::panic;
use std::fmt::{Debug, Display};

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct LineSelector {
    pub(crate) parsed: ParsedLineSelector,
    pub(crate) original: OriginalLineSelector, // used for pretty printing
}

impl LineSelector {
    pub(crate) fn from_original(
        original: OriginalLineSelector,
        n_lines: usize,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            parsed: ParsedLineSelector::from_original(original, n_lines)?,
            original,
        })
    }
}

impl Ord for LineSelector {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.parsed.cmp(&other.parsed)
    }
}

impl PartialOrd for LineSelector {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ParsedLineSelector {
    Single(usize),
    Range(usize, usize),
}

impl ParsedLineSelector {
    /// Parses `original` as a zero-based line number, converting negative line numbers and
    /// unbounded ranges.
    ///
    /// `n_lines` is the number of lines in a file, it will be used to convert negative numbers
    /// and unbounded ranges and to check if the parsed line number is out of bound.
    ///
    /// Errors:
    ///
    /// This method returns an error if:
    /// 1. `original` contains a zero (`original` is one-based and can't contain zero)
    /// 2. `original` contains a number that's beyond the limits of the file (i.e.: not between -n_lines and n_lines)
    /// 3. `original` is a range and the lower bound is larger than the upper bound (e.g.: `5:3`)
    fn from_original(original: OriginalLineSelector, n_lines: usize) -> anyhow::Result<Self> {
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
                    anyhow::bail!("Lower bound can't be more than upper bound")
                }

                if lower == upper {
                    Ok(Self::Single(lower))
                } else {
                    Ok(Self::Range(lower, upper))
                }
            }
        }
    }

    pub(crate) fn len(&self) -> usize {
        match self {
            ParsedLineSelector::Single(_) => 1,
            ParsedLineSelector::Range(lower, upper) => upper - lower + 1,
        }
    }

    pub(crate) fn lower(&self) -> usize {
        match self {
            ParsedLineSelector::Single(lower) => *lower,
            ParsedLineSelector::Range(lower, _) => *lower,
        }
    }

    pub(crate) fn upper(&self) -> usize {
        match self {
            ParsedLineSelector::Single(upper) => *upper,
            ParsedLineSelector::Range(_, upper) => *upper,
        }
    }

    pub(crate) fn overlap_len(&self, other: &Self) -> usize {
        let upper = usize::min(self.upper(), other.upper());
        let lower = usize::max(self.lower(), other.lower());
        (upper + 1).saturating_sub(lower)
    }
}

impl Ord for ParsedLineSelector {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (ParsedLineSelector::Single(a), ParsedLineSelector::Single(b)) => a.cmp(b),
            (ParsedLineSelector::Single(a), ParsedLineSelector::Range(b, _)) => a.cmp(b),
            (ParsedLineSelector::Range(a, _), ParsedLineSelector::Single(b)) => a.cmp(b),
            (ParsedLineSelector::Range(a, _), ParsedLineSelector::Range(b, _)) => a.cmp(b),
        }
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
        let parse = |s: &str| {
            let num: isize = s
                .parse()
                .with_context(|| format!("Value `{s}` is not a number"))?;

            Ok::<_, anyhow::Error>(num)
        };

        let s = s.trim();

        match s.split_once(':') {
            Some((lower, upper)) => {
                let lower = if lower.is_empty() {
                    None
                } else {
                    Some(parse(lower)?)
                };

                let upper = if upper.is_empty() {
                    None
                } else {
                    Some(parse(upper)?)
                };

                Ok(Self::Range(lower, upper))
            }
            None => {
                let num = parse(s)?;
                Ok(Self::Single(num))
            }
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
        }
    }
}

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
        fn range() {
            assert_eq!(
                create_parsed_line_selector!("-5:2", 5).unwrap(),
                ParsedLineSelector::Range(0, 1)
            );
            assert_eq!(
                create_parsed_line_selector!("2:-1", 5).unwrap(),
                ParsedLineSelector::Range(1, 4)
            );
            assert_eq!(
                create_parsed_line_selector!("2:5", 5).unwrap(),
                ParsedLineSelector::Range(1, 4)
            );
            assert_eq!(
                create_parsed_line_selector!("-5:-1", 5).unwrap(),
                ParsedLineSelector::Range(0, 4)
            );
        }

        #[test]
        fn not_parsable() {
            assert!(OriginalLineSelector::from_str("a").is_err());
        }
    }

    mod len {
        use super::*;

        #[test]
        fn single() {
            assert_eq!(ParsedLineSelector::Single(7).len(), 1);
        }

        #[test]
        fn range() {
            assert_eq!(ParsedLineSelector::Range(3, 7).len(), 5);
        }
    }

    mod lower {
        use super::*;

        #[test]
        fn single() {
            assert_eq!(ParsedLineSelector::Single(7).lower(), 7);
        }

        #[test]
        fn range() {
            assert_eq!(ParsedLineSelector::Range(3, 7).lower(), 3);
        }
    }

    mod upper {
        use super::*;

        #[test]
        fn single() {
            assert_eq!(ParsedLineSelector::Single(7).upper(), 7);
        }

        #[test]
        fn range() {
            assert_eq!(ParsedLineSelector::Range(3, 7).upper(), 7);
        }
    }

    mod overlap_len {
        use super::*;

        #[test]
        fn b_lower_is_a_lower() {
            let a = create_parsed_line_selector!("2:7", 42).unwrap();
            let b = create_parsed_line_selector!("2", 42).unwrap();
            assert_eq!(a.overlap_len(&b), 1);

            let a = create_parsed_line_selector!("2:7", 42).unwrap();
            let b = create_parsed_line_selector!("2:5", 42).unwrap();
            assert_eq!(a.overlap_len(&b), 4);

            let a = create_parsed_line_selector!("2:7", 42).unwrap();
            let b = create_parsed_line_selector!("2:7", 42).unwrap();
            assert_eq!(a.overlap_len(&b), 6);

            let a = create_parsed_line_selector!("2:7", 42).unwrap();
            let b = create_parsed_line_selector!("2:9", 42).unwrap();
            assert_eq!(a.overlap_len(&b), 6);

            let a = create_parsed_line_selector!("3", 42).unwrap();
            let b = create_parsed_line_selector!("3", 42).unwrap();
            assert_eq!(a.overlap_len(&b), 1);

            let a = create_parsed_line_selector!("3", 42).unwrap();
            let b = create_parsed_line_selector!("3:5", 42).unwrap();
            assert_eq!(a.overlap_len(&b), 1);
        }

        #[test]
        fn b_lower_is_inside_a() {
            let a = create_parsed_line_selector!("2:7", 42).unwrap();
            let b = create_parsed_line_selector!("4", 42).unwrap();
            assert_eq!(a.overlap_len(&b), 1);

            let a = create_parsed_line_selector!("2:7", 42).unwrap();
            let b = create_parsed_line_selector!("4:6", 42).unwrap();
            assert_eq!(a.overlap_len(&b), 3);

            let a = create_parsed_line_selector!("2:7", 42).unwrap();
            let b = create_parsed_line_selector!("4:7", 42).unwrap();
            assert_eq!(a.overlap_len(&b), 4);

            let a = create_parsed_line_selector!("2:7", 42).unwrap();
            let b = create_parsed_line_selector!("4:9", 42).unwrap();
            assert_eq!(a.overlap_len(&b), 4);
        }

        #[test]
        fn b_lower_is_a_upper() {
            let a = create_parsed_line_selector!("2:6", 42).unwrap();
            let b = create_parsed_line_selector!("6", 42).unwrap();
            assert_eq!(a.overlap_len(&b), 1);

            let a = create_parsed_line_selector!("2:6", 42).unwrap();
            let b = create_parsed_line_selector!("6:8", 42).unwrap();
            assert_eq!(a.overlap_len(&b), 1);
        }

        #[test]
        fn b_lower_is_outside_a() {
            let a = create_parsed_line_selector!("2:6", 42).unwrap();
            let b = create_parsed_line_selector!("7", 42).unwrap();
            assert_eq!(a.overlap_len(&b), 0);

            let a = create_parsed_line_selector!("2:6", 42).unwrap();
            let b = create_parsed_line_selector!("7:9", 42).unwrap();
            assert_eq!(a.overlap_len(&b), 0);

            let a = create_parsed_line_selector!("3", 42).unwrap();
            let b = create_parsed_line_selector!("5", 42).unwrap();
            assert_eq!(a.overlap_len(&b), 0);

            let a = create_parsed_line_selector!("3", 42).unwrap();
            let b = create_parsed_line_selector!("5:7", 42).unwrap();
            assert_eq!(a.overlap_len(&b), 0);
        }
    }
}
