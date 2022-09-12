//! Stuart: A Blazingly-Fast Static Site Generator.

#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

pub mod config;
pub mod error;
pub mod fs;
pub mod parse;
pub mod plugins;
pub mod process;

#[macro_use]
pub mod functions;

#[cfg(test)]
mod tests;

pub use config::Config;
pub use error::{Error, TracebackError};
pub use fs::Node;

use crate::error::ProcessError;
use crate::fs::ParsedContents;
use crate::parse::LocatableToken;
use crate::plugins::Manager;
use crate::process::stack::StackFrame;

use humphrey_json::{prelude::*, Value};

use std::fmt::Debug;
use std::path::{Path, PathBuf};

define_functions![
    functions::parsers::Begin,
    functions::parsers::DateFormat,
    functions::parsers::Else,
    functions::parsers::End,
    functions::parsers::Excerpt,
    functions::parsers::For,
    functions::parsers::IfDefined,
    functions::parsers::Import,
    functions::parsers::Insert,
    functions::parsers::TimeToRead,
    functions::parsers::IfEq,
    functions::parsers::IfNe,
    functions::parsers::IfGt,
    functions::parsers::IfGe,
    functions::parsers::IfLt,
    functions::parsers::IfLe,
];

/// The project builder.
pub struct Stuart {
    /// The input directory.
    pub dir: PathBuf,
    /// The input virtual filesystem tree.
    pub input: Option<Node>,
    /// The output virtual filesystem tree.
    pub output: Option<Node>,
    /// The configuration of the project.
    pub config: Config,
    /// The base stack frame for each node.
    pub base: Option<StackFrame>,
    pub plugins: Option<Box<dyn Manager>>,
}

/// The environment of the build.
#[derive(Copy, Clone, Debug)]
pub struct Environment<'a> {
    /// The environment variables.
    pub vars: &'a [(String, String)],
    /// The root HTML file.
    pub root: Option<&'a [LocatableToken]>,
    /// The root markdown HTML file.
    pub md: Option<&'a [LocatableToken]>,
}

impl Stuart {
    /// Creates a new builder from an input directory.
    pub fn new(dir: impl AsRef<Path>) -> Self {
        Self {
            dir: dir.as_ref().to_path_buf(),
            input: None,
            output: None,
            config: Config::default(),
            base: None,
            plugins: None,
        }
    }

    pub fn new_from_node(node: Node) -> Self {
        Self {
            dir: node.source().to_path_buf(),
            input: Some(node),
            output: None,
            config: Config::default(),
            base: None,
            plugins: None,
        }
    }

    pub fn with_config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }

    pub fn with_plugins<T>(mut self, plugins: T) -> Self
    where
        T: Manager + 'static,
    {
        self.plugins = Some(Box::new(plugins));
        self
    }

    /// Attempts to build the project.
    pub fn build(&mut self, stuart_env: String) -> Result<(), Error> {
        if self.input.is_none() {
            self.input = Some(match self.plugins {
                Some(ref plugins) => Node::new_with_plugins(&self.dir, true, plugins.as_ref())?,
                None => Node::new(&self.dir, true)?,
            });
        }

        let vars = {
            let mut env = std::env::vars().collect::<Vec<_>>();
            env.push(("STUART_ENV".into(), stuart_env));
            env
        };

        let env = Environment {
            vars: &vars,
            md: None,
            root: None,
        }
        .update_from_children(self.input.as_ref().unwrap().children().unwrap());

        let base = StackFrame::new("base").with_variable(
            "env",
            Value::Object(
                vars.iter()
                    .map(|(k, v)| (k.clone(), Value::String(v.clone())))
                    .collect(),
            ),
        );

        self.base = Some(base);
        self.output = Some(
            self.build_node(self.input.as_ref().unwrap(), env)
                .map_err(Error::Process)?,
        );

        Ok(())
    }

    /// Merges an output node with the built result.
    ///
    /// This is used for merging static content with the build output.
    pub fn merge_output(&mut self, node: Node) -> Result<(), Error> {
        self.output
            .as_mut()
            .ok_or(Error::NotBuilt)
            .and_then(|out| out.merge(node))
    }

    /// Saves the build output to a directory.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        if let Some(out) = &self.output {
            out.save(&path, &self.config)
        } else {
            Err(Error::NotBuilt)
        }
    }

    /// Saves the build metadata to a file.
    pub fn save_metadata(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        if !self.config.save_metadata {
            return Err(Error::MetadataNotEnabled);
        }

        if let Some(out) = &self.output {
            let base = json!({
                "name": (self.config.name.clone()),
                "author": (self.config.author.clone())
            });

            out.save_metadata(base, &path)
        } else {
            Err(Error::NotBuilt)
        }
    }

    /// Recursively builds an input node and its descendants, returning an output node.
    fn build_node(
        &self,
        node: &Node,
        env: Environment,
    ) -> Result<Node, TracebackError<ProcessError>> {
        match node {
            Node::Directory {
                name,
                children,
                source,
            } => {
                let env = env.update_from_children(children);
                let children = children
                    .iter()
                    .map(|n| self.build_node(n, env))
                    .collect::<Result<Vec<_>, TracebackError<ProcessError>>>()?;

                Ok(Node::Directory {
                    name: name.clone(),
                    children,
                    source: source.clone(),
                })
            }
            Node::File { .. } => node.process(self, env),
        }
    }
}

impl<'a> Environment<'a> {
    /// Updates the environment from a list of children, adding the closest root HTML files.
    fn update_from_children(&self, children: &'a [Node]) -> Self {
        let mut env = *self;

        for child in children {
            match child.name() {
                "root.html" => {
                    env.root = match child.parsed_contents() {
                        ParsedContents::Html(tokens) => Some(tokens),
                        _ => None,
                    }
                }
                "md.html" => {
                    env.md = match child.parsed_contents() {
                        ParsedContents::Html(tokens) => Some(tokens),
                        _ => None,
                    }
                }
                _ => (),
            }
        }

        env
    }
}
