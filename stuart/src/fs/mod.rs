use crate::parse::{parse, parse_markdown, ParseError, ParsedMarkdown, Token, TracebackError};

use humphrey_json::Value;

use std::fmt::Debug;
use std::fs::{create_dir, read, read_dir, remove_dir_all, write};
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub enum Node {
    File {
        name: String,
        contents: Vec<u8>,
        parsed_contents: ParsedContents,
    },
    Directory {
        name: String,
        children: Vec<Node>,
    },
}

#[derive(Clone, Debug)]
pub enum Error {
    Path,
    NotFound,
    Read,
    Write,
    Parse(PathBuf, TracebackError),
}

#[derive(Clone, Debug)]
pub enum ParsedContents {
    Html(Vec<Token>),
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
            .map_err(|_| Error::NotFound)?;

        let content_path = root.join("content");
        let content_dir = Self::create_from_dir(&content_path)?;

        Ok(content_dir)
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

    pub fn children_mut(&mut self) -> Option<&mut [Node]> {
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

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        let path = path.as_ref().to_path_buf();

        if path.exists() && path.is_dir() {
            remove_dir_all(&path).map_err(|_| Error::Write)?;
        }

        match self {
            Self::Directory { name: _, children } => {
                create_dir(&path).map_err(|_| Error::Write)?;

                for child in children {
                    child.save_recur(&path)?;
                }
            }
            _ => panic!("`Node::save` should only be used on the root directory"),
        }

        Ok(())
    }

    fn save_recur(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        let path = path.as_ref().to_path_buf();

        match self {
            Self::Directory { name, children } => {
                let dir = path.join(name);

                create_dir(&dir).map_err(|_| Error::Write)?;

                for child in children {
                    child.save_recur(&dir)?;
                }
            }
            Self::File { name, contents, .. } => {
                if name != "root.html" && name != "md.html" {
                    write(path.join(name), contents).map_err(|_| Error::Write)?;
                }
            }
        }

        Ok(())
    }

    fn create_from_dir(dir: impl AsRef<Path>) -> Result<Self, Error> {
        let dir = dir.as_ref();
        let content = read_dir(&dir).map_err(|_| Error::NotFound)?;
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
        })
    }

    fn create_from_file(file: impl AsRef<Path>) -> Result<Self, Error> {
        let file = file.as_ref();
        let name = file.file_name().unwrap().to_string_lossy().to_string();
        let contents = read(&file).map_err(|_| Error::Read)?;
        let extension = file.extension().map(|e| e.to_string_lossy().to_string());
        let contents_string = std::str::from_utf8(&contents).map_err(|_| Error::Read);

        let parsed_contents = match extension.as_deref() {
            Some("html") => ParsedContents::Html(
                parse(contents_string?).map_err(|e| Error::Parse(file.to_path_buf(), e))?,
            ),
            Some("md") => ParsedContents::Markdown(
                parse_markdown(contents_string?.to_string())
                    .map_err(|e| Error::Parse(file.to_path_buf(), e))?,
            ),
            Some("json") => {
                ParsedContents::Json(humphrey_json::from_str(contents_string?).map_err(|_| {
                    Error::Parse(
                        file.to_path_buf(),
                        TracebackError {
                            kind: ParseError::InvalidJson,
                            column: 0,
                            line: 0,
                        },
                    )
                })?)
            }
            _ => ParsedContents::None,
        };

        Ok(Node::File {
            name,
            contents,
            parsed_contents,
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
            } => f
                .debug_struct("File")
                .field("name", name)
                .field("contents", &format!("{} bytes", contents.len()))
                .field("parsed_contents", parsed_contents)
                .finish(),
            Self::Directory { name, children } => f
                .debug_struct("Directory")
                .field("name", name)
                .field("children", children)
                .finish(),
        }
    }
}

impl ParsedContents {
    pub fn tokens(&self) -> Option<&[Token]> {
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
}
