pub use crate::TracebackError;

#[derive(Clone, Debug)]
pub enum ParseError {
    UnexpectedEOF,
    Expected(String),
    InvalidVariableName(String),
    InvalidFunctionName(String),
    InvalidArgument,
    NonexistentFunction(String),
    GenericSyntaxError,
    PositionalArgAfterNamedArg,
    InvalidFrontmatter,
    InvalidJson,
    AssertionError(String),
}
