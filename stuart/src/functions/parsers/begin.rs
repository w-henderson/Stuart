use crate::functions::{quiet_assert, Function, FunctionParser};
use crate::parse::{ParseError, RawArgument, RawFunction};

pub struct BeginParser;

#[derive(Debug, Clone)]
pub struct BeginFunction {
    label: String,
    custom: bool,
}

impl FunctionParser for BeginParser {
    fn name(&self) -> &'static str {
        "begin"
    }

    fn parse(&self, raw: RawFunction) -> Result<Box<dyn Function>, ParseError> {
        quiet_assert(raw.positional_args.len() == 1)?;
        quiet_assert(raw.named_args.is_empty())?;

        match &raw.positional_args[0] {
            RawArgument::String(label) => Ok(Box::new(BeginFunction {
                label: label.to_string(),
                custom: true,
            })),
            RawArgument::Ident(label) => Ok(Box::new(BeginFunction {
                label: label.to_string(),
                custom: false,
            })),
            _ => Err(ParseError::InvalidArgument),
        }
    }
}

impl Function for BeginFunction {
    fn execute(&self) {
        todo!()
    }
}