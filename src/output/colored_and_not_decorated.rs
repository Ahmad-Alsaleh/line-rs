use crate::line_selector::ParsedLineSelector;
use crate::output::{CLEAR, Line, OutputWriter, RED};
use std::io::Write;

pub(crate) struct Writer<W: Write>(pub W);

impl<W: Write> OutputWriter for Writer<W> {
    fn print_line(&mut self, line: Line<'_>) -> anyhow::Result<()> {
        match line {
            Line::Context { line_num: _, line } => {
                self.0.write_all(line)?;
            }
            Line::Selected { line_num: _, line } => {
                write!(self.0, "{RED}")?;
                self.0.write_all(line)?;
                write!(self.0, "{CLEAR}")?;
            }
        }

        Ok(())
    }

    fn print_line_selector_header(
        &mut self,
        _line_selector: &ParsedLineSelector,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
