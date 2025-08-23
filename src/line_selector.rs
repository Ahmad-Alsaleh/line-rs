use anyhow::Context;

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct LineSelector<'a> {
    pub(crate) original: &'a str, // used for pretty printing
    pub(crate) parsed: ParsedLineSelector,
}

impl<'a> LineSelector<'a> {
    pub(crate) fn from_str(s: &'a str, n_lines: usize) -> anyhow::Result<Self> {
        Ok(Self {
            original: s,
            parsed: ParsedLineSelector::from_str(s, n_lines)?,
        })
    }
}

impl Ord for LineSelector<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.parsed.cmp(&other.parsed)
    }
}

impl PartialOrd for LineSelector<'_> {
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
    /// Parses `s` as a zero-based line number and converts negative line numbers to positive
    /// `n_lines` is the number of lines in a file, it will be used to convert negative numbers
    /// to positive and to check if the parsed line number is not out of bound.
    ///
    /// Errors:
    ///
    /// This method returns an error if:
    /// 1. `s` can't be parsed into a number
    /// 2. The parsed number is zero (line numbers are one-based and can't be zero)
    /// 3. The parsed number is not between -n_lines and n_lines
    pub(crate) fn from_str(s: &str, n_lines: usize) -> anyhow::Result<Self> {
        let to_positive_one_based = |s: &str| {
            let num: isize = s
                .parse()
                .with_context(|| format!("Value `{s}` is not a number"))?;

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

        match s.trim().split_once(':') {
            Some((lower, upper)) => {
                // TODO: handel unbounded ranges: `1:`, `:1`, and `:`
                let lower = to_positive_one_based(lower.trim())?;
                let upper = to_positive_one_based(upper.trim())?;
                if lower > upper {
                    anyhow::bail!("Lower bound can't be more than upper bound")
                }
                if lower == upper {
                    Ok(Self::Single(lower))
                } else {
                    Ok(Self::Range(lower, upper))
                }
            }
            None => {
                let num = to_positive_one_based(s.trim())?;
                Ok(Self::Single(num))
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

/*
input: 1,5,3
sort-input: 1,2,5
read and cache: [(1, ...), (2, ...), (5, ...)]
binrary-search and print

input: 1,5,1
sort-input: 1,1,5
read and cache: [(1, ...), (5, ...)]

input: 4,7,2,1:5
sort-input: 1:5,2,4,7
read and cache: [(1, ), (2, ), (3, ), (4, ), (5, ), (7, )]
note: before inserting a new item, ensure it's key is < vec.last, to avoid duplicates
 */

#[cfg(test)]
mod tests {
    use super::*;

    mod from_str {
        use super::*;

        #[test]
        fn single_number() -> anyhow::Result<()> {
            assert_eq!(
                ParsedLineSelector::from_str("2", 2)?,
                ParsedLineSelector::Single(1)
            );
            assert_eq!(
                ParsedLineSelector::from_str("-2", 2)?,
                ParsedLineSelector::Single(0)
            );
            Ok(())
        }

        #[test]
        fn line_number_is_zero() {
            // TODO: replace all `.is_err` with `matches!(CORRECT_ERROR_TYPE)`
            // once custom errors are created
            assert!(ParsedLineSelector::from_str("0", 42).is_err());
        }

        #[test]
        fn not_parsable() {
            assert!(ParsedLineSelector::from_str("a", 42).is_err());
        }

        #[test]
        fn lower_bound_equals_upper_bound() -> anyhow::Result<()> {
            assert_eq!(
                ParsedLineSelector::from_str("2:2", 2)?,
                ParsedLineSelector::Single(1)
            );
            assert_eq!(
                ParsedLineSelector::from_str("2:-4", 5)?,
                ParsedLineSelector::Single(1)
            );
            Ok(())
        }

        #[test]
        fn out_of_bounds() {
            assert!(ParsedLineSelector::from_str("-3", 2).is_err());
            assert!(ParsedLineSelector::from_str("3", 2).is_err());
        }

        #[test]
        fn lower_bound_more_than_upper_bound() {
            assert!(ParsedLineSelector::from_str("3:2", 42).is_err());
        }

        #[test]
        fn range() -> anyhow::Result<()> {
            assert_eq!(
                ParsedLineSelector::from_str("-5:2", 5)?,
                ParsedLineSelector::Range(0, 1)
            );
            assert_eq!(
                ParsedLineSelector::from_str("2:-1", 5)?,
                ParsedLineSelector::Range(1, 4)
            );
            assert_eq!(
                ParsedLineSelector::from_str("2:5", 5)?,
                ParsedLineSelector::Range(1, 4)
            );
            assert_eq!(
                ParsedLineSelector::from_str("-5:-1", 5)?,
                ParsedLineSelector::Range(0, 4)
            );
            Ok(())
        }
    }

    mod overlap_len {
        use super::*;

        #[test]
        fn b_lower_is_a_lower() {
            let a = ParsedLineSelector::from_str("2:7", 42).unwrap();
            let b = ParsedLineSelector::from_str("2", 42).unwrap();
            assert_eq!(a.overlap_len(&b), 1);

            let a = ParsedLineSelector::from_str("2:7", 42).unwrap();
            let b = ParsedLineSelector::from_str("2:5", 42).unwrap();
            assert_eq!(a.overlap_len(&b), 4);

            let a = ParsedLineSelector::from_str("2:7", 42).unwrap();
            let b = ParsedLineSelector::from_str("2:7", 42).unwrap();
            assert_eq!(a.overlap_len(&b), 6);

            let a = ParsedLineSelector::from_str("2:7", 42).unwrap();
            let b = ParsedLineSelector::from_str("2:9", 42).unwrap();
            assert_eq!(a.overlap_len(&b), 6);

            let a = ParsedLineSelector::from_str("3", 42).unwrap();
            let b = ParsedLineSelector::from_str("3", 42).unwrap();
            assert_eq!(a.overlap_len(&b), 1);

            let a = ParsedLineSelector::from_str("3", 42).unwrap();
            let b = ParsedLineSelector::from_str("3:5", 42).unwrap();
            assert_eq!(a.overlap_len(&b), 1);
        }

        #[test]
        fn b_lower_is_inside_a() {
            let a = ParsedLineSelector::from_str("2:7", 42).unwrap();
            let b = ParsedLineSelector::from_str("4", 42).unwrap();
            assert_eq!(a.overlap_len(&b), 1);

            let a = ParsedLineSelector::from_str("2:7", 42).unwrap();
            let b = ParsedLineSelector::from_str("4:6", 42).unwrap();
            assert_eq!(a.overlap_len(&b), 3);

            let a = ParsedLineSelector::from_str("2:7", 42).unwrap();
            let b = ParsedLineSelector::from_str("4:7", 42).unwrap();
            assert_eq!(a.overlap_len(&b), 4);

            let a = ParsedLineSelector::from_str("2:7", 42).unwrap();
            let b = ParsedLineSelector::from_str("4:9", 42).unwrap();
            assert_eq!(a.overlap_len(&b), 4);
        }

        #[test]
        fn b_lower_is_a_upper() {
            let a = ParsedLineSelector::from_str("2:6", 42).unwrap();
            let b = ParsedLineSelector::from_str("6", 42).unwrap();
            assert_eq!(a.overlap_len(&b), 1);

            let a = ParsedLineSelector::from_str("2:6", 42).unwrap();
            let b = ParsedLineSelector::from_str("6:8", 42).unwrap();
            assert_eq!(a.overlap_len(&b), 1);
        }

        #[test]
        fn b_lower_is_outside_a() {
            let a = ParsedLineSelector::from_str("2:6", 42).unwrap();
            let b = ParsedLineSelector::from_str("7", 42).unwrap();
            assert_eq!(a.overlap_len(&b), 0);

            let a = ParsedLineSelector::from_str("2:6", 42).unwrap();
            let b = ParsedLineSelector::from_str("7:9", 42).unwrap();
            assert_eq!(a.overlap_len(&b), 0);

            let a = ParsedLineSelector::from_str("3", 42).unwrap();
            let b = ParsedLineSelector::from_str("5", 42).unwrap();
            assert_eq!(a.overlap_len(&b), 0);

            let a = ParsedLineSelector::from_str("3", 42).unwrap();
            let b = ParsedLineSelector::from_str("5:7", 42).unwrap();
            assert_eq!(a.overlap_len(&b), 0);
        }
    }
}
