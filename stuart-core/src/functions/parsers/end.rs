use crate::functions::{Function, FunctionParser};
use crate::parse::{ParseError, RawArgument, RawFunction};
use crate::process::{ProcessError, Scope};
use crate::{quiet_assert, TracebackError};

/// Parses the `end` function.
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
        quiet_assert!(raw.positional_args.len() == 1)?;
        quiet_assert!(raw.named_args.is_empty())?;

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
    fn name(&self) -> &'static str {
        "end"
    }

    fn execute(&self, scope: &mut Scope) -> Result<(), TracebackError<ProcessError>> {
        let self_token = scope.tokens.current().unwrap().clone();

        match (self.custom, self.label.as_str()) {
            (true, _) => {
                let frame = scope
                    .stack
                    .pop()
                    .ok_or_else(|| self_token.traceback(ProcessError::EndWithoutBegin))?;

                if frame.name != format!("begin:{}", self.label) {
                    return Err(self_token.traceback(ProcessError::EndWithoutBegin));
                }

                scope
                    .output(&frame.output)
                    .map_err(|e| self_token.traceback(e))?;

                scope.sections.push((self.label.clone(), frame.output));
            }
            (false, label) => {
                let frame = scope
                    .stack
                    .pop()
                    .ok_or_else(|| self_token.traceback(ProcessError::EndWithoutBegin))?;

                if !frame.name.starts_with(&format!("{}:", label)) {
                    return Err(self_token.traceback(ProcessError::EndWithoutBegin));
                }

                scope
                    .output(frame.output)
                    .map_err(|e| self_token.traceback(e))?;
            }
        }

        Ok(())
    }
}
