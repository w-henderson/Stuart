//! Provides a virtual filesystem tree which parses the files as it is constructed, and saves them according to the configuration.
//!
//! When building a Stuart project, the files are loaded into memory and parsed at the same time, then processed wholly
//!   in memory. They are saved back to disk after processing. In this way, you can think of the entire build process
//!   as simply a function that maps `Node -> Node`. This function is called [`Node::process`].

use crate::parse::{
    parse_html, parse_markdown, LocatableToken, ParseError, ParsedMarkdown, TracebackError,
};
use crate::Config;

use humphrey_json::prelude::*;
use humphrey_json::Value;

use std::fmt::Debug;
use std::fs::{create_dir, metadata, read, read_dir, remove_dir_all, write};
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};

/// Represents a node in the virtual filesystem tree.
#[derive(Clone)]
pub enum Node {
    /// A file in the virtual filesystem tree.
    File {
        /// The name of the file.
        name: String,
        /// The contents of the file.
        contents: Vec<u8>,
        /// The contents of the file after having been parsed.
        parsed_contents: ParsedContents,
        /// The metadata of the file after having been processed.
        metadata: Option<Value>,
        /// The filesystem source of the file.
        source: PathBuf,
    },
    /// A directory in the virtual filesystem tree.
    Directory {
        /// The name of the directory.
        name: String,
        /// The children of the directory.
        children: Vec<Node>,
        /// The filesystem source of the directory.
        source: PathBuf,
    },
}

/// A filesystem error.
#[derive(Clone, Debug)]
pub enum Error {
    /// The filesystem source could not be found.
    NotFound(String),
    /// The filesystem source could not be read.
    Read,
    /// The filesystem source could not be written.
    Write,
    /// The file could not be parsed.
    Parse(TracebackError<ParseError>),
    /// A conflict occurred when merging two virtual filesystems.
    Conflict(PathBuf, PathBuf),
}

/// The parsed contents of a file.
#[derive(Clone, Debug)]
pub enum ParsedContents {
    /// An HTML file, parsed into template tokens.
    Html(Vec<LocatableToken>),
    /// A markdown file, parsed into frontmatter and HTML.
    Markdown(ParsedMarkdown),
    /// A JSON file.
    Json(Value),
    /// The file was not parsed.
    None,
}

impl Node {
    /// Constructs a new virtual filesystem tree from the given filesystem path.
    pub fn new(root: impl AsRef<Path>, parse: bool) -> Result<Self, Error> {
        let root = root
            .as_ref()
            .to_path_buf()
            .canonicalize()
            .map_err(|_| Error::NotFound(root.as_ref().to_string_lossy().to_string()))?;

        Self::create_from_dir(root, parse)
    }

    /// Returns `true` if the node is a directory.
    pub fn is_dir(&self) -> bool {
        matches!(self, Node::Directory { .. })
    }

    /// Returns `true` if the node is a file.
    pub fn is_file(&self) -> bool {
        matches!(self, Node::File { .. })
    }

    /// Returns the name of the node.
    pub fn name(&self) -> &str {
        match self {
            Node::File { name, .. } => name,
            Node::Directory { name, .. } => name,
        }
    }

    /// Returns the node's children.
    pub fn children(&self) -> Option<&[Node]> {
        match self {
            Node::Directory { children, .. } => Some(children),
            Node::File { .. } => None,
        }
    }

    /// Returns the node's contents.
    pub fn contents(&self) -> Option<&[u8]> {
        match self {
            Node::File { contents, .. } => Some(contents),
            Node::Directory { .. } => None,
        }
    }

    /// Returns the node's parsed contents.
    pub fn parsed_contents(&self) -> &ParsedContents {
        match self {
            Node::File {
                parsed_contents, ..
            } => parsed_contents,
            Node::Directory { .. } => &ParsedContents::None,
        }
    }

    /// Returns the filesystem source of the node.
    pub fn source(&self) -> &Path {
        match self {
            Node::File { source, .. } => source,
            Node::Directory { source, .. } => source,
        }
    }

    /// Attempts to get a node at the given path of the filesystem.
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

    /// Creates a new node from a directory of the filesystem.
    pub(crate) fn create_from_dir(dir: impl AsRef<Path>, parse: bool) -> Result<Self, Error> {
        let dir = dir.as_ref();
        let content =
            read_dir(&dir).map_err(|_| Error::NotFound(dir.to_string_lossy().to_string()))?;

        let children = content
            .flatten()
            .map(|path| {
                let path = path.path();

                match metadata(&path).map(|m| m.file_type()) {
                    Ok(t) if t.is_dir() => Self::create_from_dir(&path, parse),
                    Ok(t) if t.is_file() => Self::create_from_file(&path, parse),
                    _ => Err(Error::Read),
                }
            })
            .collect::<Result<_, _>>()?;

        Ok(Node::Directory {
            name: dir.file_name().unwrap().to_string_lossy().to_string(),
            children,
            source: dir.to_path_buf(),
        })
    }

    /// Creates a new node from a file of the filesystem.
    pub(crate) fn create_from_file(file: impl AsRef<Path>, parse: bool) -> Result<Self, Error> {
        let file = file.as_ref();
        let name = file.file_name().unwrap().to_string_lossy().to_string();
        let contents = read(&file).map_err(|_| Error::Read)?;

        let parsed_contents = if parse {
            let extension = file.extension().map(|e| e.to_string_lossy().to_string());
            let contents_string = std::str::from_utf8(&contents).map_err(|_| Error::Read);

            match extension.as_deref() {
                Some("html") => {
                    ParsedContents::Html(parse_html(contents_string?, file).map_err(Error::Parse)?)
                }
                Some("md") => ParsedContents::Markdown(
                    parse_markdown(contents_string?.to_string(), file).map_err(Error::Parse)?,
                ),
                Some("json") => ParsedContents::Json(
                    humphrey_json::from_str(contents_string?).map_err(|_| {
                        Error::Parse(TracebackError {
                            path: file.to_path_buf(),
                            kind: ParseError::InvalidJson,
                            column: 0,
                            line: 0,
                        })
                    })?,
                ),
                _ => ParsedContents::None,
            }
        } else {
            ParsedContents::None
        };

        Ok(Node::File {
            name,
            contents,
            parsed_contents,
            metadata: None,
            source: file.to_path_buf(),
        })
    }

    /// Save the node to the filesystem with the given configuration.
    pub fn save(&self, path: impl AsRef<Path>, config: &Config) -> Result<(), Error> {
        let path = path.as_ref().to_path_buf();

        if path.exists() && path.is_dir() {
            remove_dir_all(&path).map_err(|_| Error::Write)?;
        }

        match self {
            Self::Directory { children, .. } => {
                create_dir(&path).map_err(|_| Error::Write)?;

                for child in children {
                    child.save_recur(&path, config)?;
                }
            }
            _ => panic!("`Node::save` should only be used on the root directory"),
        }

        Ok(())
    }

    /// Save the node's metadata to the given path.
    /// The `base` argument should be a JSON object to which the metadata will be added under the key `data`.
    pub fn save_metadata(&self, mut base: Value, path: impl AsRef<Path>) -> Result<(), Error> {
        base["data"] = self.save_metadata_recur(true);

        write(path, base.serialize()).map_err(|_| Error::Write)?;

        Ok(())
    }

    /// Merge two virtual filesystem trees into a single virtual filesystem tree.
    /// This will return an error if two files share the same path.
    pub fn merge(&mut self, other: Node) -> Result<(), Error> {
        match (self, other) {
            (
                Self::Directory { children, .. },
                Self::Directory {
                    children: other_children,
                    ..
                },
            ) => {
                for other_child in other_children {
                    if let Some(child) = children
                        .iter_mut()
                        .find(|child| child.name() == other_child.name())
                    {
                        // This is definitely not the best way of doing this (it should be done through destructuring in a match statement),
                        //   but I can't seem to get around lifetime problems with the other way.
                        if matches!(child, Self::Directory { .. })
                            && matches!(other_child, Self::Directory { .. })
                        {
                            child.merge(other_child)?;
                        } else {
                            return Err(Error::Conflict(
                                child.source().to_path_buf(),
                                other_child.source().to_path_buf(),
                            ));
                        }
                    } else {
                        children.push(other_child);
                    }
                }

                Ok(())
            }
            _ => panic!("`Node::merge` should only be used on directories"),
        }
    }

    /// Recursively saves this node and its descendants to the filesystem.
    fn save_recur(&self, path: impl AsRef<Path>, config: &Config) -> Result<(), Error> {
        let path = path.as_ref().to_path_buf();

        match self {
            Self::Directory { name, children, .. } => {
                let dir = path.join(name);

                // It is possible that the directory already exists if strip extensions is enabled.
                match create_dir(&dir) {
                    Ok(_) => (),
                    Err(e) if e.kind() == ErrorKind::AlreadyExists => (),
                    Err(_) => return Err(Error::Write),
                };

                for child in children {
                    child.save_recur(&dir, config)?;
                }
            }
            Self::File { name, contents, .. } => {
                if name != "root.html"
                    && name != "md.html"
                    && (config.save_data_files || !name.ends_with(".json"))
                {
                    if config.strip_extensions && name.ends_with(".html") && name != "index.html" {
                        let directory_name = name.strip_suffix(".html").unwrap().to_string();
                        let dir = path.join(directory_name);

                        match create_dir(&dir) {
                            Ok(_) => (),
                            Err(e) if e.kind() == ErrorKind::AlreadyExists => (),
                            Err(_) => return Err(Error::Write),
                        };

                        write(dir.join("index.html"), contents).map_err(|_| Error::Write)?;
                    } else {
                        write(path.join(name), contents).map_err(|_| Error::Write)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Recursively exports this node's and its descendants' metadata to a JSON object.
    fn save_metadata_recur(&self, is_first: bool) -> Value {
        match self {
            Self::Directory { name, children, .. } => {
                let children = children
                    .iter()
                    .map(|c| c.save_metadata_recur(false))
                    .collect();

                if is_first {
                    Value::Array(children)
                } else {
                    json!({
                        "type": "directory",
                        "name": name,
                        "children": (Value::Array(children))
                    })
                }
            }
            Self::File {
                name,
                metadata: json,
                ..
            } => {
                let mut metadata = json!({ "name": name });

                if let Some(json) = json {
                    for (key, value) in json.as_object().unwrap() {
                        metadata[key.as_str()] = value.clone();
                    }
                } else {
                    metadata["type"] = json!("file");
                }

                metadata
            }
        }
    }
}

impl Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::File {
                name,
                contents,
                parsed_contents,
                metadata,
                source,
            } => f
                .debug_struct("File")
                .field("name", name)
                .field("contents", &format!("{} bytes", contents.len()))
                .field("parsed_contents", parsed_contents)
                .field("metadata", metadata)
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
    /// Returns the template tokens of the parsed contents, if applicable.
    pub fn tokens(&self) -> Option<&[LocatableToken]> {
        match self {
            Self::Html(tokens) => Some(tokens),
            _ => None,
        }
    }

    /// Returns the parsed markdown data, if applicable.
    pub fn markdown(&self) -> Option<&ParsedMarkdown> {
        match self {
            Self::Markdown(markdown) => Some(markdown),
            _ => None,
        }
    }

    /// Converts the parsed contents to a JSON value, if applicable.
    pub fn to_json(&self) -> Option<Value> {
        match self {
            ParsedContents::Html(_) => None,
            ParsedContents::None => None,

            ParsedContents::Markdown(md) => Some(json!({
                "type": "markdown",
                "value": (md.frontmatter_to_value())
            })),

            ParsedContents::Json(v) => Some(json!({
                "type": "json",
                "value": (v.clone())
            })),
        }
    }
}
