use std::io::BufRead;

/// Reads lines of a file in an efficeint way.
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
        fn input_with_trailing_new_line() -> anyhow::Result<()> {
            let cursor = Cursor::new("one\ntwo\n");
            let mut line_reader = LineReader::new(cursor);
            assert_eq!(line_reader.current_line, 0);

            let mut buf = Vec::new();

            line_reader.read_next_line(&mut buf)?;
            assert_eq!(buf, b"one\n");
            assert_eq!(line_reader.current_line, 1);
            buf.clear();

            line_reader.read_next_line(&mut buf)?;
            assert_eq!(buf, b"two\n");
            assert_eq!(line_reader.current_line, 2);
            buf.clear();

            line_reader.read_next_line(&mut buf)?;
            assert_eq!(buf, b"");
            assert_eq!(line_reader.current_line, 2);
            buf.clear();

            Ok(())
        }

        #[test]
        fn input_without_trailing_new_line() -> anyhow::Result<()> {
            let cursor = Cursor::new("one\ntwo");
            let mut line_reader = LineReader::new(cursor);
            assert_eq!(line_reader.current_line, 0);

            let mut buf = Vec::new();

            line_reader.read_next_line(&mut buf)?;
            assert_eq!(buf, b"one\n");
            assert_eq!(line_reader.current_line, 1);
            buf.clear();

            line_reader.read_next_line(&mut buf)?;
            assert_eq!(buf, b"two");
            assert_eq!(line_reader.current_line, 2);
            buf.clear();

            line_reader.read_next_line(&mut buf)?;
            assert_eq!(buf, b"");
            assert_eq!(line_reader.current_line, 2);
            buf.clear();

            Ok(())
        }

        #[test]
        fn empty_input() -> anyhow::Result<()> {
            let cursor = Cursor::new("");
            let mut line_reader = LineReader::new(cursor);
            assert_eq!(line_reader.current_line, 0);

            let mut buf = Vec::new();

            line_reader.read_next_line(&mut buf)?;
            assert_eq!(buf, b"");
            assert_eq!(line_reader.current_line, 0);

            Ok(())
        }

        #[test]
        fn input_is_new_line_only() -> anyhow::Result<()> {
            let cursor = Cursor::new("\n");
            let mut line_reader = LineReader::new(cursor);
            assert_eq!(line_reader.current_line, 0);

            let mut buf = Vec::new();

            line_reader.read_next_line(&mut buf)?;
            assert_eq!(buf, b"\n");
            assert_eq!(line_reader.current_line, 1);

            Ok(())
        }

        #[test]
        fn no_read_permessions() -> anyhow::Result<()> {
            let temp_dir = tempfile::tempdir()?;
            let path = temp_dir.path().join("file.txt");

            // `File::create` creates a file with _write-only_ permessions
            let mut file = File::create(&path)?;
            write!(file, "one\ntwo\n")?;

            let mut line_reader = LineReader::new(BufReader::new(file));

            let mut buf = Vec::new();
            assert!(line_reader.read_next_line(&mut buf).is_err());

            Ok(())
        }
    }

    mod skip_lines {
        use super::*;

        #[test]
        fn skip_zero_lines() -> anyhow::Result<()> {
            let cursor = Cursor::new("one\ntwo\n");
            let mut line_reader = LineReader::new(cursor);

            line_reader.skip_lines(0)?;
            assert_eq!(line_reader.current_line, 0);

            let mut buf = Vec::new();
            line_reader.reader.read_to_end(&mut buf)?;
            assert_eq!(buf, b"one\ntwo\n");

            Ok(())
        }

        #[test]
        fn empty_input() -> anyhow::Result<()> {
            let cursor = Cursor::new("");
            let mut line_reader = LineReader::new(cursor);

            line_reader.skip_lines(0)?;
            assert_eq!(line_reader.current_line, 0);

            line_reader.skip_lines(10)?;
            assert_eq!(line_reader.current_line, 0);

            Ok(())
        }

        #[test]
        fn input_is_new_line_only() -> anyhow::Result<()> {
            let cursor = Cursor::new("\n");
            let mut line_reader = LineReader::new(cursor);

            line_reader.skip_lines(1)?;
            assert_eq!(line_reader.current_line, 1);

            let mut buf = Vec::new();
            line_reader.reader.read_to_end(&mut buf)?;
            assert_eq!(buf, b"");

            Ok(())
        }

        #[test]
        fn skip_line_in_range_with_trailing_ln() -> anyhow::Result<()> {
            let cursor = Cursor::new("one\ntwo\nthree\n");
            let mut line_reader = LineReader::new(cursor);

            line_reader.skip_lines(2)?;
            assert_eq!(line_reader.current_line, 2);

            let mut buf = Vec::new();
            line_reader.reader.read_to_end(&mut buf)?;
            assert_eq!(buf, b"three\n");

            Ok(())
        }

        #[test]
        fn skip_last_line_with_trailing_ln() -> anyhow::Result<()> {
            let cursor = Cursor::new("one\ntwo\nthree\n");
            let mut line_reader = LineReader::new(cursor);

            line_reader.skip_lines(3)?;
            assert_eq!(line_reader.current_line, 3);

            let mut buf = Vec::new();
            line_reader.reader.read_to_end(&mut buf)?;
            assert_eq!(buf, b"");

            Ok(())
        }

        #[test]
        fn skip_last_line_without_trailing_ln() -> anyhow::Result<()> {
            let cursor = Cursor::new("one\ntwo\nthree");
            let mut line_reader = LineReader::new(cursor);

            line_reader.skip_lines(3)?;
            assert_eq!(line_reader.current_line, 3);

            let mut buf = Vec::new();
            line_reader.reader.read_to_end(&mut buf)?;
            assert_eq!(buf, b"");

            Ok(())
        }

        #[test]
        fn skip_line_out_of_range_with_trailing_ln() -> anyhow::Result<()> {
            let cursor = Cursor::new("one\ntwo\nthree\n");
            let mut line_reader = LineReader::new(cursor);

            line_reader.skip_lines(4)?;
            assert_eq!(line_reader.current_line, 3);

            let mut buf = Vec::new();
            line_reader.reader.read_to_end(&mut buf)?;
            assert_eq!(buf, b"");

            Ok(())
        }

        #[test]
        fn skip_line_in_range_withno_trailing_ln() -> anyhow::Result<()> {
            let cursor = Cursor::new("one\ntwo\nthree");
            let mut line_reader = LineReader::new(cursor);

            line_reader.skip_lines(2)?;
            assert_eq!(line_reader.current_line, 2);

            let mut buf = Vec::new();
            line_reader.reader.read_to_end(&mut buf)?;
            assert_eq!(buf, b"three");

            Ok(())
        }

        #[test]
        fn skip_line_out_of_range_without_trailing_ln() -> anyhow::Result<()> {
            let cursor = Cursor::new("one\ntwo\nthree");
            let mut line_reader = LineReader::new(cursor);

            line_reader.skip_lines(4)?;
            assert_eq!(line_reader.current_line, 3);

            let mut buf = Vec::new();
            line_reader.reader.read_to_end(&mut buf)?;
            assert_eq!(buf, b"");

            Ok(())
        }
    }

    mod read_specific_line {
        use super::*;

        #[test]
        fn input_with_trailing_new_line() -> anyhow::Result<()> {
            let cursor = Cursor::new("one\ntwo\nthree\n");
            let mut line_reader = LineReader::new(cursor);

            let mut buf = Vec::new();

            line_reader.read_specific_line(&mut buf, 0)?;
            assert_eq!(buf, b"one\n");
            buf.clear();

            line_reader.read_specific_line(&mut buf, 2)?;
            assert_eq!(buf, b"three\n");
            buf.clear();

            line_reader.read_specific_line(&mut buf, 4)?;
            assert_eq!(buf, b"");
            buf.clear();

            Ok(())
        }

        #[test]
        fn input_without_trailing_new_line() -> anyhow::Result<()> {
            let cursor = Cursor::new("one\ntwo\nthree");
            let mut line_reader = LineReader::new(cursor);

            let mut buf = Vec::new();

            line_reader.read_specific_line(&mut buf, 0)?;
            assert_eq!(buf, b"one\n");
            buf.clear();

            line_reader.read_specific_line(&mut buf, 2)?;
            assert_eq!(buf, b"three");
            buf.clear();

            line_reader.read_specific_line(&mut buf, 4)?;
            assert_eq!(buf, b"");
            buf.clear();

            Ok(())
        }
    }
}
