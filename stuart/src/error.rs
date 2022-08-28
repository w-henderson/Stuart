use crate::fs;
use crate::parse::ParseError;
use crate::process::ProcessError;

use termcolor::{Buffer, BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

use std::fmt::{Debug, Display};
use std::io::Write;
use std::path::PathBuf;

pub trait StuartError {
    fn display(&self, buf: &mut Buffer);

    fn print(&self) {
        let writer = BufferWriter::stderr(ColorChoice::Always);
        let mut buffer = writer.buffer();

        buffer
            .set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_intense(true))
            .unwrap();
        write!(buffer, "error: ").unwrap();
        buffer.reset().unwrap();

        self.display(&mut buffer);
        writer.print(&buffer).unwrap();
    }
}

#[derive(Clone, Debug)]
pub struct TracebackError<T: Clone + Debug> {
    pub(crate) path: PathBuf,
    pub(crate) line: u32,
    pub(crate) column: u32,
    pub(crate) kind: T,
}

impl<T: Clone + Debug + StuartError> StuartError for TracebackError<T> {
    fn display(&self, buf: &mut Buffer) {
        self.kind.display(buf);
        writeln!(
            buf,
            "  at {}:{}:{}",
            self.path.display(),
            self.line,
            self.column
        )
        .unwrap();
    }
}

impl StuartError for fs::Error {
    fn display(&self, buf: &mut Buffer) {
        match self {
            fs::Error::NotFound(s) => format!("not found: {}", s).display(buf),
            fs::Error::Read => "could not read from filesystem".display(buf),
            fs::Error::Write => "could not write to filesystem".display(buf),
            fs::Error::Parse(e) => e.display(buf),
        }
    }
}

impl StuartError for ParseError {
    fn display(&self, buf: &mut Buffer) {
        match self {
            ParseError::UnexpectedEOF => "unexpected end of file".display(buf),
            ParseError::Expected(expected) => format!("expected {}", expected).display(buf),
            ParseError::InvalidVariableName(name) => {
                format!("invalid variable name: {}", name).display(buf)
            }
            ParseError::InvalidFunctionName(name) => {
                format!("invalid function name: {}", name).display(buf)
            }
            ParseError::InvalidArgument => "invalid argument".display(buf),
            ParseError::NonexistentFunction(name) => {
                format!("function does not exist: {}", name).display(buf)
            }
            ParseError::GenericSyntaxError => "syntax error".display(buf),
            ParseError::PositionalArgAfterNamedArg => {
                "positional arguments cannot follow named arguments".display(buf)
            }
            ParseError::InvalidFrontmatter => "invalid frontmatter".display(buf),
            ParseError::InvalidJson => "invalid json".display(buf),
            ParseError::AssertionError(assertion) => {
                format!("assertion failed: {}", assertion).display(buf)
            }
        }
    }
}

impl StuartError for ProcessError {
    fn display(&self, buf: &mut Buffer) {
        match self {
            ProcessError::MissingHtmlRoot => "cannot find `root.html` template".display(buf),
            ProcessError::MissingMarkdownRoot => "cannot find `md.html` template".display(buf),
            ProcessError::StackError => "stack error".display(buf),
            ProcessError::EndWithoutBegin => "no matching `begin` for `end`".display(buf),
            ProcessError::NotJsonArray => "not a json array".display(buf),
            ProcessError::InvalidDate => "invalid date".display(buf),
            ProcessError::UnexpectedEndOfFile => "unexpected end of file".display(buf),
            ProcessError::Save(e) => e.display(buf),
            ProcessError::UndefinedVariable(name) => {
                format!("undefined variable: {}", name).display(buf)
            }
            ProcessError::UndefinedSection(name) => {
                format!("undefined section: {}", name).display(buf)
            }
            ProcessError::NullError(name) => format!("null error: {}", name).display(buf),
            ProcessError::NotFound(name) => format!("not found: {}", name).display(buf),
            ProcessError::InvalidDataType {
                variable,
                expected,
                found,
            } => if found.is_empty() {
                format!(
                    "type error in variable `{}`: expected {}",
                    variable, expected
                )
            } else {
                format!(
                    "type error in variable `{}`: expected {} but found {}",
                    variable, expected, found
                )
            }
            .display(buf),
        }
    }
}

impl<T: Display> StuartError for T {
    fn display(&self, buf: &mut Buffer) {
        writeln!(buf, "{}", self).unwrap();
    }
}

impl<T: StuartError + 'static> From<T> for Box<dyn StuartError> {
    fn from(t: T) -> Self {
        Box::new(t)
    }
}
