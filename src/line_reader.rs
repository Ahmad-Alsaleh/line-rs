use std::io::BufRead;

/// Efficient line-by-line reader that can skip to specific line numbers.
///
/// This reader is optimized for scenarios where you need to read specific lines
/// from a file without loading the entire content into memory. It maintains
/// an internal line counter and can efficiently skip over unwanted lines.
///
/// # Undefined Behaviour
///
/// For efficiency reasons, lines should be read incrementally. That is, if you try to read lines 3
/// and 5. You should read line 3 first then 5. Otherwise, the behaviour will be undefined.
///
/// # Examples
///
/// ```rust,no_run
/// use std::io::BufReader;
/// use std::fs::File;
///
/// let file = File::open("file.txt").unwrap();
/// let mut reader = LineReader::new(BufReader::new(file));
///
/// let mut buffer = Vec::new();
/// reader.read_specific_line(&mut buffer, 42).unwrap(); // Read line 43 (zero-based indexing)
/// ```
pub(crate) struct LineReader<R> {
    reader: R,
    current_line: usize,
}

impl<R: BufRead> LineReader<R> {
    pub(crate) fn new(reader: R) -> Self {
        Self {
            reader,
            current_line: 0,
        }
    }

    fn read_next_line(&mut self, buf: &mut Vec<u8>) -> anyhow::Result<()> {
        let n = self.reader.read_until(b'\n', buf)?;
        if n != 0 {
            self.current_line += 1;
        }
        Ok(())
    }

    /// Skips `n` lines.
    fn skip_lines(&mut self, n: usize) -> anyhow::Result<()> {
        let mut i = 0;
        while i < n && self.reader.skip_until(b'\n')? > 0 {
            i += 1;
        }
        self.current_line += i;
        Ok(())
    }

    // TODO: double, check, is it > or >=
    /// `line_num` is zero-based.
    /// `lines_num` should be more than `self.current_line`.
    pub(crate) fn read_specific_line(
        &mut self,
        buf: &mut Vec<u8>,
        line_num: usize,
    ) -> anyhow::Result<()> {
        if line_num != self.current_line {
            self.skip_lines(line_num - self.current_line)?;
        }
        self.read_next_line(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::{
        fs::File,
        io::{BufReader, Cursor},
    };

    mod read_next_line {
        use super::*;

        #[test]
        fn input_with_trailing_new_line() {
            let cursor = Cursor::new("one\ntwo\n");
            let mut line_reader = LineReader::new(cursor);
            assert_eq!(line_reader.current_line, 0);

            let mut buf = Vec::new();

            line_reader.read_next_line(&mut buf).unwrap();
            assert_eq!(buf, b"one\n");
            assert_eq!(line_reader.current_line, 1);
            buf.clear();

            line_reader.read_next_line(&mut buf).unwrap();
            assert_eq!(buf, b"two\n");
            assert_eq!(line_reader.current_line, 2);
            buf.clear();

            line_reader.read_next_line(&mut buf).unwrap();
            assert_eq!(buf, b"");
            assert_eq!(line_reader.current_line, 2);
            buf.clear();
        }

        #[test]
        fn input_without_trailing_new_line() {
            let cursor = Cursor::new("one\ntwo");
            let mut line_reader = LineReader::new(cursor);
            assert_eq!(line_reader.current_line, 0);

            let mut buf = Vec::new();

            line_reader.read_next_line(&mut buf).unwrap();
            assert_eq!(buf, b"one\n");
            assert_eq!(line_reader.current_line, 1);
            buf.clear();

            line_reader.read_next_line(&mut buf).unwrap();
            assert_eq!(buf, b"two");
            assert_eq!(line_reader.current_line, 2);
            buf.clear();

            line_reader.read_next_line(&mut buf).unwrap();
            assert_eq!(buf, b"");
            assert_eq!(line_reader.current_line, 2);
            buf.clear();
        }

        #[test]
        fn empty_input() {
            let cursor = Cursor::new("");
            let mut line_reader = LineReader::new(cursor);
            assert_eq!(line_reader.current_line, 0);

            let mut buf = Vec::new();

            line_reader.read_next_line(&mut buf).unwrap();
            assert_eq!(buf, b"");
            assert_eq!(line_reader.current_line, 0);
        }

        #[test]
        fn input_is_new_line_only() {
            let cursor = Cursor::new("\n");
            let mut line_reader = LineReader::new(cursor);
            assert_eq!(line_reader.current_line, 0);

            let mut buf = Vec::new();

            line_reader.read_next_line(&mut buf).unwrap();
            assert_eq!(buf, b"\n");
            assert_eq!(line_reader.current_line, 1);
        }

        #[test]
        fn no_read_permissions() {
            let temp_dir = tempfile::tempdir().unwrap();
            let path = temp_dir.path().join("file.txt");

            // `File::create` creates a file with _write-only_ permissions
            let mut file = File::create(&path).unwrap();
            write!(file, "one\ntwo\n").unwrap();

            let mut line_reader = LineReader::new(BufReader::new(file));

            let mut buf = Vec::new();
            assert!(line_reader.read_next_line(&mut buf).is_err());
        }
    }

    mod skip_lines {
        use super::*;

        #[test]
        fn skip_zero_lines() {
            let cursor = Cursor::new("one\ntwo\n");
            let mut line_reader = LineReader::new(cursor);

            line_reader.skip_lines(0).unwrap();
            assert_eq!(line_reader.current_line, 0);

            let mut buf = Vec::new();
            line_reader.reader.read_to_end(&mut buf).unwrap();
            assert_eq!(buf, b"one\ntwo\n");
        }

        #[test]
        fn empty_input() {
            let cursor = Cursor::new("");
            let mut line_reader = LineReader::new(cursor);

            line_reader.skip_lines(0).unwrap();
            assert_eq!(line_reader.current_line, 0);

            line_reader.skip_lines(10).unwrap();
            assert_eq!(line_reader.current_line, 0);
        }

        #[test]
        fn input_is_new_line_only() {
            let cursor = Cursor::new("\n");
            let mut line_reader = LineReader::new(cursor);

            line_reader.skip_lines(1).unwrap();
            assert_eq!(line_reader.current_line, 1);

            let mut buf = Vec::new();
            line_reader.reader.read_to_end(&mut buf).unwrap();
            assert_eq!(buf, b"");
        }

        #[test]
        fn skip_line_in_range_with_trailing_ln() {
            let cursor = Cursor::new("one\ntwo\nthree\n");
            let mut line_reader = LineReader::new(cursor);

            line_reader.skip_lines(2).unwrap();
            assert_eq!(line_reader.current_line, 2);

            let mut buf = Vec::new();
            line_reader.reader.read_to_end(&mut buf).unwrap();
            assert_eq!(buf, b"three\n");
        }

        #[test]
        fn skip_last_line_with_trailing_ln() {
            let cursor = Cursor::new("one\ntwo\nthree\n");
            let mut line_reader = LineReader::new(cursor);

            line_reader.skip_lines(3).unwrap();
            assert_eq!(line_reader.current_line, 3);

            let mut buf = Vec::new();
            line_reader.reader.read_to_end(&mut buf).unwrap();
            assert_eq!(buf, b"");
        }

        #[test]
        fn skip_last_line_without_trailing_ln() {
            let cursor = Cursor::new("one\ntwo\nthree");
            let mut line_reader = LineReader::new(cursor);

            line_reader.skip_lines(3).unwrap();
            assert_eq!(line_reader.current_line, 3);

            let mut buf = Vec::new();
            line_reader.reader.read_to_end(&mut buf).unwrap();
            assert_eq!(buf, b"");
        }

        #[test]
        fn skip_line_out_of_range_with_trailing_ln() {
            let cursor = Cursor::new("one\ntwo\nthree\n");
            let mut line_reader = LineReader::new(cursor);

            line_reader.skip_lines(4).unwrap();
            assert_eq!(line_reader.current_line, 3);

            let mut buf = Vec::new();
            line_reader.reader.read_to_end(&mut buf).unwrap();
            assert_eq!(buf, b"");
        }

        #[test]
        fn skip_line_in_range_with_no_trailing_ln() {
            let cursor = Cursor::new("one\ntwo\nthree");
            let mut line_reader = LineReader::new(cursor);

            line_reader.skip_lines(2).unwrap();
            assert_eq!(line_reader.current_line, 2);

            let mut buf = Vec::new();
            line_reader.reader.read_to_end(&mut buf).unwrap();
            assert_eq!(buf, b"three");
        }

        #[test]
        fn skip_line_out_of_range_without_trailing_ln() {
            let cursor = Cursor::new("one\ntwo\nthree");
            let mut line_reader = LineReader::new(cursor);

            line_reader.skip_lines(4).unwrap();
            assert_eq!(line_reader.current_line, 3);

            let mut buf = Vec::new();
            line_reader.reader.read_to_end(&mut buf).unwrap();
            assert_eq!(buf, b"");
        }
    }

    mod read_specific_line {
        use super::*;

        #[test]
        fn input_with_trailing_new_line() {
            let cursor = Cursor::new("one\ntwo\nthree\n");
            let mut line_reader = LineReader::new(cursor);

            let mut buf = Vec::new();

            line_reader.read_specific_line(&mut buf, 0).unwrap();
            assert_eq!(buf, b"one\n");
            buf.clear();

            line_reader.read_specific_line(&mut buf, 2).unwrap();
            assert_eq!(buf, b"three\n");
            buf.clear();

            line_reader.read_specific_line(&mut buf, 4).unwrap();
            assert_eq!(buf, b"");
            buf.clear();
        }

        #[test]
        fn input_without_trailing_new_line() {
            let cursor = Cursor::new("one\ntwo\nthree");
            let mut line_reader = LineReader::new(cursor);

            let mut buf = Vec::new();

            line_reader.read_specific_line(&mut buf, 0).unwrap();
            assert_eq!(buf, b"one\n");
            buf.clear();

            line_reader.read_specific_line(&mut buf, 2).unwrap();
            assert_eq!(buf, b"three");
            buf.clear();

            line_reader.read_specific_line(&mut buf, 4).unwrap();
            assert_eq!(buf, b"");
            buf.clear();
        }
    }
}
