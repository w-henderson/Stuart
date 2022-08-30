use crate::functions::{Function, FunctionParser};
use crate::parse::{ParseError, RawFunction};
use crate::process::{ProcessError, Scope};
use crate::{quiet_assert, TracebackError};

pub struct ExcerptParser;

#[derive(Debug, Clone)]
pub struct ExcerptFunction {
    variable_name: String,
    length: usize,
}

impl FunctionParser for ExcerptParser {
    fn name(&self) -> &'static str {
        "excerpt"
    }

    fn parse(&self, raw: RawFunction) -> Result<Box<dyn Function>, ParseError> {
        quiet_assert!(raw.positional_args.len() == 2)?;
        quiet_assert!(raw.named_args.is_empty())?;

        let variable_name = raw.positional_args[0]
            .as_variable()
            .ok_or(ParseError::InvalidArgument)?;

        let length: usize = raw.positional_args[1]
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
    fn name(&self) -> &'static str {
        "excerpt"
    }

    fn execute(&self, scope: &mut Scope) -> Result<(), TracebackError<ProcessError>> {
        let self_token = scope.tokens.current().unwrap().clone();

        let variable = scope.get_variable(&self.variable_name).ok_or_else(|| {
            self_token.traceback(ProcessError::UndefinedVariable(self.variable_name.clone()))
        })?;

        let string = variable.as_str().ok_or_else(|| {
            self_token.traceback(ProcessError::InvalidDataType {
                variable: self.variable_name.clone(),
                expected: "string".to_string(),
                found: String::new(),
            })
        })?;

        let mut chars = string.chars();
        let mut excerpt = String::with_capacity(self.length + 3);
        let mut tag = false;
        let mut total_chars: usize = 0;

        while excerpt.len() < self.length {
            if let Some(ch) = chars.next() {
                if ch == '<' {
                    tag = true;
                } else if ch == '>' {
                    tag = false;
                } else if !tag {
                    excerpt.push(ch);
                }

                total_chars += 1;
            } else {
                break;
            }
        }

        if total_chars < string.len() {
            excerpt.push_str("...");
        }

        scope.output(excerpt).map_err(|e| self_token.traceback(e))?;

        Ok(())
    }
}
