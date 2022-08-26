use crate::functions::{quiet_assert, Function, FunctionParser};
use crate::parse::{ParseError, RawFunction};
use crate::process::{ProcessError, Scope};

pub struct ExcerptParser;

#[derive(Debug, Clone)]
pub struct ExcerptFunction {
    variable_name: String,
    length: u16,
}

impl FunctionParser for ExcerptParser {
    fn name(&self) -> &'static str {
        "excerpt"
    }

    fn parse(&self, raw: RawFunction) -> Result<Box<dyn Function>, ParseError> {
        quiet_assert(raw.positional_args.len() == 2)?;
        quiet_assert(raw.named_args.is_empty())?;

        let variable_name = raw.positional_args[0]
            .as_variable()
            .ok_or(ParseError::InvalidArgument)?;

        let length: u16 = raw.positional_args[1]
            .as_integer()
            .ok_or(ParseError::InvalidArgument)?
            .try_into()
            .map_err(|_| ParseError::InvalidArgument)?;

        Ok(Box::new(ExcerptFunction {
            variable_name: variable_name.to_string(),
            length,
        }))
    }
}

impl Function for ExcerptFunction {
    fn execute(&self, scope: &mut Scope) -> Result<(), ProcessError> {
        todo!()
    }
}
