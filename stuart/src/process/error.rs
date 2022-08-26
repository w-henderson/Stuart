pub use crate::TracebackError;

#[derive(Clone, Debug)]
pub enum ProcessError {
    MissingHtmlRoot,
    MissingMarkdownRoot,
    StackError,
    EndWithoutBegin,
    UndefinedVariable(String),
    UndefinedSection(String),
    NullError(String),

    InvalidDataType {
        variable: String,
        expected: String,
        found: String,
    },
}
