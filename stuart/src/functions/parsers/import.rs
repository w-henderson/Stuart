use crate::fs::ParsedContents;
use crate::functions::{Function, FunctionParser};
use crate::parse::{ParseError, RawFunction};
use crate::process::{ProcessError, Scope};
use crate::{quiet_assert, TracebackError};

use std::path::PathBuf;

pub struct ImportParser;

#[derive(Debug, Clone)]
pub struct ImportFunction {
    variable_name: String,
    file_name: String,
}

impl FunctionParser for ImportParser {
    fn name(&self) -> &'static str {
        "import"
    }

    fn parse(&self, raw: RawFunction) -> Result<Box<dyn Function>, ParseError> {
        quiet_assert!(raw.positional_args.len() == 2)?;
        quiet_assert!(raw.named_args.is_empty())?;

        let variable_name = raw.positional_args[0]
            .as_variable()
            .ok_or(ParseError::InvalidArgument)?
            .to_string();

        let file_name = raw.positional_args[1]
            .as_string()
            .ok_or(ParseError::InvalidArgument)?
            .to_string();

        Ok(Box::new(ImportFunction {
            variable_name,
            file_name,
        }))
    }
}

impl Function for ImportFunction {
    fn name(&self) -> &'static str {
        "import"
    }

    fn execute(&self, scope: &mut Scope) -> Result<(), TracebackError<ProcessError>> {
        let self_token = scope.tokens.current().unwrap().clone();

        let file = scope
            .processor
            .fs
            .get_at_path(&PathBuf::from(self.file_name.clone()))
            .ok_or_else(|| self_token.traceback(ProcessError::NotFound(self.file_name.clone())))?;

        if !file.is_file() {
            return Err(self_token.traceback(ProcessError::NotFound(self.file_name.clone())));
        }

        let json = match file.parsed_contents() {
            ParsedContents::Json(json) => Some(json.clone()),
            _ => None,
        }
        .ok_or_else(|| {
            self_token.traceback(ProcessError::InvalidDataType {
                variable: "<file>".to_string(),
                expected: "json".to_string(),
                found: String::new(),
            })
        })?;

        let frame = scope
            .stack
            .last_mut()
            .ok_or_else(|| self_token.traceback(ProcessError::StackError))?;

        if frame.get_variable(&self.variable_name).is_some() {
            return Err(self_token.traceback(ProcessError::VariableAlreadyExists(
                self.variable_name.clone(),
            )));
        }

        frame.add_variable(self.variable_name.clone(), json);

        Ok(())
    }
}
