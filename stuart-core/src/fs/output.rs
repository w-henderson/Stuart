//! Provides a virtual filesystem tree which saves its files according to the configuration.

use crate::config::Config;
use crate::fs::Error;

use humphrey_json::prelude::*;
use humphrey_json::Value;

use std::fs::{create_dir, read, read_dir, remove_dir_all, write};
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};

/// Represents an output node in the virtual filesystem tree.
#[derive(Clone, Debug)]
pub enum OutputNode {
    /// A file in the virtual filesystem tree.
    File {
        /// The name of the file.
        name: String,
        /// The contents of the file.
        contents: Vec<u8>,
        /// The filesystem source of the file.
        source: PathBuf,
        /// The JSON metadata of the file, if enabled and available.
        json: Option<Value>,
    },
    /// A directory in the virtual filesystem tree.
    Directory {
        /// The name of the directory.
        name: String,
        /// The children of the directory.
        children: Vec<OutputNode>,
        /// The filesystem source of the directory.
        source: PathBuf,
    },
}

impl OutputNode {
    /// Constructs a new output node from the filesystem source.
    /// This is used when merging static content into the virtual filesystem tree.
    ///
    /// **Note:** this does not parse the file.
    pub fn new(root: impl AsRef<Path>) -> Result<Self, Error> {
        let root = root
            .as_ref()
            .to_path_buf()
            .canonicalize()
            .map_err(|_| Error::NotFound(root.as_ref().to_string_lossy().to_string()))?;

        Self::create_from_dir(root)
    }

    /// Save the output node to the filesystem with the given configuration.
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
            _ => panic!("`OutputNode::save` should only be used on the root directory"),
        }

        Ok(())
    }

    /// Save the output node's metadata to the given path.
    /// The `base` argument should be a JSON object to which the metadata will be added under the key `data`.
    pub fn save_metadata(&self, mut base: Value, path: impl AsRef<Path>) -> Result<(), Error> {
        base["data"] = self.save_metadata_recur(true);

        write(path, base.serialize()).map_err(|_| Error::Write)?;

        Ok(())
    }

    /// Merge two virtual filesystem trees into a single virtual filesystem tree.
    /// This will return an error if two files share the same path.
    pub fn merge(&mut self, other: OutputNode) -> Result<(), Error> {
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
            _ => panic!("`OutputNode::merge` should only be used on directories"),
        }
    }

    /// Attempts to get the output node at the given path.
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

    /// Returns the name of the output node.
    pub fn name(&self) -> &str {
        match self {
            Self::File { name, .. } => name,
            Self::Directory { name, .. } => name,
        }
    }

    /// Returns the filesystem source of the output node.
    pub fn source(&self) -> &Path {
        match self {
            Self::File { source, .. } => source,
            Self::Directory { source, .. } => source,
        }
    }

    /// Returns the children of the output node.
    pub fn children(&self) -> Option<&[Self]> {
        match self {
            Self::Directory { children, .. } => Some(children),
            Self::File { .. } => None,
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
            Self::File { name, json, .. } => {
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

    /// Constructs an output node from a directory.
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

        Ok(Self::Directory {
            name: dir.file_name().unwrap().to_string_lossy().to_string(),
            children,
            source: dir.to_path_buf(),
        })
    }

    /// Constructs an output node from a file.
    fn create_from_file(file: impl AsRef<Path>) -> Result<Self, Error> {
        let file = file.as_ref();
        let name = file.file_name().unwrap().to_string_lossy().to_string();
        let contents = read(&file).map_err(|_| Error::Read)?;

        Ok(Self::File {
            name,
            contents,
            source: file.to_path_buf(),
            json: None,
        })
    }
}
