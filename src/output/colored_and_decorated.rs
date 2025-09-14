use crate::line_selector::ParsedLineSelector;
use crate::output::{BLUE_BOLD, BOLD, CLEAR, GREEN_BOLD, Line, OutputWriter, RED};
use std::io::Write;

pub(crate) struct Writer<W: Write>(pub W);

impl<W: Write> OutputWriter for Writer<W> {
    fn print_line(&mut self, line: Line<'_>) -> anyhow::Result<()> {
        match line {
            Line::Context { line_num, line } => {
                write!(self.0, "{BOLD}{line_num}:{CLEAR} ", line_num = line_num + 1)?;
                self.0.write_all(line)?;
            }
            Line::Selected { line_num, line } => {
                write!(
                    self.0,
                    "{GREEN_BOLD}{line_num}:{CLEAR} {RED}",
                    line_num = line_num + 1
                )?;
                self.0.write_all(line)?;
                write!(self.0, "{CLEAR}")?;
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
        writeln!(self.0, "\n{BLUE_BOLD}Line: {line_selector:?}{CLEAR}")?;
        Ok(())
    }
}
