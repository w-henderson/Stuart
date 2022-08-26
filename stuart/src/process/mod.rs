pub mod error;
pub mod stack;

pub use self::error::ProcessError;

use self::error::TracebackError;
use self::stack::StackFrame;

use crate::fs::{Node, ParsedContents};
use crate::parse::{ParsedMarkdown, Token};
use crate::{SpecialFiles, Stuart};

use humphrey_json::Value;

pub struct Scope<'a> {
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

        let mut stack: Vec<StackFrame> = vec![StackFrame::new("base")];
        let mut sections: Vec<(String, Vec<u8>)> = Vec::new();
        let mut scope = Scope {
            stack: &mut stack,
            processor,
            sections: &mut sections,
        };

        for token in tokens {
            Node::process_token(token, &mut scope).map_err(|kind| TracebackError {
                path: self.source().to_path_buf(),
                line: 0,
                column: 0,
                kind,
            })?;
        }

        for token in &root {
            Node::process_token(token, &mut scope).map_err(|kind| TracebackError {
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

        let mut stack: Vec<StackFrame> = vec![{
            let mut frame = StackFrame::new("base");
            frame.add_variable("self", md.to_value());
            frame
        }];
        let mut sections: Vec<(String, Vec<u8>)> = Vec::new();
        let mut scope = Scope {
            stack: &mut stack,
            processor,
            sections: &mut sections,
        };

        for token in &md_tokens {
            Node::process_token(token, &mut scope).map_err(|kind| TracebackError {
                path: self.source().to_path_buf(),
                line: 0,
                column: 0,
                kind,
            })?;
        }

        for token in &root {
            Node::process_token(token, &mut scope).map_err(|kind| TracebackError {
                path: self.source().to_path_buf(),
                line: 0,
                column: 0,
                kind,
            })?;
        }

        let new_name = format!("{}.html", self.name().strip_suffix(".md").unwrap());

        Ok((Some(stack.pop().unwrap().output), Some(new_name)))
    }

    fn process_token(token: &Token, scope: &mut Scope) -> Result<(), ProcessError> {
        let stack_depth = scope.stack.len();

        match token {
            Token::Raw(raw) => scope.stack[stack_depth - 1]
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
                    scope.stack[stack_depth - 1]
                        .output
                        .extend_from_slice(s.as_bytes());
                }

                return Err(ProcessError::UndefinedVariable(variable_name.to_string()));
            }
        }

        Ok(())
    }
}
