use crate::line_selector::ParsedLineSelector;
use crate::output::{BLUE_BOLD, BOLD, CLEAR, GREEN_BOLD, Line, OutputWriter, RED};
use std::io::Write;

pub(crate) struct Writer<W: Write>(pub W);

// TODO: consider making a macro to implement Write
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
            Line::Context { line_num, line } => {
                write!(self, "{BOLD}{line_num}:{CLEAR} ", line_num = line_num + 1)?;
                self.write_all(line)?;
            }
            Line::Selected { line_num, line } => {
                write!(
                    self,
                    "{GREEN_BOLD}{line_num}:{CLEAR} {RED}",
                    line_num = line_num + 1
                )?;
                self.write_all(line)?;
                write!(self, "{CLEAR}")?;
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
        first_line: bool,
    ) -> anyhow::Result<()> {
        if !first_line {
            writeln!(self)?;
        }
        writeln!(self, "{BLUE_BOLD}Line: {line_selector:?}{CLEAR}")?;
        Ok(())
    }
}
