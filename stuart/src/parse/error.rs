#[derive(Debug)]
pub struct TracebackError {
    pub(crate) line: u32,
    pub(crate) column: u32,
    pub(crate) kind: ParseError,
}

#[derive(Debug)]
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
}
