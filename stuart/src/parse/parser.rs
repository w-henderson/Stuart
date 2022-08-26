use super::error::{ParseError, TracebackError};

use std::iter::Peekable;
use std::path::Path;
use std::str::Chars;

pub struct Parser<'a> {
    chars: Peekable<Chars<'a>>,
    path: &'a Path,
    line: u32,
    column: u32,
    next_line: u32,
    next_column: u32,
}

impl<'a> Parser<'a> {
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

    pub fn traceback(&self, e: ParseError) -> TracebackError<ParseError> {
        TracebackError {
            path: self.path.to_path_buf(),
            line: self.line,
            column: self.column,
            kind: e,
        }
    }

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

    pub fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    pub fn expect(&mut self, s: &str) -> Result<(), TracebackError<ParseError>> {
        let chars = s.chars();

        for c in chars {
            if c != self.next()? {
                return Err(self.traceback(ParseError::Expected(s.to_string())));
            }
        }

        Ok(())
    }

    /// does not include the pattern
    pub fn extract_until(&mut self, s: &str) -> Option<String> {
        let mut result = String::with_capacity(128);
        let old_chars = self.chars.clone();

        loop {
            if let Ok(c) = self.next() {
                result.push(c);

                if result.ends_with(s) {
                    result.truncate(result.len() - s.len());

                    return Some(result);
                }
            } else {
                self.chars = old_chars;

                return None;
            }
        }
    }

    pub fn extract_while<F>(&mut self, f: F) -> String
    where
        F: Fn(char) -> bool,
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

    pub fn extract_remaining(&mut self) -> String {
        let mut result = String::with_capacity(128);

        while let Ok(c) = self.next() {
            result.push(c);
        }

        result
    }

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
}
