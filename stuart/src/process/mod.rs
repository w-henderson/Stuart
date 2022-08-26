pub mod error;
pub mod iter;
pub mod stack;

pub use self::error::ProcessError;

use self::error::TracebackError;
use self::iter::TokenIter;
use self::stack::StackFrame;

use crate::fs::{Node, ParsedContents};
use crate::parse::{ParsedMarkdown, Token};
use crate::{SpecialFiles, Stuart};

use humphrey_json::Value;

pub struct Scope<'a> {
    pub tokens: &'a mut TokenIter<'a>,
    pub stack: &'a mut Vec<StackFrame>,
    pub processor: &'a Stuart,
    pub sections: &'a mut Vec<(String, Vec<u8>)>,
}

impl Node {
    pub fn process(
        &self,
        processor: &Stuart,
        special_files: SpecialFiles,
    ) -> Result<(Option<Vec<u8>>, Option<String>), TracebackError<ProcessError>> {
        println!("processing {}", self.name());

        Ok(match self.parsed_contents() {
            ParsedContents::Html(tokens) => (
                Some(self.process_html(tokens, processor, special_files)?),
                None,
            ),
            ParsedContents::Markdown(md) => self.process_markdown(md, processor, special_files)?,
            _ => (None, None),
        })
    }

    fn process_html(
        &self,
        tokens: &[Token],
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
            token.process(&mut scope).map_err(|kind| TracebackError {
                path: self.source().to_path_buf(),
                line: 0,
                column: 0,
                kind,
            })?;
        }

        println!(
            "sections: {:?}",
            scope
                .sections
                .iter()
                .map(|(name, contents)| (name, String::from_utf8(contents.clone()).unwrap()))
                .collect::<Vec<_>>()
        );

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

        let mut token_iter = TokenIter::new(&root);

        scope.stack.push(StackFrame::new("base2"));
        scope.tokens = &mut token_iter;

        while let Some(token) = scope.tokens.next() {
            token.process(&mut scope).map_err(|kind| TracebackError {
                path: self.source().to_path_buf(),
                line: 0,
                column: 0,
                kind,
            })?;
        }

        Ok(stack.pop().unwrap().output)
    }

    fn process_markdown(
        &self,
        md: &ParsedMarkdown,
        processor: &Stuart,
        special_files: SpecialFiles,
    ) -> Result<(Option<Vec<u8>>, Option<String>), TracebackError<ProcessError>> {
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

        let mut token_iter = TokenIter::new(&md_tokens);
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
            token.process(&mut scope).map_err(|kind| TracebackError {
                path: self.source().to_path_buf(),
                line: 0,
                column: 0,
                kind,
            })?;
        }

        for token in &root {
            token.process(&mut scope).map_err(|kind| TracebackError {
                path: self.source().to_path_buf(),
                line: 0,
                column: 0,
                kind,
            })?;
        }

        let new_name = format!("{}.html", self.name().strip_suffix(".md").unwrap());

        Ok((Some(stack.pop().unwrap().output), Some(new_name)))
    }
}

impl Token {
    fn process(&self, scope: &mut Scope) -> Result<(), ProcessError> {
        match self {
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
                        match value {
                            Value::String(s) => {
                                string = Some(s);
                                break;
                            }

                            Value::Null => Err(ProcessError::NullError(variable.to_string())),
                            Value::Bool(_) => Err(ProcessError::InvalidDataType {
                                variable: variable.to_string(),
                                expected: "string".to_string(),
                                found: "bool".to_string(),
                            }),
                            Value::Number(_) => Err(ProcessError::InvalidDataType {
                                variable: variable.to_string(),
                                expected: "string".to_string(),
                                found: "number".to_string(),
                            }),
                            Value::Array(_) => Err(ProcessError::InvalidDataType {
                                variable: variable.to_string(),
                                expected: "string".to_string(),
                                found: "array".to_string(),
                            }),
                            Value::Object(_) => Err(ProcessError::InvalidDataType {
                                variable: variable.to_string(),
                                expected: "string".to_string(),
                                found: "object".to_string(),
                            }),
                        }?;
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
                    return Err(ProcessError::UndefinedVariable(variable_name.to_string()));
                }
            }
        }

        Ok(())
    }
}
