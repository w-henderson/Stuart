pub mod error;
pub mod variable;

pub use self::error::ProcessError;

use self::error::TracebackError;
use self::variable::StackFrame;

use crate::fs::{Node, ParsedContents};
use crate::parse::{ParsedMarkdown, Token};
use crate::{SpecialFiles, Stuart};

use humphrey_json::Value;

pub struct Scope<'a> {
    stack: &'a mut Vec<StackFrame>,
    output: &'a mut Vec<u8>,
    processor: &'a Stuart,
    queue: &'a mut [&'a [Token]],
}

impl Node {
    pub fn process(
        &self,
        processor: &Stuart,
        special_files: SpecialFiles,
    ) -> Result<Option<Vec<u8>>, TracebackError<ProcessError>> {
        Ok(match self.parsed_contents() {
            ParsedContents::Html(tokens) => {
                Some(self.process_html(tokens, processor, special_files)?)
            }
            ParsedContents::Markdown(md) => {
                Some(self.process_markdown(md, processor, special_files)?)
            }
            _ => None,
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

        let mut output: Vec<u8> = Vec::new();
        let mut stack: Vec<StackFrame> = Vec::new();
        let mut queue: Vec<&[Token]> = vec![tokens];
        let mut scope = Scope {
            stack: &mut stack,
            output: &mut output,
            processor,
            queue: &mut queue,
        };

        Node::process_tokens(&root, &mut scope).map_err(|kind| TracebackError {
            path: self.source().to_path_buf(),
            line: 0,
            column: 0,
            kind,
        })?;

        Ok(output)
    }

    fn process_markdown(
        &self,
        md: &ParsedMarkdown,
        processor: &Stuart,
        special_files: SpecialFiles,
    ) -> Result<Vec<u8>, TracebackError<ProcessError>> {
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

        let mut output: Vec<u8> = Vec::new();
        let mut stack: Vec<StackFrame> = vec![StackFrame {
            variables: vec![("self".to_string(), md.to_value())],
        }];
        let mut queue: Vec<&[Token]> = vec![&md_tokens];
        let mut scope = Scope {
            stack: &mut stack,
            output: &mut output,
            processor,
            queue: &mut queue,
        };

        Node::process_tokens(&root, &mut scope).map_err(|kind| TracebackError {
            path: self.source().to_path_buf(),
            line: 0,
            column: 0,
            kind,
        })?;

        Ok(output)
    }

    fn process_tokens(tokens: &[Token], scope: &mut Scope) -> Result<(), ProcessError> {
        let stack_depth = scope.stack.len();

        for token in tokens {
            match token {
                Token::Raw(raw) => scope.output.extend_from_slice(raw.as_bytes()),
                Token::Function(function) => function.execute(scope)?,
                Token::Variable(variable) => {
                    let mut variable_iter = variable.split('.');
                    let variable_name = variable_iter.next().unwrap();
                    let variable_indexes = variable_iter.collect::<Vec<_>>();

                    for frame in scope.stack.iter().rev() {
                        if let Some(value) = frame
                            .get_variable(variable_name)
                            .map(|v| variable::get_value(&variable_indexes, v))
                        {
                            match value {
                                Value::String(s) => {
                                    scope.output.extend_from_slice(s.as_bytes());
                                    Ok(())
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

                            continue;
                        }
                    }

                    return Err(ProcessError::UndefinedVariable(variable_name.to_string()));
                }
            }
        }

        if stack_depth != scope.stack.len() {
            return Err(ProcessError::StackError);
        }

        Ok(())
    }
}
