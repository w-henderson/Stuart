use std::fmt::Debug;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub enum Error {
    Fs(FsError),
    Parse(TracebackError<ParseError>),
    Process(TracebackError<ProcessError>),

    /// The project has not yet been built, but its build output is required for this operation.
    NotBuilt,
    /// Metadata was requested, but its generation is not enabled in the configuration.
    MetadataNotEnabled,
}

/// Encapsulates an error and its location.
#[derive(Clone, Debug)]
pub struct TracebackError<T: Clone + Debug> {
    /// The path of the file in which the error occurred.
    pub path: PathBuf,
    /// The line number at which the error occurred.
    pub line: u32,
    /// The column number at which the error occurred.
    pub column: u32,
    /// The error.
    pub kind: T,
}

/// A filesystem error.
#[derive(Clone, Debug)]
pub enum FsError {
    /// The filesystem source could not be found.
    NotFound(String),
    /// The filesystem source could not be read.
    Read,
    /// The filesystem source could not be written.
    Write,
    /// A conflict occurred when merging two virtual filesystems.
    Conflict(PathBuf, PathBuf),
}

/// Represents an error which can occur during the parsing of a file.
#[derive(Clone, Debug)]
pub enum ParseError {
    /// The end of the file was reached unexpectedly.
    UnexpectedEOF,
    /// An expectation was not met.
    Expected(String),
    /// The variable name was invalid.
    InvalidVariableName(String),
    /// The function name was invalid.
    InvalidFunctionName(String),
    /// An argument to the function was invalid.
    InvalidArgument,
    /// The function did not exist
    NonexistentFunction(String),
    /// A syntax error, which cannot be otherwise classified, occurred.
    GenericSyntaxError,
    /// A positional argument was found after a named argument.
    PositionalArgAfterNamedArg,
    /// The frontmatter of a markdown file was invalid.
    InvalidFrontmatter,
    /// A JSON file contained invalid JSON.
    InvalidJson,
    /// An assertion with the [`quiet_assert`] macro failed.
    AssertionError(String),
}

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
