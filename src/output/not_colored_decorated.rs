use crate::line_selector::{LineSelector, RawLineSelector};
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
                write!(self, "{line_num}: ", line_num = line_num + 1)?;
                self.write_all(line)?;
            }
        }

        Ok(())
    }

    fn print_line_selector_header(
        &mut self,
        line_selector: &LineSelector,
        first_line: bool,
    ) -> anyhow::Result<()> {
        if !first_line {
            writeln!(self)?;
        }
        let prefix = match line_selector.raw {
            RawLineSelector::Single(_) => "Line",
            RawLineSelector::Range(..) => "Lines",
            RawLineSelector::RangeWithStep(..) => "Lines",
        };
        writeln!(self, "{prefix}: {}", line_selector.raw)?;
        Ok(())
    }
}
