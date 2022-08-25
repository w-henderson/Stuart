pub mod fs;
pub mod parse;
pub mod process;

#[macro_use]
pub mod functions;

use crate::fs::Node;
use crate::parse::Token;

use std::path::{Path, PathBuf};

define_functions![
    functions::parsers::Begin,
    functions::parsers::DateFormat,
    functions::parsers::End,
    functions::parsers::Excerpt,
    functions::parsers::For,
    functions::parsers::IfDefined,
    functions::parsers::Insert,
    functions::parsers::Timestamp,
    functions::parsers::TimeToRead,
];

#[derive(Debug)]
pub struct Stuart {
    fs: Node,
    stack: Vec<usize>,
}

#[derive(Debug)]
pub struct SpecialFiles {
    pub root: Option<Vec<Token>>,
    pub md: Option<Vec<Token>>,
}

#[derive(Clone, Debug)]
pub struct TracebackError<T> {
    pub(crate) path: PathBuf,
    pub(crate) line: u32,
    pub(crate) column: u32,
    pub(crate) kind: T,
}

impl Stuart {
    pub fn new(fs: Node) -> Self {
        Self {
            fs,
            stack: Vec::new(),
        }
    }

    pub fn build(&mut self) {
        loop {
            while self.stack_target().map(|n| n.is_dir()).unwrap_or(false) {
                self.stack.push(0);
            }

            let special_files = self.nearest_special_files();

            match self.stack_target() {
                Some(n) if n.is_file() => {
                    if n.name() != "root.html" && n.name() != "md.html" {
                        n.process(special_files.unwrap());
                    }

                    let index = self.stack.pop().unwrap();
                    self.stack.push(index + 1);
                }
                None => {
                    self.stack.pop();

                    if self.stack.is_empty() {
                        break;
                    } else {
                        let index = self.stack.pop().unwrap();
                        self.stack.push(index + 1);
                    }
                }
                _ => unreachable!(),
            }
        }
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), fs::Error> {
        self.fs.save(path)
    }

    fn stack_target(&mut self) -> Option<&mut Node> {
        let mut n = &mut self.fs;

        for child in &self.stack {
            n = n.children_mut()?.get_mut(*child)?;
        }

        Some(n)
    }

    fn nearest_special_files(&self) -> Option<SpecialFiles> {
        let mut stack = Vec::with_capacity(self.stack.len());
        let mut n = &self.fs;

        for child in &self.stack {
            stack.push(n);
            n = n.children()?.get(*child)?;
        }

        let mut root = None;
        let mut md = None;

        for dir in stack.into_iter().rev() {
            if root.is_none() {
                if let Some(child) = dir.children()?.iter().find(|c| c.name() == "root.html") {
                    root = Some(child);
                }
            }

            if md.is_none() {
                if let Some(child) = dir.children()?.iter().find(|c| c.name() == "md.html") {
                    md = Some(child);
                }
            }

            if root.is_some() && md.is_some() {
                break;
            }
        }

        Some(SpecialFiles {
            root: root
                .map(|n| n.parsed_contents())
                .and_then(|c| c.tokens())
                .map(|t| t.to_vec()),
            md: md
                .map(|n| n.parsed_contents())
                .and_then(|c| c.tokens())
                .map(|t| t.to_vec()),
        })
    }
}
