use crate::functions::{quiet_assert, Function, FunctionParser};
use crate::parse::{ParseError, RawArgument, RawFunction};

pub struct ForParser;

#[derive(Debug, Clone)]
pub struct ForFunction {
    variable_name: String,
    source: String,
    source_type: ForFunctionSourceType,
    limit: Option<u16>,
    sort_variable: Option<String>,
}

#[derive(Clone, Debug)]
pub enum ForFunctionSourceType {
    MarkdownDirectory,
    JSONFile,
    JSONObject,
}

impl FunctionParser for ForParser {
    fn name(&self) -> &'static str {
        "for"
    }

    fn parse(&self, raw: RawFunction) -> Result<Box<dyn Function>, ParseError> {
        quiet_assert(raw.positional_args.len() == 2)?;

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

        let mut limit = None;
        let mut sort_variable = None;

        for (name, arg) in &raw.named_args {
            match name.as_str() {
                "limit" => {
                    quiet_assert(arg.as_integer().is_some())?;
                    quiet_assert(limit.is_none())?;

                    limit = Some(
                        arg.as_integer()
                            .unwrap()
                            .try_into()
                            .map_err(|_| ParseError::InvalidArgument)?,
                    );
                }
                "sortby" => {
                    quiet_assert(arg.as_variable().is_some())?;
                    quiet_assert(sort_variable.is_none())?;

                    sort_variable = Some(arg.as_variable().unwrap().to_string());
                }
                _ => return Err(ParseError::InvalidArgument),
            }
        }

        Ok(Box::new(ForFunction {
            variable_name: variable_name.to_string(),
            source,
            source_type,
            limit,
            sort_variable,
        }))
    }
}

impl Function for ForFunction {
    fn execute(&self) {
        todo!()
    }
}
