use crate::functions::{quiet_assert, Function, FunctionParser};
use crate::parse::{ParseError, RawArgument, RawFunction};

pub struct EndParser;

#[derive(Debug, Clone)]
pub struct EndFunction {
    label: String,
    custom: bool,
}

impl FunctionParser for EndParser {
    fn name(&self) -> &'static str {
        "end"
    }

    fn parse(&self, raw: RawFunction) -> Result<Box<dyn Function>, ParseError> {
        quiet_assert(raw.positional_args.len() == 1)?;
        quiet_assert(raw.named_args.is_empty())?;

        match &raw.positional_args[0] {
            RawArgument::String(label) => Ok(Box::new(EndFunction {
                label: label.to_string(),
                custom: true,
            })),
            RawArgument::Ident(label) => Ok(Box::new(EndFunction {
                label: label.to_string(),
                custom: false,
            })),
            _ => Err(ParseError::InvalidArgument),
        }
    }
}

impl Function for EndFunction {
    fn execute(&self) {
        todo!()
    }
}
