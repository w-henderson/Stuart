use crate::functions::{Function, FunctionParser};
use crate::parse::{ParseError, RawFunction};
use crate::process::{ProcessError, Scope};
use crate::{quiet_assert, TracebackError};

/// Parses the `else` function.
pub struct ElseParser;

#[derive(Debug, Clone)]
pub struct ElseFunction;

impl FunctionParser for ElseParser {
    fn name(&self) -> &'static str {
        "else"
    }

    fn parse(&self, raw: RawFunction) -> Result<Box<dyn Function>, ParseError> {
        quiet_assert!(raw.positional_args.is_empty())?;
        quiet_assert!(raw.named_args.is_empty())?;

        Ok(Box::new(ElseFunction))
    }
}

impl Function for ElseFunction {
    fn name(&self) -> &'static str {
        "else"
    }

    fn execute(&self, scope: &mut Scope) -> Result<(), TracebackError<ProcessError>> {
        let self_token = scope.tokens.current().unwrap().clone();

        let name = &scope
            .stack
            .last()
            .ok_or_else(|| self_token.traceback(ProcessError::ElseWithoutIf))?
            .name;

        if !name.starts_with("if") {
            return Err(self_token.traceback(ProcessError::ElseWithoutIf));
        }

        Ok(())
    }
}
