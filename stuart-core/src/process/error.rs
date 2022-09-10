//! Provides the [`ProcessError`] type.

use crate::fs;

pub use crate::TracebackError;

/// Represents an error which can occur during the processing of a file.
#[derive(Clone, Debug)]
pub enum ProcessError {
    /// The `root.html` file was not found.
    MissingHtmlRoot,
    /// The `md.html` file was not found.
    MissingMarkdownRoot,
    /// An error occurred with the stack.
    StackError,
    /// An `end(x)` function was called without a previous `begin(x)`.
    EndWithoutBegin,
    /// An `else()` function was called without a previous `ifeq`, `ifne`, etc.
    ElseWithoutIf,
    /// A JSON array was expected but not found.
    NotJsonArray,
    /// An invalid date was found.
    InvalidDate,
    /// The end of the file was reached unexpectedly.
    UnexpectedEndOfFile,
    /// The project has not yet been built, but its build output is required for this operation.
    NotBuilt,
    /// Metadata was requested, but its generation is not enabled in the configuration.
    MetadataNotEnabled,
    /// A filesystem error occurred.
    Fs(fs::Error),
    /// The variable already exists.
    VariableAlreadyExists(String),
    /// The variable does not exist.
    UndefinedVariable(String),
    /// The function does not exist.
    UndefinedSection(String),
    /// The variable is null.
    NullError(String),
    /// The file was not found.
    NotFound(String),

    /// The data type of the variable was invalid.
    InvalidDataType {
        /// The name of the variable.
        variable: String,
        /// The expected data type.
        expected: String,
        /// The actual data type.
        found: String,
    },
}
