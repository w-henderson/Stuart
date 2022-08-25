use crate::functions::{quiet_assert, Function, FunctionParser};
use crate::parse::{ParseError, RawFunction};

pub struct DateFormatParser;

#[derive(Debug, Clone)]
pub struct DateFormatFunction {
    variable_name: String,
}

impl FunctionParser for DateFormatParser {
    fn name(&self) -> &'static str {
        "dateformat"
    }

    fn parse(&self, raw: RawFunction) -> Result<Box<dyn Function>, ParseError> {
        quiet_assert(raw.positional_args.len() == 1)?;
        quiet_assert(raw.named_args.is_empty())?;

        let variable_name = raw.positional_args[0]
            .as_variable()
            .ok_or(ParseError::InvalidArgument)?;

        Ok(Box::new(DateFormatFunction {
            variable_name: variable_name.to_string(),
        }))
    }
}

impl Function for DateFormatFunction {
    fn execute(&self) {
        todo!()
    }
}
