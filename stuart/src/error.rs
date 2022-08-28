use crate::fs;
use crate::parse::ParseError;
use crate::process::ProcessError;

use termcolor::{Buffer, BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

use std::env::current_dir;
use std::fmt::{Debug, Display};
use std::fs::read_to_string;
use std::io::Write;
use std::path::PathBuf;

pub trait StuartError {
    fn display(&self, buf: &mut Buffer);

    fn help(&self) -> Option<String> {
        None
    }

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
        let relative_path = if let Ok(dir) = current_dir().and_then(std::fs::canonicalize) {
            self.path.strip_prefix(dir).unwrap_or(&self.path)
        } else {
            &self.path
        };

        let path = relative_path
            .to_string_lossy()
            .to_string()
            .trim_start_matches("\\\\?\\")
            .to_string();

        let line = read_to_string(&self.path)
            .ok()
            .and_then(|s| s.lines().nth(self.line as usize - 1).map(|s| s.to_string()));

        // Output first line (e.g. `error: some error message`)
        buf.set_color(
            ColorSpec::new()
                .set_fg(Some(Color::White))
                .set_intense(true),
        )
        .unwrap();
        self.kind.display(buf);
        buf.reset().unwrap();

        if let Some(line) = line {
            // Output location line
            buf.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_intense(true))
                .unwrap();
            write!(buf, "  --> ").unwrap();
            buf.reset().unwrap();
            writeln!(buf, "{}:{}:{}", path, self.line, self.column).unwrap();

            // Output preview
            let line_number_length = self.line.to_string().len();
            buf.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_intense(true))
                .unwrap();
            writeln!(buf, "{}|", " ".repeat(line_number_length + 1)).unwrap();
            write!(buf, "{} | ", self.line).unwrap();
            buf.reset().unwrap();
            writeln!(buf, "{}", line).unwrap();
            buf.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_intense(true))
                .unwrap();
            write!(buf, "{}| ", " ".repeat(line_number_length + 1)).unwrap();
            buf.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_intense(true))
                .unwrap();
            writeln!(
                buf,
                "{}^^^ error occurred here",
                " ".repeat((self.column as i32 - 2).clamp(0, i32::MAX) as usize)
            )
            .unwrap();
            buf.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_intense(true))
                .unwrap();
            writeln!(buf, "{}|", " ".repeat(line_number_length + 1)).unwrap();

            // Output help
            if let Some(help) = self.kind.help() {
                buf.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_intense(true))
                    .unwrap();
                write!(buf, "{}= ", " ".repeat(line_number_length + 1)).unwrap();
                buf.set_color(
                    ColorSpec::new()
                        .set_fg(Some(Color::White))
                        .set_intense(true),
                )
                .unwrap();
                write!(buf, "help: ").unwrap();
                buf.reset().unwrap();
                writeln!(buf, "{}", help).unwrap();
            } else {
                buf.reset().unwrap();
            }
        } else {
            writeln!(buf, "  at {}:{}:{}", path, self.line, self.column).unwrap();

            if let Some(help) = self.kind.help() {
                writeln!(buf, "  help: {}", help).unwrap();
            }
        }
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

    fn help(&self) -> Option<String> {
        match self {
            fs::Error::NotFound(_) => {
                Some("ensure that the file path is typed correctly".to_string())
            }
            fs::Error::Read => Some(
                "are any other processes using the file or directory, such as the terminal?"
                    .to_string(),
            ),
            fs::Error::Write => Some(
                "are any other processes using the file or directory, such as the terminal?"
                    .to_string(),
            ),
            fs::Error::Parse(e) => None,
        }
    }
}

impl StuartError for ParseError {
    fn display(&self, buf: &mut Buffer) {
        match self {
            ParseError::UnexpectedEOF => "unexpected end of file".display(buf),
            ParseError::Expected(expected) => format!("expected `{}`", expected).display(buf),
            ParseError::InvalidVariableName(name) => {
                format!("invalid variable name: `{}`", name).display(buf)
            }
            ParseError::InvalidFunctionName(name) => {
                format!("invalid function name: `{}`", name).display(buf)
            }
            ParseError::InvalidArgument => "invalid argument".display(buf),
            ParseError::NonexistentFunction(name) => {
                format!("function does not exist: `{}`", name).display(buf)
            }
            ParseError::GenericSyntaxError => "syntax error".display(buf),
            ParseError::PositionalArgAfterNamedArg => {
                "positional argument after named argument".display(buf)
            }
            ParseError::InvalidFrontmatter => "invalid frontmatter".display(buf),
            ParseError::InvalidJson => "invalid json".display(buf),
            ParseError::AssertionError(assertion) => {
                format!("assertion failed: `{}`", assertion).display(buf)
            }
        }
    }

    fn help(&self) -> Option<String> {
        match self {
            ParseError::UnexpectedEOF => Some("are you missing a closing command?".to_string()),
            ParseError::Expected(expected) => Some(format!("try adding `{}`", expected)),
            ParseError::InvalidVariableName(_) => Some(
                "variable names must not be empty and can only contain certain characters"
                    .to_string(),
            ),
            ParseError::InvalidFunctionName(_) => Some(
                "function names must not be empty and can only contain certain characters"
                    .to_string(),
            ),
            ParseError::InvalidArgument => None,
            ParseError::NonexistentFunction(_) => None,
            ParseError::GenericSyntaxError => None,
            ParseError::PositionalArgAfterNamedArg => {
                Some("place positional arguments before named arguments".to_string())
            }
            ParseError::InvalidFrontmatter => None,
            ParseError::InvalidJson => None,
            ParseError::AssertionError(_) => None,
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
                format!("undefined variable: `{}`", name).display(buf)
            }
            ProcessError::UndefinedSection(name) => {
                format!("undefined section: `{}`", name).display(buf)
            }
            ProcessError::NullError(name) => format!("null error: `{}`", name).display(buf),
            ProcessError::NotFound(name) => format!("not found: `{}`", name).display(buf),
            ProcessError::InvalidDataType {
                variable,
                expected,
                found,
            } => if found.is_empty() {
                format!(
                    "type error in variable `{}`: expected `{}`",
                    variable, expected
                )
            } else {
                format!(
                    "type error in variable `{}`: expected `{}` but found `{}`",
                    variable, expected, found
                )
            }
            .display(buf),
        }
    }

    fn help(&self) -> Option<String> {
        match self {
            ProcessError::MissingHtmlRoot => {
                Some("ensure that your `root.html` template exists and is accessible".to_string())
            }
            ProcessError::MissingMarkdownRoot => {
                Some("ensure that your `md.html` template exists and is accessible".to_string())
            }
            ProcessError::StackError => {
                Some("this shouldn't have happened, please open an issue!".to_string())
            }
            ProcessError::EndWithoutBegin => None,
            ProcessError::NotJsonArray => {
                Some("only arrays can be used in this context".to_string())
            }
            ProcessError::InvalidDate => {
                Some("ensure the date is valid and the format is correct".to_string())
            }
            ProcessError::UnexpectedEndOfFile => None,
            ProcessError::Save(e) => e.help(),
            ProcessError::UndefinedVariable(_) => None,
            ProcessError::UndefinedSection(_) => None,
            ProcessError::NullError(_) => Some(
                "if the variable is sometimes null, consider using the `ifdefined` function"
                    .to_string(),
            ),
            ProcessError::NotFound(_) => None,
            ProcessError::InvalidDataType { .. } => None,
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
