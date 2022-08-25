use crate::functions::{quiet_assert, Function, FunctionParser};
use crate::parse::{ParseError, RawFunction};

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
        quiet_assert(raw.positional_args.len() == 1)?;
        quiet_assert(raw.named_args.is_empty())?;

        let string = raw.positional_args[0]
            .as_string()
            .ok_or(ParseError::InvalidArgument)?;

        Ok(Box::new(InsertFunction {
            label: string.to_string(),
        }))
    }
}

impl Function for InsertFunction {
    fn execute(&self) {
        todo!()
    }
}
