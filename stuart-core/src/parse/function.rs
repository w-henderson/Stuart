//! Provides functionality for parsing raw functions and arguments.

use crate::functions::is_ident;
use crate::parse::ParseError;

/// Represents a raw function.
///
/// A raw function is the result of the first stage of parsing a function. It contains the parsed name of the function,
///   as well as its positional arguments and named arguments as [`RawArgument`]s. The raw function is then further
///   processed into an executable function using the [`FunctionParser`] trait.
pub struct RawFunction {
    /// The name of the function.
    pub name: String,
    /// The positional arguments of the function, in order.
    pub positional_args: Vec<RawArgument>,
    /// The named arguments of the function, in order.
    pub named_args: Vec<(String, RawArgument)>,
}

/// Represents a raw argument.
pub enum RawArgument {
    /// A variable name.
    Variable(String),
    /// A string literal.
    String(String),
    /// An identifier, such as a function name.
    Ident(String),
    /// A number literal. (floats are not yet supported)
    Integer(i32),
}

impl RawArgument {
    /// Attempts to parse a string into a raw argument.
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

            Err(ParseError::GenericSyntaxError)
        }
    }

    /// Returns the argument as a variable name, if it is a variable.
    pub fn as_variable(&self) -> Option<&str> {
        match self {
            Self::Variable(variable_name) => Some(variable_name),
            _ => None,
        }
    }

    /// Returns the argument as a string, if it is a string.
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(string) => Some(string),
            _ => None,
        }
    }

    /// Returns the argument as an identifier, if it is an identifier.
    pub fn as_ident(&self) -> Option<&str> {
        match self {
            Self::Ident(ident) => Some(ident),
            _ => None,
        }
    }

    /// Returns the argument as an integer, if it is an integer.
    pub fn as_integer(&self) -> Option<i32> {
        match self {
            Self::Integer(int) => Some(*int),
            _ => None,
        }
    }
}
