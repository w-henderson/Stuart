pub mod config;
pub mod fs;
pub mod parse;
pub mod process;

#[macro_use]
pub mod functions;

pub use config::Config;
pub use fs::{Node, OutputNode};

use crate::fs::ParsedContents;
use crate::parse::LocatableToken;
use crate::process::error::ProcessError;

use humphrey_json::prelude::*;

use std::fmt::Debug;
use std::path::{Path, PathBuf};

define_functions![
    functions::parsers::Begin,
    functions::parsers::DateFormat,
    functions::parsers::End,
    functions::parsers::Excerpt,
    functions::parsers::For,
    functions::parsers::IfDefined,
    functions::parsers::Import,
    functions::parsers::Insert,
    functions::parsers::TimeToRead,
];

#[derive(Debug)]
pub struct Stuart {
    pub fs: Node,
    pub out: Option<OutputNode>,
    pub config: Config,
}

#[derive(Clone, Copy, Debug)]
pub struct SpecialFiles<'a> {
    pub root: Option<&'a [LocatableToken]>,
    pub md: Option<&'a [LocatableToken]>,
}

#[derive(Clone, Debug)]
pub struct TracebackError<T: Clone + Debug> {
    pub path: PathBuf,
    pub line: u32,
    pub column: u32,
    pub kind: T,
}

impl Stuart {
    pub fn new(fs: Node, config: Config) -> Self {
        Self {
            fs,
            out: None,
            config,
        }
    }

    pub fn build(&mut self) -> Result<(), TracebackError<ProcessError>> {
        let specials = SpecialFiles {
            md: None,
            root: None,
        }
        .update_from_children(self.fs.children().unwrap());

        self.out = Some(self.build_node(&self.fs, specials)?);

        Ok(())
    }

    pub fn merge_output(&mut self, node: OutputNode) -> Result<(), ProcessError> {
        if let Some(out) = &mut self.out {
            out.merge(node).map_err(ProcessError::Fs)
        } else {
            Err(ProcessError::NotBuilt)
        }
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), ProcessError> {
        if let Some(out) = &self.out {
            out.save(&path, &self.config).map_err(ProcessError::Fs)
        } else {
            Err(ProcessError::NotBuilt)
        }
    }

    pub fn save_metadata(&self, path: impl AsRef<Path>) -> Result<(), ProcessError> {
        if !self.config.save_metadata {
            return Err(ProcessError::MetadataNotEnabled);
        }

        if let Some(out) = &self.out {
            let base = json!({
                "name": (self.config.name.clone()),
                "author": (self.config.author.clone())
            });

            out.save_metadata(base, &path).map_err(ProcessError::Fs)
        } else {
            Err(ProcessError::NotBuilt)
        }
    }

    fn build_node(
        &self,
        node: &Node,
        specials: SpecialFiles,
    ) -> Result<OutputNode, TracebackError<ProcessError>> {
        match node {
            Node::Directory {
                name,
                children,
                source,
            } => {
                let specials = specials.update_from_children(children);
                let children = children
                    .iter()
                    .map(|n| self.build_node(n, specials))
                    .collect::<Result<Vec<_>, TracebackError<ProcessError>>>()?;

                Ok(OutputNode::Directory {
                    name: name.clone(),
                    children,
                    source: source.clone(),
                })
            }
            Node::File {
                name,
                contents,
                source,
                parsed_contents,
            } => {
                if name != "root.html" && name != "md.html" {
                    let (new_contents, new_name) = node.process(self, specials)?;

                    Ok(OutputNode::File {
                        name: new_name.unwrap_or_else(|| name.clone()),
                        contents: new_contents.unwrap_or_else(|| contents.clone()),
                        source: source.clone(),
                        json: if self.config.save_metadata {
                            parsed_contents.to_json()
                        } else {
                            None
                        },
                    })
                } else {
                    Ok(OutputNode::File {
                        name: name.clone(),
                        contents: contents.clone(),
                        source: source.clone(),
                        json: None,
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
