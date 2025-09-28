use crate::line_selector::ParsedLineSelector;
use crate::output::{Line, OutputWriter};
use std::io::Write;

pub(crate) struct Writer<W: Write>(pub W);

impl<W: Write> Write for Writer<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}

impl<W: Write> OutputWriter for Writer<W> {
    fn print_line(&mut self, line: Line<'_>) -> anyhow::Result<()> {
        match line {
            Line::Context { line_num, line } | Line::Selected { line_num, line } => {
                write!(self.0, "{line_num}: ", line_num = line_num + 1)?;
                self.0.write_all(line)?;
            }
        }

        Ok(())
    }

    // TODO: print the raw selectors, not the parsed ones. the parsed ones are internal and
    // shouldn't be user-facing. if the user selects `-n=-1` it'll be confusing to show the parsed
    // selectors
    fn print_line_selector_header(
        &mut self,
        line_selector: &ParsedLineSelector,
    ) -> anyhow::Result<()> {
        writeln!(self.0, "Line: {line_selector:?}")?;
        Ok(())
    }
}
