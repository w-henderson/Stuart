mod error;
mod function;
mod markdown;
mod parser;

use crate::functions::Function;

pub use self::error::{ParseError, TracebackError};
pub use self::function::{RawArgument, RawFunction};
pub use self::markdown::{parse_markdown, ParsedMarkdown};
pub use self::parser::Parser;

use std::path::Path;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub enum Token {
    Raw(String),
    Function(Rc<Box<dyn Function>>),
    Variable(String),
}

impl Token {
    pub fn as_raw(&self) -> Option<&str> {
        match self {
            Token::Raw(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_function(&self) -> Option<Rc<Box<dyn Function>>> {
        match self {
            Token::Function(f) => Some(f.clone()),
            _ => None,
        }
    }

    pub fn as_variable(&self) -> Option<&str> {
        match self {
            Token::Variable(s) => Some(s.as_str()),
            _ => None,
        }
    }
}

pub fn parse(input: &str, path: &Path) -> Result<Vec<Token>, TracebackError<ParseError>> {
    let chars = input.chars();
    let mut parser = Parser::new(chars, path);
    let mut tokens = Vec::new();

    while let Some(raw) = parser.extract_until("{{") {
        if !raw.is_empty() {
            tokens.push(Token::Raw(raw));
        }

        parser.ignore_while(|c| c.is_whitespace());

        let token = match parser.peek() {
            Some('$') => parse_variable(&mut parser)?,
            Some(_) => parse_function(&mut parser)?,
            None => return Err(parser.traceback(ParseError::UnexpectedEOF)),
        };

        tokens.push(token);

        parser.ignore_while(|c| c.is_whitespace());
        parser.expect("}}")?;
    }

    let remaining = parser.extract_remaining();
    if !remaining.is_empty() {
        tokens.push(Token::Raw(remaining));
    }

    Ok(tokens)
}

fn parse_variable(parser: &mut Parser) -> Result<Token, TracebackError<ParseError>> {
    parser.expect("$")?;

    let variable_name = parser.extract_while(|c| c.is_alphanumeric() || c == '_' || c == '.');

    if variable_name.is_empty() {
        return Err(parser.traceback(ParseError::InvalidVariableName("<empty>".to_string())));
    }

    Ok(Token::Variable(variable_name))
}

fn parse_function(parser: &mut Parser) -> Result<Token, TracebackError<ParseError>> {
    let function_name = parser.extract_while(|c| c.is_alphanumeric() || c == '_');

    if function_name.is_empty() {
        return Err(parser.traceback(ParseError::InvalidFunctionName("<empty>".to_string())));
    }

    parser.ignore_while(|c| c.is_whitespace());
    parser.expect("(")?;

    let mut positional_args: Vec<RawArgument> = Vec::new();
    let mut named_args: Vec<(String, RawArgument)> = Vec::new();

    loop {
        parser.ignore_while(|c| c.is_whitespace());

        let mut open_quote = false;
        let arg = parser
            .extract_while(|c| {
                if c == '"' {
                    open_quote = !open_quote;
                }

                open_quote || (c != ')' && c != ',')
            })
            .trim()
            .to_string();

        if arg.contains('=') {
            // Parse a named argument.

            // Extract the name and value.
            let mut parts = arg.splitn(2, '=');
            let name = parts
                .next()
                .ok_or_else(|| parser.traceback(ParseError::GenericSyntaxError))?;
            let value = parts
                .next()
                .ok_or_else(|| parser.traceback(ParseError::GenericSyntaxError))?;

            // Verify that the name and value are probably valid.
            if name.is_empty()
                || value.is_empty()
                || !name.chars().all(|c| c.is_alphanumeric() || c == '_')
            {
                return Err(parser.traceback(ParseError::GenericSyntaxError));
            }

            // Parse the value.
            let argument = RawArgument::parse(value).map_err(|e| parser.traceback(e))?;
            named_args.push((name.to_string(), argument));
        } else {
            // Ensure that there are no positional arguments after any named arguments.
            if !named_args.is_empty() {
                return Err(parser.traceback(ParseError::PositionalArgAfterNamedArg));
            }

            // Parse the value.
            let argument = RawArgument::parse(&arg).map_err(|e| parser.traceback(e))?;
            positional_args.push(argument);
        }

        match parser.next()? {
            ')' => break,
            ',' => continue,
            _ => unreachable!(),
        }
    }

    parser.ignore_while(|c| c.is_whitespace());

    let raw_function = RawFunction {
        name: function_name.clone(),
        positional_args,
        named_args,
    };

    for function_parser in &*crate::FUNCTION_PARSERS {
        if function_parser.can_parse(&raw_function) {
            return Ok(Token::Function(Rc::new(
                function_parser
                    .parse(raw_function)
                    .map_err(|e| parser.traceback(e))?,
            )));
        }
    }

    Err(parser.traceback(ParseError::NonexistentFunction(function_name.to_string())))
}
