use crate::functions::{Function, FunctionParser};
use crate::parse::{ParseError, RawFunction};
use crate::process::{ProcessError, Scope};
use crate::quiet_assert;

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

    fn execute(&self, scope: &mut Scope) -> Result<(), ProcessError> {
        todo!()
    }
}
