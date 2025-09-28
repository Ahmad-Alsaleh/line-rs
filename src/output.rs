use crate::{cli::When, line_selector::ParsedLineSelector};
use std::io::Write;

mod colored_and_decorated;
mod colored_and_not_decorated;
mod not_colored_decorated;
mod not_colored_not_decorated;

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

pub(crate) trait OutputWriter: Write {
    fn print_line(&mut self, line: Line<'_>) -> anyhow::Result<()>;
    fn print_line_selector_header(
        &mut self,
        line_selector: &ParsedLineSelector,
        first_line: bool,
    ) -> anyhow::Result<()>;
}

pub(crate) fn get_output_writer<W>(
    writer: W,
    color: When,
    plain: bool,
    is_terminal: bool,
) -> Box<dyn OutputWriter>
where
    W: Write + 'static,
{
    // TODO: respect env vars: https://bixense.com/clicolors/
    // you can use: https://docs.rs/anstream/latest/anstream/struct.AutoStream.html
    let color = match color {
        When::Auto => is_terminal,
        When::Always => true,
        When::Never => false,
    };
    match (color, plain) {
        (true, false) => Box::new(colored_and_decorated::Writer(writer)),
        (true, true) => Box::new(colored_and_not_decorated::Writer(writer)),
        (false, false) => Box::new(not_colored_decorated::Writer(writer)),
        (false, true) => Box::new(not_colored_not_decorated::Writer(writer)),
    }
}
