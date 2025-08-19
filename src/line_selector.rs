use anyhow::Context;

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct LineSelector<'a> {
    pub(crate) original: &'a str, // used for pretty printing
    pub(crate) parsed: ParsedLineSelector,
}

impl<'a> LineSelector<'a> {
    pub(crate) fn new(s: &'a str, n_lines: usize) -> anyhow::Result<Self> {
        Ok(Self {
            original: s,
            parsed: ParsedLineSelector::new(s, n_lines)?,
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
    pub(crate) fn new(s: &str, n_lines: usize) -> anyhow::Result<Self> {
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
                let num = to_positive_one_based(s)?;
                Ok(Self::Single(num))
            }
        }
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

    #[test]
    fn single_number() -> anyhow::Result<()> {
        assert_eq!(
            ParsedLineSelector::new("2", 2)?,
            ParsedLineSelector::Single(1)
        );
        assert_eq!(
            ParsedLineSelector::new("-2", 2)?,
            ParsedLineSelector::Single(0)
        );
        Ok(())
    }

    #[test]
    fn line_number_is_zero() {
        // TODO: replace all `.is_err` with `matches!(CORRECT_ERROR_TYPE)`
        // once custom errors are created
        assert!(ParsedLineSelector::new("0", 42).is_err());
    }

    #[test]
    fn not_parsable() {
        assert!(ParsedLineSelector::new("a", 42).is_err());
    }

    #[test]
    fn lower_bound_equals_upper_bound() -> anyhow::Result<()> {
        assert_eq!(
            ParsedLineSelector::new("2:2", 2)?,
            ParsedLineSelector::Single(1)
        );
        assert_eq!(
            ParsedLineSelector::new("2:-4", 5)?,
            ParsedLineSelector::Single(1)
        );
        Ok(())
    }

    #[test]
    fn out_of_bounds() {
        assert!(ParsedLineSelector::new("-3", 2).is_err());
        assert!(ParsedLineSelector::new("3", 2).is_err());
    }

    #[test]
    fn lower_bound_more_than_upper_bound() {
        assert!(ParsedLineSelector::new("3:2", 42).is_err());
    }

    #[test]
    fn range() -> anyhow::Result<()> {
        assert_eq!(
            ParsedLineSelector::new("-5:2", 5)?,
            ParsedLineSelector::Range(0, 1)
        );
        assert_eq!(
            ParsedLineSelector::new("2:-1", 5)?,
            ParsedLineSelector::Range(1, 4)
        );
        assert_eq!(
            ParsedLineSelector::new("2:5", 5)?,
            ParsedLineSelector::Range(1, 4)
        );
        assert_eq!(
            ParsedLineSelector::new("-5:-1", 5)?,
            ParsedLineSelector::Range(0, 4)
        );
        Ok(())
    }
}
