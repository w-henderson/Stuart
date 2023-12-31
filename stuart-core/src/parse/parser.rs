//! Provides a low-level parser.

use crate::error::{ParseError, TracebackError};

use std::iter::Peekable;
use std::path::Path;
use std::str::Chars;

/// Represents a parser.
///
/// Heavily inspired by [Humphrey JSON's parser](https://github.com/w-henderson/Humphrey/blob/8bf07aada8acb7e25991ac9e9f9462d9fb3086b0/humphrey-json/src/parser.rs#L59).
#[allow(clippy::missing_docs_in_private_items)]
pub struct Parser<'a> {
    chars: Peekable<Chars<'a>>,
    path: &'a Path,
    line: u32,
    column: u32,
    next_line: u32,
    next_column: u32,
}

impl<'a> Parser<'a> {
    /// Creates a new parser for characters at the given path.
    pub fn new(chars: Chars<'a>, path: &'a Path) -> Self {
        Self {
            chars: chars.peekable(),
            path,
            line: 1,
            column: 1,
            next_line: 1,
            next_column: 1,
        }
    }

    /// Generates a traceback error for the current position.
    pub fn traceback(&self, e: ParseError) -> TracebackError<ParseError> {
        TracebackError {
            path: self.path.to_path_buf(),
            line: self.line,
            column: self.column,
            kind: e,
        }
    }

    /// Gets the next character from the parser, returning an error if the end of the input is reached.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Result<char, TracebackError<ParseError>> {
        if let Some(c) = self.chars.next() {
            self.line = self.next_line;
            self.column = self.next_column;

            if c == '\n' {
                self.next_line += 1;
                self.next_column = 0;
            } else if c != '\r' {
                self.next_column += 1;
            }

            Ok(c)
        } else {
            Err(self.traceback(ParseError::UnexpectedEOF))
        }
    }

    /// Peeks at the next character from the parser.
    pub fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    /// Consumes characters from the parser for the length of the input string, and returns an error if they do not match.
    pub fn expect(&mut self, s: &str) -> Result<(), TracebackError<ParseError>> {
        let chars = s.chars();

        for c in chars {
            if c != self.next()? {
                return Err(self.traceback(ParseError::Expected(s.to_string())));
            }
        }

        Ok(())
    }

    /// Extracts a string from the parser until the given string is found.
    ///
    /// The string is not included in the output.
    pub fn extract_until(&mut self, s: &str, allow_escape: bool) -> Option<String> {
        let mut result = String::with_capacity(128);
        let old_chars = self.chars.clone();

        loop {
            if let Ok(c) = self.next() {
                result.push(c);

                if result.ends_with(s) {
                    if allow_escape
                        && result.len() > s.len()
                        && result.as_bytes()[result.len() - s.len() - 1] == b'\\'
                    {
                        result.remove(result.len() - s.len() - 1);
                        continue;
                    }

                    result.truncate(result.len() - s.len());

                    return Some(result);
                }
            } else {
                self.chars = old_chars;

                return None;
            }
        }
    }

    /// Extracts characters from the parser while the predicate returns `true`.
    pub fn extract_while<F>(&mut self, mut f: F) -> String
    where
        F: FnMut(char) -> bool,
    {
        let mut result = String::with_capacity(128);

        while let Some(c) = self.chars.peek() {
            if f(*c) {
                result.push(self.next().unwrap());
            } else {
                break;
            }
        }

        result
    }

    /// Extracts all remaining characters in the parser.
    pub fn extract_remaining(&mut self, allow_escape: bool) -> String {
        let mut result = String::with_capacity(128);

        while let Ok(c) = self.next() {
            result.push(c);
        }

        if allow_escape {
            result = result.replace("\\{{", "{{");
        }

        result
    }

    /// Ignores characters from the parser while the predicate returns `true`.
    pub fn ignore_while<F>(&mut self, f: F)
    where
        F: Fn(char) -> bool,
    {
        while let Some(c) = self.chars.peek() {
            if f(*c) {
                self.next().unwrap();
            } else {
                break;
            }
        }
    }

    /// Returns the current line and column of the parser.
    pub fn location(&self) -> (u32, u32) {
        (self.line, self.column)
    }

    /// Returns the path of the parser.
    pub fn path(&self) -> &Path {
        self.path
    }
}
