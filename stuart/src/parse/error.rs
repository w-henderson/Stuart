pub use crate::TracebackError;

#[derive(Clone, Debug)]
pub enum ParseError {
    UnexpectedEOF,
    Expected(String),
    InvalidLabelName,
    InvalidVariableName,
    InvalidFunctionName,
    InvalidArgument,
    NonexistentFunction(String),
    FunctionWithoutParens,
    GenericSyntaxError,
    PositionalArgAfterNamedArg,
    InvalidFrontmatter,
    InvalidJson,
    AssertionError(String),
}
