use crate::fs;

pub use crate::TracebackError;

#[derive(Clone, Debug)]
pub enum ProcessError {
    MissingHtmlRoot,
    MissingMarkdownRoot,
    StackError,
    EndWithoutBegin,
    NotJsonArray,
    InvalidDate,
    UnexpectedEndOfFile,
    Save(fs::Error),
    UndefinedVariable(String),
    UndefinedSection(String),
    NullError(String),
    NotFound(String),

    InvalidDataType {
        variable: String,
        expected: String,
        found: String,
    },
}
