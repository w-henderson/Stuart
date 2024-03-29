use crate::fs::ParsedContents;
use crate::functions::{Function, FunctionParser};
use crate::parse::{ParseError, RawArgument, RawFunction};
use crate::process::stack::StackFrame;
use crate::process::{ProcessError, Scope};
use crate::{quiet_assert, TracebackError};

use humphrey_json::Value;

use std::path::PathBuf;

/// Parses the `for` function.
pub struct ForParser;

#[derive(Debug, Clone)]
pub struct ForFunction {
    variable_name: String,
    source: String,
    source_type: ForFunctionSourceType,
    skip: Option<usize>,
    limit: Option<usize>,
    sort_variable: Option<String>,
    sort_order: SortOrder,
}

#[derive(Clone, Copy, Debug)]
pub enum ForFunctionSourceType {
    MarkdownDirectory,
    JSONFile,
    JSONObject,
}

#[derive(Clone, Copy, Debug)]
pub enum SortOrder {
    Asc,
    Desc,
}

impl FunctionParser for ForParser {
    fn name(&self) -> &'static str {
        "for"
    }

    fn parse(&self, raw: RawFunction) -> Result<Box<dyn Function>, ParseError> {
        quiet_assert!(raw.positional_args.len() == 2)?;

        let variable_name = raw.positional_args[0]
            .as_variable()
            .ok_or(ParseError::InvalidArgument)?;

        let (source, is_file) = match &raw.positional_args[1] {
            RawArgument::String(source) => Ok((source.to_string(), true)),
            RawArgument::Variable(source) => Ok((source.to_string(), false)),
            _ => return Err(ParseError::InvalidArgument),
        }?;

        let source_type = if is_file {
            if source.ends_with(".json") {
                Ok(ForFunctionSourceType::JSONFile)
            } else if source.ends_with('/') {
                Ok(ForFunctionSourceType::MarkdownDirectory)
            } else {
                Err(ParseError::InvalidArgument)
            }?
        } else {
            ForFunctionSourceType::JSONObject
        };

        let mut skip = None;
        let mut limit = None;
        let mut sort_variable = None;
        let mut sort_order = SortOrder::Asc;

        for (name, arg) in &raw.named_args {
            match name.as_str() {
                "skip" => {
                    quiet_assert!(arg.as_integer().is_some())?;
                    quiet_assert!(skip.is_none())?;

                    skip = Some(
                        arg.as_integer()
                            .unwrap()
                            .try_into()
                            .map_err(|_| ParseError::InvalidArgument)?,
                    );
                }
                "limit" => {
                    quiet_assert!(arg.as_integer().is_some())?;
                    quiet_assert!(limit.is_none())?;

                    limit = Some(
                        arg.as_integer()
                            .unwrap()
                            .try_into()
                            .map_err(|_| ParseError::InvalidArgument)?,
                    );
                }
                "sortby" => {
                    quiet_assert!(arg.as_variable().is_some())?;
                    quiet_assert!(sort_variable.is_none())?;

                    sort_variable = Some(arg.as_variable().unwrap().to_string());
                }
                "order" => {
                    sort_order = match arg.as_string() {
                        Some("asc") => SortOrder::Asc,
                        Some("desc") => SortOrder::Desc,
                        _ => return Err(ParseError::InvalidArgument),
                    };
                }
                _ => return Err(ParseError::InvalidArgument),
            }
        }

        Ok(Box::new(ForFunction {
            variable_name: variable_name.to_string(),
            source,
            source_type,
            skip,
            limit,
            sort_variable,
            sort_order,
        }))
    }
}

impl Function for ForFunction {
    fn name(&self) -> &'static str {
        "for"
    }

    fn execute(&self, scope: &mut Scope) -> Result<(), TracebackError<ProcessError>> {
        let waypoint = scope.tokens.waypoint();
        let self_token = scope.tokens.current().unwrap().clone();

        let mut variables: Vec<Value> = match self.source_type {
            ForFunctionSourceType::MarkdownDirectory => {
                let directory = scope
                    .processor
                    .input
                    .as_ref()
                    .unwrap()
                    .get_at_path(&PathBuf::from(self.source.clone()))
                    .ok_or_else(|| {
                        self_token.traceback(ProcessError::NotFound(self.source.clone()))
                    })?;

                if !directory.is_dir() {
                    return Err(self_token.traceback(ProcessError::NotFound(self.source.clone())));
                }

                directory
                    .children()
                    .unwrap()
                    .iter()
                    .filter_map(|n| match n.parsed_contents() {
                        ParsedContents::Markdown(md) => Some(md.to_value()),
                        _ => None,
                    })
                    .collect()
            }
            ForFunctionSourceType::JSONFile => {
                let file = scope
                    .processor
                    .input
                    .as_ref()
                    .unwrap()
                    .get_at_path(&PathBuf::from(self.source.clone()))
                    .ok_or_else(|| {
                        self_token.traceback(ProcessError::NotFound(self.source.clone()))
                    })?;

                if !file.is_file() {
                    return Err(self_token.traceback(ProcessError::NotFound(self.source.clone())));
                }

                match file.parsed_contents() {
                    ParsedContents::Json(json) => json.as_array().map(|a| a.iter().cloned()),
                    _ => None,
                }
                .ok_or_else(|| self_token.traceback(ProcessError::NotJsonArray))?
                .collect()
            }
            ForFunctionSourceType::JSONObject => {
                let mut variable_iter = self.source.split('.');
                let variable_name = variable_iter.next().unwrap();
                let variable_indexes = variable_iter.collect::<Vec<_>>();

                let mut variable = None;

                for frame in scope.stack.iter().rev() {
                    if let Some(value) = frame
                        .get_variable(variable_name)
                        .map(|v| crate::process::stack::get_value(&variable_indexes, v))
                    {
                        variable = Some(value);
                        break;
                    }
                }

                variable
                    .and_then(|v| v.as_array().map(|a| a.to_vec()))
                    .ok_or_else(|| self_token.traceback(ProcessError::NotJsonArray))?
            }
        };

        if let Some(key) = &self.sort_variable {
            let indexes = key.split('.').skip(1).collect::<Vec<_>>();

            variables.sort_by_cached_key(|v| {
                crate::process::stack::get_value(&indexes, v)
                    .as_str()
                    .unwrap_or("")
                    .to_string()
            });
        }

        let mut variable_iter: Box<dyn Iterator<Item = Value>> = match self.sort_order {
            SortOrder::Asc => Box::new(variables.into_iter()),
            SortOrder::Desc => Box::new(variables.into_iter().rev()),
        };

        if let Some(s) = self.skip {
            variable_iter = Box::new(variable_iter.skip(s));
        }

        if let Some(l) = self.limit {
            variable_iter = Box::new(variable_iter.take(l));
        }

        for variable in variable_iter {
            scope.tokens.rewind_to(waypoint);

            let frame = {
                let mut frame = StackFrame::new(format!("for:{}", self.variable_name));
                frame.add_variable(&self.variable_name, variable);
                frame
            };

            let stack_height = scope.stack.len();
            scope.stack.push(frame);

            while scope.stack.len() > stack_height {
                let token = scope
                    .tokens
                    .next()
                    .ok_or_else(|| self_token.traceback(ProcessError::UnexpectedEndOfFile))?;
                token.process(scope)?;
            }
        }

        Ok(())
    }
}
