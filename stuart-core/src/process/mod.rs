//! Provides processing functionality.

pub mod iter;
pub mod stack;

pub use crate::error::ProcessError;
use crate::error::TracebackError;

use self::iter::TokenIter;
use self::stack::StackFrame;

use crate::fs::{Node, ParsedContents};
use crate::parse::{LocatableToken, ParsedMarkdown, Token};
use crate::{Environment, Error, Stuart};

use humphrey_json::Value;
use pulldown_cmark::{html, Options, Parser};

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

/// The output of the processing stage.
#[derive(Default)]
pub struct ProcessOutput {
    /// The new contents of the file, if they are to be changed.
    pub new_contents: Option<Vec<u8>>,
    /// The new name of the file, if it is to be changed.
    pub new_name: Option<String>,
}

impl Node {
    /// Processes a node, returning an output node.
    pub fn process(&self, processor: &Stuart, env: Environment) -> Result<Node, Error> {
        let output = if self.name() != "root.html" && self.name() != "md.html" {
            match self.parsed_contents() {
                ParsedContents::Html(tokens) => self
                    .process_html(tokens, processor, env)
                    .map_err(Error::Process)?,
                ParsedContents::Markdown(md) => self
                    .process_markdown(md, processor, env)
                    .map_err(Error::Process)?,
                ParsedContents::Custom(custom) => {
                    custom.process(processor, env).map_err(Error::Plugin)?
                }
                _ => ProcessOutput::default(),
            }
        } else {
            ProcessOutput::default()
        };

        Ok(Node::File {
            name: output.new_name.unwrap_or_else(|| self.name().to_string()),
            contents: output
                .new_contents
                .unwrap_or_else(|| self.contents().unwrap().to_vec()),
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
        env: Environment,
    ) -> Result<ProcessOutput, TracebackError<ProcessError>> {
        let root = env.root.ok_or(TracebackError {
            path: self.source().to_path_buf(),
            line: 0,
            column: 0,
            kind: ProcessError::MissingHtmlRoot,
        })?;

        let mut token_iter = TokenIter::new(tokens);
        let mut stack: Vec<StackFrame> = vec![processor.base.as_ref().unwrap().clone()];
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

        scope.stack.push(processor.base.as_ref().unwrap().clone());
        scope.tokens = &mut token_iter;

        while let Some(token) = scope.tokens.next() {
            token.process(&mut scope)?;
        }

        Ok(ProcessOutput {
            new_contents: Some(stack.pop().unwrap().output),
            new_name: None,
        })
    }

    /// Processes a markdown node, returning the processed output.
    fn process_markdown(
        &self,
        md: &ParsedMarkdown,
        processor: &Stuart,
        env: Environment,
    ) -> Result<ProcessOutput, TracebackError<ProcessError>> {
        let root = env.root.ok_or(TracebackError {
            path: self.source().to_path_buf(),
            line: 0,
            column: 0,
            kind: ProcessError::MissingHtmlRoot,
        })?;

        let md_tokens = env.md.ok_or(TracebackError {
            path: self.source().to_path_buf(),
            line: 0,
            column: 0,
            kind: ProcessError::MissingMarkdownRoot,
        })?;

        let mut token_iter = TokenIter::new(md_tokens);

        let mut stack: Vec<StackFrame> = vec![processor
            .base
            .as_ref()
            .unwrap()
            .clone()
            .with_variable("self", md.to_value())];

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

        scope.stack.push(processor.base.as_ref().unwrap().clone());
        scope.tokens = &mut token_iter;

        while let Some(token) = scope.tokens.next() {
            token.process(&mut scope)?;
        }

        let new_name = format!("{}.html", self.name().strip_suffix(".md").unwrap());

        Ok(ProcessOutput {
            new_contents: Some(stack.pop().unwrap().output),
            new_name: Some(new_name),
        })
    }

    /// Preprocess the markdown node, executing functions within the raw markdown and
    /// converting it to HTML. The implementation of this is currently quite dodgy but
    /// it works for the time being.
    pub(crate) fn preprocess_markdown(
        &mut self,
        processor: &Stuart,
    ) -> Result<(), TracebackError<ProcessError>> {
        let source = self.source().to_path_buf();

        let md = match self.parsed_contents_mut() {
            ParsedContents::Markdown(md) => md,
            _ => return Ok(()),
        };

        let mut token_iter = TokenIter::new(&md.markdown);
        let mut stack: Vec<StackFrame> = vec![processor.base.as_ref().unwrap().clone()];
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

        if let Some(frame) = scope.stack.pop() {
            if frame.name == "base" {
                let processed_markdown =
                    String::from_utf8(frame.output).map_err(|_| TracebackError {
                        path: source.clone(),
                        line: 0,
                        column: 0,
                        kind: ProcessError::StackError,
                    })?;

                let parser = Parser::new_ext(&processed_markdown, Options::all());
                let mut processed_html = String::new();
                html::push_html(&mut processed_html, parser);

                md.html = Some(processed_html);
                return Ok(());
            }
        }

        Err(TracebackError {
            path: self.source().to_path_buf(),
            line: 0,
            column: 0,
            kind: ProcessError::StackError,
        })
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
