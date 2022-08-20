use super::error::ParseError;

#[derive(Debug)]
pub enum Function {
    Begin {
        label: String,
        custom: bool,
    },
    End {
        label: String,
        custom: bool,
    },
    For {
        variable_name: String,
        source: String,
        source_type: ForFunctionSourceType,
        limit: Option<u16>,
        sort_variable: Option<String>,
    },
    IfDefined {
        variable_name: String,
    },
    DateFormat {
        variable_name: String,
    },
    TimeToRead {
        variable_name: String,
    },
    Excerpt {
        variable_name: String,
        length: u16,
    },
}

#[derive(Debug)]
pub enum ForFunctionSourceType {
    MarkdownDirectory,
    JSONFile,
    JSONObject,
}

pub struct RawFunction {
    pub(crate) name: String,
    pub(crate) positional_args: Vec<RawArgument>,
    pub(crate) named_args: Vec<(String, RawArgument)>,
}

pub enum RawArgument {
    Variable(String),
    String(String),
    Ident(String),
    Integer(i32),
}

impl RawArgument {
    pub fn parse(arg: &str) -> Result<RawArgument, ParseError> {
        if arg.starts_with('$') {
            // Parse a positional variable argument.

            let variable_name = arg.strip_prefix('$').unwrap();

            if !variable_name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '.')
            {
                return Err(ParseError::InvalidVariableName);
            }

            Ok(Self::Variable(variable_name.to_string()))
        } else if arg.starts_with('"') && arg.ends_with('"') {
            // Parse a positional string argument.

            let string = arg.strip_prefix('"').unwrap().strip_suffix('"').unwrap();

            if string.contains('"') {
                return Err(ParseError::GenericSyntaxError);
            }

            Ok(Self::String(string.to_string()))
        } else if let Ok(int) = arg.parse::<i32>() {
            // Parse an integer argument.

            Ok(Self::Integer(int))
        } else if is_ident(arg) {
            // Parse an identifier argument.

            Ok(Self::Ident(arg.to_string()))
        } else {
            // Invalid positional argument

            Err(ParseError::GenericSyntaxError)
        }
    }

    fn as_variable(&self) -> Option<&str> {
        match self {
            Self::Variable(variable_name) => Some(variable_name),
            _ => None,
        }
    }

    fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(string) => Some(string),
            _ => None,
        }
    }

    fn as_ident(&self) -> Option<&str> {
        match self {
            Self::Ident(ident) => Some(ident),
            _ => None,
        }
    }

    fn as_integer(&self) -> Option<i32> {
        match self {
            Self::Integer(int) => Some(*int),
            _ => None,
        }
    }
}

impl TryFrom<RawFunction> for Function {
    type Error = ParseError;

    fn try_from(value: RawFunction) -> Result<Self, Self::Error> {
        match value.name.as_str() {
            "begin" => {
                quiet_assert(value.positional_args.len() == 1)?;
                quiet_assert(value.named_args.is_empty())?;

                match &value.positional_args[0] {
                    RawArgument::String(label) => Ok(Self::Begin {
                        label: label.to_string(),
                        custom: true,
                    }),
                    RawArgument::Ident(label) => Ok(Self::Begin {
                        label: label.to_string(),
                        custom: false,
                    }),
                    _ => Err(ParseError::InvalidArgument),
                }
            }
            "end" => {
                quiet_assert(value.positional_args.len() == 1)?;
                quiet_assert(value.named_args.is_empty())?;

                match &value.positional_args[0] {
                    RawArgument::String(label) => Ok(Self::End {
                        label: label.to_string(),
                        custom: true,
                    }),
                    RawArgument::Ident(label) => Ok(Self::End {
                        label: label.to_string(),
                        custom: false,
                    }),
                    _ => Err(ParseError::InvalidArgument),
                }
            }
            "for" => {
                quiet_assert(value.positional_args.len() == 2)?;

                let variable_name = value.positional_args[0]
                    .as_variable()
                    .ok_or(ParseError::InvalidArgument)?;

                let (source, is_file) = match &value.positional_args[1] {
                    RawArgument::String(source) => Ok((source.to_string(), true)),
                    RawArgument::Variable(source) => Ok((source.to_string(), false)),
                    _ => return Err(ParseError::InvalidArgument),
                }?;

                let source_type = if is_file {
                    if source.ends_with(".json") {
                        Ok(ForFunctionSourceType::JSONFile)
                    } else if source.ends_with("/") {
                        Ok(ForFunctionSourceType::MarkdownDirectory)
                    } else {
                        Err(ParseError::InvalidArgument)
                    }?
                } else {
                    ForFunctionSourceType::JSONObject
                };

                let mut limit = None;
                let mut sort_variable = None;

                for (name, arg) in &value.named_args {
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

                Ok(Self::For {
                    variable_name: variable_name.to_string(),
                    source: source.to_string(),
                    source_type,
                    limit,
                    sort_variable,
                })
            }
            "ifdefined" => {
                quiet_assert(value.positional_args.len() == 1)?;
                quiet_assert(value.named_args.is_empty())?;

                let variable_name = value.positional_args[0]
                    .as_variable()
                    .ok_or(ParseError::InvalidArgument)?;

                Ok(Self::IfDefined {
                    variable_name: variable_name.to_string(),
                })
            }
            "dateformat" => {
                quiet_assert(value.positional_args.len() == 1)?;
                quiet_assert(value.named_args.is_empty())?;

                let variable_name = value.positional_args[0]
                    .as_variable()
                    .ok_or(ParseError::InvalidArgument)?;

                Ok(Self::DateFormat {
                    variable_name: variable_name.to_string(),
                })
            }
            "timetoread" => {
                quiet_assert(value.positional_args.len() == 1)?;
                quiet_assert(value.named_args.is_empty())?;

                let variable_name = value.positional_args[0]
                    .as_variable()
                    .ok_or(ParseError::InvalidArgument)?;

                Ok(Self::TimeToRead {
                    variable_name: variable_name.to_string(),
                })
            }
            "excerpt" => {
                quiet_assert(value.positional_args.len() == 2)?;
                quiet_assert(value.named_args.is_empty())?;

                let variable_name = value.positional_args[0]
                    .as_variable()
                    .ok_or(ParseError::InvalidArgument)?;

                let length: u16 = value.positional_args[1]
                    .as_integer()
                    .ok_or(ParseError::InvalidArgument)?
                    .try_into()
                    .map_err(|_| ParseError::InvalidArgument)?;

                Ok(Self::Excerpt {
                    variable_name: variable_name.to_string(),
                    length,
                })
            }
            s => Err(ParseError::NonexistentFunction(s.to_string())),
        }
    }
}

#[inline]
fn is_ident(s: &str) -> bool {
    s == "begin"
        || s == "end"
        || s == "for"
        || s == "ifdefined"
        || s == "dateformat"
        || s == "timetoread"
        || s == "excerpt"
}

fn quiet_assert(condition: bool) -> Result<(), ParseError> {
    match condition {
        true => Ok(()),
        false => Err(ParseError::GenericSyntaxError),
    }
}
