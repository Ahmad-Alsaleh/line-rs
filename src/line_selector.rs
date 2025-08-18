use anyhow::Context;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum LineSelector {
    Single(Number),
    Range(Number, Number),
}

impl Ord for LineSelector {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (LineSelector::Single(a), LineSelector::Single(b)) => a.cmp(b),
            (LineSelector::Single(a), LineSelector::Range(b, _)) => a.cmp(b),
            (LineSelector::Range(a, _), LineSelector::Single(b)) => a.cmp(b),
            (LineSelector::Range(a, _), LineSelector::Range(b, _)) => a.cmp(b),
        }
    }
}

impl PartialOrd for LineSelector {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Number {
    Positive(usize),
    Negative(usize),
}

impl Ord for Number {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Number::Positive(a), Number::Positive(b)) => a.cmp(b),
            (Number::Positive(_), Number::Negative(_)) => std::cmp::Ordering::Greater,
            (Number::Negative(_), Number::Positive(_)) => std::cmp::Ordering::Less,
            (Number::Negative(a), Number::Negative(b)) => b.cmp(a),
        }
    }
}

impl PartialOrd for Number {
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
note: before inserting a new item, check if it's key is < vec.last to avoid duplicates
 */

// impl LineSelector {
// Converts negative line numbers to possitve and converts one-index to zero-index
// pub(crate) fn to_pisitive_zero_index(&mut self, n_lines: usize) {
//     let convert = |line_num: isize| {
//         if line_num < 0 {
//             n_lines - -line_num as usize
//         } else {
//             // subtracte one to convert to zero-index
//             line_num as usize - 1
//         }
//     };
//     if let LineSelector::Single(v) = self {}
//     match self {
//         LineSelector::Single(line_num) => {
//             *line_num = convert(*line_num);
//         }
//         LineSelector::Range(lower, upper) => {
//             *lower = convert(*lower);
//             *upper = convert(*upper);
//         }
//     };
// }
// }

/*
-5 -4 -3 -2 -1
 1  2  3  4  5



3:-1
-1+(5+1) =
*/

impl FromStr for LineSelector {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn parse_num(s: &str) -> anyhow::Result<Number> {
            let num: isize = s
                .parse()
                .with_context(|| format!("Value `{s}` is not a number"))?;

            if num == 0 {
                anyhow::bail!("Line number can't be zero");
            }

            let num = if num < 0 {
                Number::Negative(num.unsigned_abs())
            } else {
                Number::Positive(num as usize)
            };

            Ok(num)
        }

        match s.split_once(':') {
            Some((lower, upper)) => {
                let lower = parse_num(lower)?;
                let upper = parse_num(upper)?;
                if lower == upper {
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
