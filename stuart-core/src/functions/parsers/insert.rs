use crate::functions::{Function, FunctionParser};
use crate::parse::{ParseError, RawFunction};
use crate::process::{ProcessError, Scope};
use crate::{quiet_assert, TracebackError};

/// Parses the `insert` function.
pub struct InsertParser;

#[derive(Debug, Clone)]
pub struct InsertFunction {
    label: String,
}

impl FunctionParser for InsertParser {
    fn name(&self) -> &'static str {
        "insert"
    }

    fn parse(&self, raw: RawFunction) -> Result<Box<dyn Function>, ParseError> {
        quiet_assert!(raw.positional_args.len() == 1)?;
        quiet_assert!(raw.named_args.is_empty())?;

        let string = raw.positional_args[0]
            .as_string()
            .ok_or(ParseError::InvalidArgument)?;

        Ok(Box::new(InsertFunction {
            label: string.to_string(),
        }))
    }
}

impl Function for InsertFunction {
    fn name(&self) -> &'static str {
        "insert"
    }

    fn execute(&self, scope: &mut Scope) -> Result<(), TracebackError<ProcessError>> {
        let self_token = scope.tokens.current().unwrap().clone();

        let frame = scope
            .stack
            .last_mut()
            .ok_or_else(|| self_token.traceback(ProcessError::StackError))?;

        let (_, section) = scope
            .sections
            .iter()
            .find(|(label, _)| label == &self.label)
            .ok_or_else(|| {
                self_token.traceback(ProcessError::UndefinedSection(self.label.clone()))
            })?;

        frame.output.extend_from_slice(section);

        Ok(())
    }
}
