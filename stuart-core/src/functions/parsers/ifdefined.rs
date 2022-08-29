use crate::functions::{Function, FunctionParser};
use crate::parse::{ParseError, RawFunction};
use crate::process::stack::StackFrame;
use crate::process::{ProcessError, Scope};
use crate::{quiet_assert, TracebackError};

use humphrey_json::Value;

pub struct IfDefinedParser;

#[derive(Debug, Clone)]
pub struct IfDefinedFunction {
    variable_name: String,
}

impl FunctionParser for IfDefinedParser {
    fn name(&self) -> &'static str {
        "ifdefined"
    }

    fn parse(&self, raw: RawFunction) -> Result<Box<dyn Function>, ParseError> {
        quiet_assert!(raw.positional_args.len() == 1)?;
        quiet_assert!(raw.named_args.is_empty())?;

        let variable_name = raw.positional_args[0]
            .as_variable()
            .ok_or(ParseError::InvalidArgument)?;

        Ok(Box::new(IfDefinedFunction {
            variable_name: variable_name.to_string(),
        }))
    }
}

impl Function for IfDefinedFunction {
    fn name(&self) -> &'static str {
        "ifdefined"
    }

    fn execute(&self, scope: &mut Scope) -> Result<(), TracebackError<ProcessError>> {
        let self_token = scope.tokens.current().unwrap().clone();

        let defined = scope
            .get_variable(&self.variable_name)
            .map(|v| !matches!(v, Value::Null))
            .unwrap_or(false);

        let frame = StackFrame::new(format!("ifdefined:{}", self.variable_name));

        let stack_height = scope.stack.len();
        scope.stack.push(frame);

        while scope.stack.len() > stack_height {
            let token = scope
                .tokens
                .next()
                .ok_or_else(|| self_token.traceback(ProcessError::UnexpectedEndOfFile))?;

            if defined
                || (token
                    .as_function()
                    .map(|f| f.name() == "end")
                    .unwrap_or(false)
                    && scope.stack.len() == stack_height + 1)
            {
                token.process(scope)?;
            }
        }

        Ok(())
    }
}
