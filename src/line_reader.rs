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

    /// Returns `false` if no bytes were read and `true` otherwise.
    fn read_next_line(&mut self, buf: &mut Vec<u8>) -> anyhow::Result<()> {
        let n = self.reader.read_until(b'\n', buf)?;
        if n != 0 {
            self.current_line += 1;
        }
        Ok(())
    }

    /// Skips `n` lines.
    /// Returns `false` if reached EOF before skipping `n` lines.
    fn skip_lines(&mut self, n: usize) -> anyhow::Result<()> {
        let mut i = 0;
        while i < n && self.reader.skip_until(b'\n')? > 0 {
            i += 1;
        }
        self.current_line += i;
        Ok(())
    }

    /// `lines_num` should be more than `self.current_line`.
    /// `line_num` is zero-indexed.
    /// Returns `false` if `line_num` is beyod EOF and `true` otherwise.
    pub(crate) fn read_specific_line(
        &mut self,
        buf: &mut Vec<u8>,
        line_num: usize,
    ) -> anyhow::Result<()> {
        // avoid attempting to skip lines if there is no need
        if line_num != self.current_line {
            self.skip_lines(line_num - self.current_line)?;
        }
        self.read_next_line(buf)
    }
}
