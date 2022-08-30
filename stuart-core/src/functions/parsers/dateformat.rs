use crate::functions::{Function, FunctionParser};
use crate::parse::{ParseError, RawFunction};
use crate::process::{ProcessError, Scope};
use crate::{quiet_assert, TracebackError};

use chrono::{Local, NaiveTime};
use dateparser::parse_with;

pub struct DateFormatParser;

#[derive(Debug, Clone)]
pub struct DateFormatFunction {
    variable_name: String,
    format: String,
}

impl FunctionParser for DateFormatParser {
    fn name(&self) -> &'static str {
        "dateformat"
    }

    fn parse(&self, raw: RawFunction) -> Result<Box<dyn Function>, ParseError> {
        quiet_assert!(raw.positional_args.len() == 2)?;
        quiet_assert!(raw.named_args.is_empty())?;

        let variable_name = raw.positional_args[0]
            .as_variable()
            .ok_or(ParseError::InvalidArgument)?
            .to_string();

        let format = raw.positional_args[1]
            .as_string()
            .ok_or(ParseError::InvalidArgument)?
            .to_string();

        Ok(Box::new(DateFormatFunction {
            variable_name,
            format,
        }))
    }
}

impl Function for DateFormatFunction {
    fn name(&self) -> &'static str {
        "dateformat"
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

        let date = std::panic::catch_unwind(|| {
            parse_with(string, &Local, NaiveTime::from_hms(0, 0, 0))
                .ok()
                .map(|d| d.format(&self.format).to_string())
                .ok_or(ProcessError::InvalidDate)
        })
        .map_err(|_| self_token.traceback(ProcessError::InvalidDate))?
        .map_err(|_| self_token.traceback(ProcessError::InvalidDate))?;

        scope.output(date).map_err(|e| self_token.traceback(e))?;

        Ok(())
    }
}
