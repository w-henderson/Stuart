use stuart_core::functions::{Function, FunctionParser};
use stuart_core::parse::{ParseError, RawFunction};
use stuart_core::process::{ProcessError, Scope};
use stuart_core::{quiet_assert, TracebackError};

pub struct DemoParser;

#[derive(Debug, Clone)]
pub struct DemoFunction;

impl FunctionParser for DemoParser {
    fn name(&self) -> &'static str {
        "demo"
    }

    fn parse(&self, raw: RawFunction) -> Result<Box<dyn Function>, ParseError> {
        quiet_assert!(raw.positional_args.is_empty())?;
        quiet_assert!(raw.named_args.is_empty())?;

        Ok(Box::new(DemoFunction))
    }
}

impl Function for DemoFunction {
    fn name(&self) -> &'static str {
        "demo"
    }

    fn execute(&self, scope: &mut Scope) -> Result<(), TracebackError<ProcessError>> {
        scope
            .output("<p>This was written by the demo function!</p>")
            .unwrap();

        Ok(())
    }
}
