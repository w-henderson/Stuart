//! Provides the [`ParseError`] type.

pub use crate::TracebackError;

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
