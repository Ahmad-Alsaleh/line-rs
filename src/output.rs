use crate::line_selector::ParsedLineSelector;
use std::io::Write;

// TODO (FIXME): handle SIGPIPE, eg: `line -n=: large_file.txt | head -n1`

// TODO: make this cross-platform
const RED: &str = "\x1b[31m";
const GREEN_BOLD: &str = "\x1b[32;1m";
const BOLD: &str = "\x1b[1m";
const CLEAR: &str = "\x1b[0m";
const BLUE_BOLD: &str = "\x1b[36;1m";

pub(crate) enum Line<'a> {
    Context { line_num: usize, line: &'a [u8] },
    Selected { line_num: usize, line: &'a [u8] },
}

pub(crate) trait OutputWriter {
    fn print_line(&mut self, line: Line<'_>) -> anyhow::Result<()>;
    fn print_line_selector_header(
        &mut self,
        line_selector: &ParsedLineSelector,
    ) -> anyhow::Result<()>;
}

/// No styles at all (no line numbers, headers, or colors)
pub(crate) struct PlainOutputWriter<W: Write>(pub W);

/// Line numbers and headers are displayed but without colors
pub(crate) struct NotColoredOutputWriter<W: Write>(pub W);

/// Full style (line numbers, headers, and colors)
pub(crate) struct ColoredOutputWriter<W: Write>(pub W);

impl<W: Write> OutputWriter for PlainOutputWriter<W> {
    fn print_line(&mut self, line: Line<'_>) -> anyhow::Result<()> {
        match line {
            Line::Context { line_num: _, line } | Line::Selected { line_num: _, line } => {
                self.0.write_all(line)?;
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

impl<W: Write> OutputWriter for NotColoredOutputWriter<W> {
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
        writeln!(self.0, "\nLine: {line_selector:?}")?;
        Ok(())
    }
}

impl<W: Write> OutputWriter for ColoredOutputWriter<W> {
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
