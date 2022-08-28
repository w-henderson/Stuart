use crate::functions::is_ident;
use crate::parse::ParseError;

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
                return Err(ParseError::InvalidVariableName(variable_name.to_string()));
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

            println!("invalid arg");

            Err(ParseError::GenericSyntaxError)
        }
    }

    pub fn as_variable(&self) -> Option<&str> {
        match self {
            Self::Variable(variable_name) => Some(variable_name),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(string) => Some(string),
            _ => None,
        }
    }

    pub fn as_ident(&self) -> Option<&str> {
        match self {
            Self::Ident(ident) => Some(ident),
            _ => None,
        }
    }

    pub fn as_integer(&self) -> Option<i32> {
        match self {
            Self::Integer(int) => Some(*int),
            _ => None,
        }
    }
}
