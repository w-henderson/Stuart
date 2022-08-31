use crate::functions::{Function, FunctionParser};
use crate::parse::{ParseError, RawFunction};
use crate::process::{ProcessError, Scope};
use crate::{quiet_assert, TracebackError};

static WORDS_PER_MINUTE: usize = 200;

/// Parses the `timetoread` function.
pub struct TimeToReadParser;

#[derive(Debug, Clone)]
pub struct TimeToReadFunction {
    variable_name: String,
}

impl FunctionParser for TimeToReadParser {
    fn name(&self) -> &'static str {
        "timetoread"
    }

    fn parse(&self, raw: RawFunction) -> Result<Box<dyn Function>, ParseError> {
        quiet_assert!(raw.positional_args.len() == 1)?;
        quiet_assert!(raw.named_args.is_empty())?;

        let variable_name = raw.positional_args[0]
            .as_variable()
            .ok_or(ParseError::InvalidArgument)?;

        Ok(Box::new(TimeToReadFunction {
            variable_name: variable_name.to_string(),
        }))
    }
}

impl Function for TimeToReadFunction {
    fn name(&self) -> &'static str {
        "timetoread"
    }

    fn execute(&self, scope: &mut Scope) -> Result<(), TracebackError<ProcessError>> {
        let self_token = scope.tokens.current().unwrap().clone();

        let variable = scope.get_variable(&self.variable_name).ok_or_else(|| {
            self_token.traceback(ProcessError::UndefinedVariable(self.variable_name.clone()))
        })?;

        let string = variable.as_str().ok_or_else(|| {
            self_token.traceback(ProcessError::InvalidDataType {
                variable: self.variable_name.clone(),
                expected: "string".to_string(),
                found: String::new(),
            })
        })?;

        let words = string.split_whitespace().count();
        let minutes = (words / WORDS_PER_MINUTE).max(1);

        scope
            .output(minutes.to_string())
            .map_err(|e| self_token.traceback(e))?;

        Ok(())
    }
}
