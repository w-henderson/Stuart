//! Stuart: A Blazingly-Fast Static Site Generator.

#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

pub mod config;
pub mod fs;
pub mod parse;
pub mod process;

#[macro_use]
pub mod functions;

#[cfg(test)]
mod tests;

pub use config::Config;
pub use fs::Node;

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

/// The project builder.
#[derive(Debug)]
pub struct Stuart {
    /// The input virtual filesystem tree.
    pub fs: Node,
    /// The output virtual filesystem tree.
    pub out: Option<Node>,
    /// The configuration of the project.
    pub config: Config,
}

/// The nearest "special files" to a given node.
///
/// These are the root HTML file and the root markdown HTML file.
#[derive(Clone, Copy, Debug)]
pub struct SpecialFiles<'a> {
    /// The root HTML file.
    pub root: Option<&'a [LocatableToken]>,
    /// The root markdown HTML file.
    pub md: Option<&'a [LocatableToken]>,
}

/// Encapsulates an error and its location.
#[derive(Clone, Debug)]
pub struct TracebackError<T: Clone + Debug> {
    /// The path of the file in which the error occurred.
    pub path: PathBuf,
    /// The line number at which the error occurred.
    pub line: u32,
    /// The column number at which the error occurred.
    pub column: u32,
    /// The error.
    pub kind: T,
}

impl Stuart {
    /// Creates a new builder from an input virtual filesystem tree and a configuration.
    pub fn new(fs: Node, config: Config) -> Self {
        Self {
            fs,
            out: None,
            config,
        }
    }

    /// Attempts to build the project.
    pub fn build(&mut self) -> Result<(), TracebackError<ProcessError>> {
        let specials = SpecialFiles {
            md: None,
            root: None,
        }
        .update_from_children(self.fs.children().unwrap());

        self.out = Some(self.build_node(&self.fs, specials)?);

        Ok(())
    }

    /// Merges an output node with the built result.
    ///
    /// This is used for merging static content with the build output.
    pub fn merge_output(&mut self, node: Node) -> Result<(), ProcessError> {
        self.out
            .as_mut()
            .ok_or(ProcessError::NotBuilt)
            .and_then(|out| out.merge(node).map_err(ProcessError::Fs))
    }

    /// Saves the build output to a directory.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), ProcessError> {
        if let Some(out) = &self.out {
            out.save(&path, &self.config).map_err(ProcessError::Fs)
        } else {
            Err(ProcessError::NotBuilt)
        }
    }

    /// Saves the build metadata to a file.
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

    /// Recursively builds an input node and its descendants, returning an output node.
    fn build_node(
        &self,
        node: &Node,
        specials: SpecialFiles,
    ) -> Result<Node, TracebackError<ProcessError>> {
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

                Ok(Node::Directory {
                    name: name.clone(),
                    children,
                    source: source.clone(),
                })
            }
            Node::File { .. } => node.process(self, specials),
        }
    }
}

impl<'a> SpecialFiles<'a> {
    /// Updates the special files from a list of children.
    fn update_from_children(&self, children: &'a [Node]) -> SpecialFiles {
        let mut specials = *self;

        for child in children {
            match child.name() {
                "root.html" => {
                    specials.root = match child.parsed_contents() {
                        ParsedContents::Html(tokens) => Some(tokens),
                        _ => None,
                    }
                }
                "md.html" => {
                    specials.md = match child.parsed_contents() {
                        ParsedContents::Html(tokens) => Some(tokens),
                        _ => None,
                    }
                }
                _ => (),
            }
        }

        specials
    }
}
