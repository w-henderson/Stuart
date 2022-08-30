mod output;

pub use output::OutputNode;

use crate::parse::{
    parse, parse_markdown, LocatableToken, ParseError, ParsedMarkdown, TracebackError,
};

use humphrey_json::prelude::*;
use humphrey_json::Value;

use std::fmt::Debug;
use std::fs::{read, read_dir};
use std::path::{Component, Path, PathBuf};

#[derive(Clone)]
pub enum Node {
    File {
        name: String,
        contents: Vec<u8>,
        parsed_contents: ParsedContents,
        source: PathBuf,
    },
    Directory {
        name: String,
        children: Vec<Node>,
        source: PathBuf,
    },
}

#[derive(Clone, Debug)]
pub enum Error {
    NotFound(String),
    Read,
    Write,
    Parse(TracebackError<ParseError>),
    Conflict(PathBuf, PathBuf),
}

#[derive(Clone, Debug)]
pub enum ParsedContents {
    Html(Vec<LocatableToken>),
    Markdown(ParsedMarkdown),
    Json(Value),
    None,
}

impl Node {
    pub fn new(root: impl AsRef<Path>) -> Result<Self, Error> {
        let root = root
            .as_ref()
            .to_path_buf()
            .canonicalize()
            .map_err(|_| Error::NotFound(root.as_ref().to_string_lossy().to_string()))?;

        Self::create_from_dir(root)
    }

    pub fn is_dir(&self) -> bool {
        matches!(self, Node::Directory { .. })
    }

    pub fn is_file(&self) -> bool {
        matches!(self, Node::File { .. })
    }

    pub fn name(&self) -> &str {
        match self {
            Node::File { name, .. } => name,
            Node::Directory { name, .. } => name,
        }
    }

    pub fn children(&self) -> Option<&[Node]> {
        match self {
            Node::Directory { children, .. } => Some(children),
            Node::File { .. } => None,
        }
    }

    pub fn contents(&self) -> Option<&[u8]> {
        match self {
            Node::File { contents, .. } => Some(contents),
            Node::Directory { .. } => None,
        }
    }

    pub fn parsed_contents(&self) -> &ParsedContents {
        match self {
            Node::File {
                parsed_contents, ..
            } => parsed_contents,
            Node::Directory { .. } => &ParsedContents::None,
        }
    }

    pub fn source(&self) -> &Path {
        match self {
            Node::File { source, .. } => source,
            Node::Directory { source, .. } => source,
        }
    }

    pub fn get_at_path(&self, path: &Path) -> Option<&Self> {
        let mut working_path = vec![self];

        for part in path.components() {
            match part {
                Component::Normal(name) => {
                    working_path.push(
                        working_path
                            .last()
                            .and_then(|n| n.children())
                            .and_then(|children| children.iter().find(|n| n.name() == name))?,
                    );
                }
                Component::CurDir => (),
                _ => return None,
            }
        }

        working_path.last().copied()
    }

    fn create_from_dir(dir: impl AsRef<Path>) -> Result<Self, Error> {
        let dir = dir.as_ref();
        let content =
            read_dir(&dir).map_err(|_| Error::NotFound(dir.to_string_lossy().to_string()))?;
        let mut children = Vec::new();

        for path in content.flatten() {
            let path = path.path();

            if path.is_dir() {
                let dir = Self::create_from_dir(path)?;
                children.push(dir);
            } else if path.is_file() {
                let file = Self::create_from_file(path)?;
                children.push(file);
            }
        }

        Ok(Node::Directory {
            name: dir.file_name().unwrap().to_string_lossy().to_string(),
            children,
            source: dir.to_path_buf(),
        })
    }

    fn create_from_file(file: impl AsRef<Path>) -> Result<Self, Error> {
        let file = file.as_ref();
        let name = file.file_name().unwrap().to_string_lossy().to_string();
        let contents = read(&file).map_err(|_| Error::Read)?;
        let extension = file.extension().map(|e| e.to_string_lossy().to_string());
        let contents_string = std::str::from_utf8(&contents).map_err(|_| Error::Read);

        let parsed_contents = match extension.as_deref() {
            Some("html") => {
                ParsedContents::Html(parse(contents_string?, file).map_err(Error::Parse)?)
            }
            Some("md") => ParsedContents::Markdown(
                parse_markdown(contents_string?.to_string(), file).map_err(Error::Parse)?,
            ),
            Some("json") => {
                ParsedContents::Json(humphrey_json::from_str(contents_string?).map_err(|_| {
                    Error::Parse(TracebackError {
                        path: file.to_path_buf(),
                        kind: ParseError::InvalidJson,
                        column: 0,
                        line: 0,
                    })
                })?)
            }
            _ => ParsedContents::None,
        };

        Ok(Node::File {
            name,
            contents,
            parsed_contents,
            source: file.to_path_buf(),
        })
    }
}

impl Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::File {
                name,
                contents,
                parsed_contents,
                source,
            } => f
                .debug_struct("File")
                .field("name", name)
                .field("contents", &format!("{} bytes", contents.len()))
                .field("parsed_contents", parsed_contents)
                .field("source", source)
                .finish(),
            Self::Directory {
                name,
                children,
                source,
            } => f
                .debug_struct("Directory")
                .field("name", name)
                .field("children", children)
                .field("source", source)
                .finish(),
        }
    }
}

impl ParsedContents {
    pub fn tokens(&self) -> Option<&[LocatableToken]> {
        match self {
            Self::Html(tokens) => Some(tokens),
            _ => None,
        }
    }

    pub fn markdown(&self) -> Option<&ParsedMarkdown> {
        match self {
            Self::Markdown(markdown) => Some(markdown),
            _ => None,
        }
    }

    pub fn to_json(&self) -> Option<Value> {
        match self {
            ParsedContents::Html(_) => None,
            ParsedContents::None => None,

            ParsedContents::Markdown(md) => Some(json!({
                "type": "markdown",
                "value": (md.to_json())
            })),

            ParsedContents::Json(v) => Some(json!({
                "type": "json",
                "value": (v.clone())
            })),
        }
    }
}
