pub mod config;
pub mod fs;
pub mod parse;
pub mod process;
pub mod scripts;

mod error;

#[macro_use]
pub mod functions;

pub use config::Config;
pub use error::*;
pub use fs::Node;
pub use scripts::Scripts;

use crate::fs::{OutputNode, ParsedContents};
use crate::parse::LocatableToken;
use crate::process::error::ProcessError;

use std::fmt::Debug;
use std::path::Path;

define_functions![
    functions::parsers::Begin,
    functions::parsers::DateFormat,
    functions::parsers::End,
    functions::parsers::Excerpt,
    functions::parsers::For,
    functions::parsers::IfDefined,
    functions::parsers::Insert,
    functions::parsers::TimeToRead,
];

#[derive(Debug)]
pub struct Stuart {
    fs: Node,
    stack: Vec<usize>,
    config: Config,
}

#[derive(Clone, Copy, Debug)]
pub struct SpecialFiles<'a> {
    pub root: Option<&'a [LocatableToken]>,
    pub md: Option<&'a [LocatableToken]>,
}

impl Stuart {
    pub fn new(fs: Node, config: Config) -> Self {
        Self {
            fs,
            stack: Vec::new(),
            config,
        }
    }

    pub fn build(&mut self, path: impl AsRef<Path>) -> Result<(), TracebackError<ProcessError>> {
        let specials = SpecialFiles {
            md: None,
            root: None,
        }
        .update_from_children(self.fs.children().unwrap());

        let root = self.build_node(&self.fs, specials)?;

        root.save(&path, &self.config).map_err(|e| TracebackError {
            path: path.as_ref().to_path_buf(),
            line: 0,
            column: 0,
            kind: ProcessError::Save(e),
        })?;

        Ok(())
    }

    fn build_node(
        &self,
        node: &Node,
        specials: SpecialFiles,
    ) -> Result<OutputNode, TracebackError<ProcessError>> {
        match node {
            Node::Directory { name, children, .. } => {
                let specials = specials.update_from_children(children);
                let children = children
                    .iter()
                    .map(|n| self.build_node(n, specials))
                    .collect::<Result<Vec<_>, TracebackError<ProcessError>>>()?;

                Ok(OutputNode::Directory {
                    name: name.clone(),
                    children,
                })
            }
            Node::File { name, contents, .. } => {
                if name != "root.html" && name != "md.html" {
                    let (new_contents, new_name) = node.process(self, specials)?;

                    Ok(OutputNode::File {
                        name: new_name.unwrap_or_else(|| name.clone()),
                        contents: new_contents.unwrap_or_else(|| contents.clone()),
                    })
                } else {
                    Ok(OutputNode::File {
                        name: name.clone(),
                        contents: contents.clone(),
                    })
                }
            }
        }
    }
}

impl<'a> SpecialFiles<'a> {
    fn update_from_children(&self, children: &'a [Node]) -> SpecialFiles {
        let mut specials = *self;

        for child in children {
            if child.name() == "root.html" {
                specials.root = match child.parsed_contents() {
                    ParsedContents::Html(tokens) => Some(tokens),
                    _ => None,
                };
            } else if child.name() == "md.html" {
                specials.md = match child.parsed_contents() {
                    ParsedContents::Html(tokens) => Some(tokens),
                    _ => None,
                };
            }
        }

        specials
    }
}
