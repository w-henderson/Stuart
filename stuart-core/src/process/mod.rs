//! Provides processing functionality.

pub mod error;
pub mod iter;
pub mod stack;

pub use self::error::ProcessError;

use self::error::TracebackError;
use self::iter::TokenIter;
use self::stack::StackFrame;

use crate::fs::{Node, ParsedContents};
use crate::parse::{LocatableToken, ParsedMarkdown, Token};
use crate::{SpecialFiles, Stuart};

use humphrey_json::Value;

/// Represents the scope of a function execution.
pub struct Scope<'a> {
    /// The token iterator.
    ///
    /// This allows functions to consume more tokens if necessary, as well as peek at their own token.
    /// For example, the `for` function continues consuming tokens until it reaches `end(for)`.
    pub tokens: &'a mut TokenIter<'a>,

    /// The call stack.
    ///
    /// This allows functions to execute other functions, and to control the scope of their variables.
    /// For example, the `for` function's iteration variable is dropped when the function exits.
    pub stack: &'a mut Vec<StackFrame>,

    /// The Stuart instance that is processing the build.
    pub processor: &'a Stuart,

    /// The sections of the file.
    ///
    /// These are started with `begin("section name")` and ended with `end("section name")`.
    /// This should not be manipulated by custom functions.
    pub sections: &'a mut Vec<(String, Vec<u8>)>,
}

/// A tuple of a new body and a new name for a file.
type NodeModifications = (Option<Vec<u8>>, Option<String>);

impl Node {
    /// Processes a node, returning an output node.
    pub fn process(
        &self,
        processor: &Stuart,
        special_files: SpecialFiles,
    ) -> Result<Node, TracebackError<ProcessError>> {
        let (new_contents, new_name) = if self.name() != "root.html" && self.name() != "md.html" {
            match self.parsed_contents() {
                ParsedContents::Html(tokens) => (
                    Some(self.process_html(tokens, processor, special_files)?),
                    None,
                ),
                ParsedContents::Markdown(md) => {
                    self.process_markdown(md, processor, special_files)?
                }
                _ => (None, None),
            }
        } else {
            (None, None)
        };

        Ok(Node::File {
            name: new_name.unwrap_or_else(|| self.name().to_string()),
            contents: new_contents.unwrap_or_else(|| self.contents().unwrap().to_vec()),
            parsed_contents: ParsedContents::None,
            metadata: if processor.config.save_metadata {
                self.parsed_contents().to_json()
            } else {
                None
            },
            source: self.source().to_path_buf(),
        })
    }

    /// Processes an HTML node, returning the processed output.
    fn process_html(
        &self,
        tokens: &[LocatableToken],
        processor: &Stuart,
        special_files: SpecialFiles,
    ) -> Result<Vec<u8>, TracebackError<ProcessError>> {
        let root = special_files.root.ok_or(TracebackError {
            path: self.source().to_path_buf(),
            line: 0,
            column: 0,
            kind: ProcessError::MissingHtmlRoot,
        })?;

        let mut token_iter = TokenIter::new(tokens);
        let mut stack: Vec<StackFrame> = vec![StackFrame::new("base")];
        let mut sections: Vec<(String, Vec<u8>)> = Vec::new();
        let mut scope = Scope {
            tokens: &mut token_iter,
            stack: &mut stack,
            processor,
            sections: &mut sections,
        };

        while let Some(token) = scope.tokens.next() {
            token.process(&mut scope)?;
        }

        if !scope
            .stack
            .pop()
            .map(|frame| frame.name == "base")
            .unwrap_or(false)
        {
            return Err(TracebackError {
                path: self.source().to_path_buf(),
                line: 0,
                column: 0,
                kind: ProcessError::StackError,
            });
        }

        let mut token_iter = TokenIter::new(root);

        scope.stack.push(StackFrame::new("base2"));
        scope.tokens = &mut token_iter;

        while let Some(token) = scope.tokens.next() {
            token.process(&mut scope)?;
        }

        Ok(stack.pop().unwrap().output)
    }

    /// Processes a markdown node, returning the processed output.
    fn process_markdown(
        &self,
        md: &ParsedMarkdown,
        processor: &Stuart,
        special_files: SpecialFiles,
    ) -> Result<NodeModifications, TracebackError<ProcessError>> {
        let root = special_files.root.ok_or(TracebackError {
            path: self.source().to_path_buf(),
            line: 0,
            column: 0,
            kind: ProcessError::MissingHtmlRoot,
        })?;

        let md_tokens = special_files.md.ok_or(TracebackError {
            path: self.source().to_path_buf(),
            line: 0,
            column: 0,
            kind: ProcessError::MissingMarkdownRoot,
        })?;

        let mut token_iter = TokenIter::new(md_tokens);
        let mut stack: Vec<StackFrame> = vec![{
            let mut frame = StackFrame::new("base");
            frame.add_variable("self", md.to_value());
            frame
        }];
        let mut sections: Vec<(String, Vec<u8>)> = Vec::new();
        let mut scope = Scope {
            tokens: &mut token_iter,
            stack: &mut stack,
            processor,
            sections: &mut sections,
        };

        while let Some(token) = scope.tokens.next() {
            token.process(&mut scope)?;
        }

        if !scope
            .stack
            .pop()
            .map(|frame| frame.name == "base")
            .unwrap_or(false)
        {
            return Err(TracebackError {
                path: self.source().to_path_buf(),
                line: 0,
                column: 0,
                kind: ProcessError::StackError,
            });
        }

        let mut token_iter = TokenIter::new(root);

        scope.stack.push(StackFrame::new("base2"));
        scope.tokens = &mut token_iter;

        while let Some(token) = scope.tokens.next() {
            token.process(&mut scope)?;
        }

        let new_name = format!("{}.html", self.name().strip_suffix(".md").unwrap());

        Ok((Some(stack.pop().unwrap().output), Some(new_name)))
    }
}

impl LocatableToken {
    /// Processes a token, updating the scope.
    pub fn process(&self, scope: &mut Scope) -> Result<(), TracebackError<ProcessError>> {
        match &self.inner {
            Token::Raw(raw) => scope
                .stack
                .last_mut()
                .unwrap()
                .output
                .extend_from_slice(raw.as_bytes()),

            Token::Function(function) => function.execute(scope)?,

            Token::Variable(variable) => {
                let mut variable_iter = variable.split('.');
                let variable_name = variable_iter.next().unwrap();
                let variable_indexes = variable_iter.collect::<Vec<_>>();

                let mut string = None;

                for frame in scope.stack.iter().rev() {
                    if let Some(value) = frame
                        .get_variable(variable_name)
                        .map(|v| stack::get_value(&variable_indexes, v))
                    {
                        let e = |found: &str| {
                            Err(ProcessError::InvalidDataType {
                                variable: variable.to_string(),
                                expected: "string".to_string(),
                                found: found.to_string(),
                            })
                        };

                        match value {
                            Value::String(s) => {
                                string = Some(s);
                                break;
                            }

                            Value::Null => Err(ProcessError::NullError(variable.to_string())),
                            Value::Bool(_) => e("bool"),
                            Value::Number(_) => e("number"),
                            Value::Array(_) => e("array"),
                            Value::Object(_) => e("object"),
                        }
                        .map_err(|e| self.traceback(e))?;
                    }
                }

                if let Some(s) = string {
                    scope
                        .stack
                        .last_mut()
                        .unwrap()
                        .output
                        .extend_from_slice(s.as_bytes());
                } else {
                    return Err(
                        self.traceback(ProcessError::UndefinedVariable(variable.to_string()))
                    );
                }
            }
        }

        Ok(())
    }
}

impl<'a> Scope<'a> {
    /// Gets a variable from the scope by looking down the stack.
    pub fn get_variable(&self, name: &str) -> Option<Value> {
        let mut variable_iter = name.split('.');
        let variable_name = variable_iter.next().unwrap();
        let variable_indexes = variable_iter.collect::<Vec<_>>();

        let mut variable = None;

        for frame in self.stack.iter().rev() {
            if let Some(value) = frame
                .get_variable(variable_name)
                .map(|v| crate::process::stack::get_value(&variable_indexes, v))
            {
                variable = Some(value);
                break;
            }
        }

        variable
    }

    /// Adds to the output of the current stack frame.
    pub fn output(&mut self, output: impl AsRef<[u8]>) -> Result<(), ProcessError> {
        self.stack
            .last_mut()
            .ok_or(ProcessError::StackError)?
            .output
            .extend_from_slice(output.as_ref());

        Ok(())
    }
}
