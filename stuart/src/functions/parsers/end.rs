use crate::functions::{quiet_assert, Function, FunctionParser};
use crate::parse::{ParseError, RawArgument, RawFunction};
use crate::process::{ProcessError, Scope};

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
    fn name(&self) -> &'static str {
        "end"
    }

    fn execute(&self, scope: &mut Scope) -> Result<(), ProcessError> {
        match (self.custom, self.label.as_str()) {
            (true, _) => {
                let frame = scope.stack.pop().ok_or(ProcessError::EndWithoutBegin)?;

                if frame.name != format!("begin:{}", self.label) {
                    return Err(ProcessError::EndWithoutBegin);
                }

                scope
                    .stack
                    .last_mut()
                    .ok_or(ProcessError::StackError)?
                    .output
                    .extend_from_slice(&frame.output);

                scope.sections.push((self.label.clone(), frame.output));
            }
            (false, "for") => {
                let frame = scope.stack.pop().ok_or(ProcessError::EndWithoutBegin)?;

                if !frame.name.starts_with("for:") {
                    return Err(ProcessError::EndWithoutBegin);
                }

                scope
                    .stack
                    .last_mut()
                    .ok_or(ProcessError::StackError)?
                    .output
                    .extend_from_slice(&frame.output);
            }
            _ => {
                return Err(ProcessError::EndWithoutBegin);
            }
        }

        Ok(())
    }
}
