use crate::config::Config;
use crate::fs::Error;

use std::fs::{create_dir, read, read_dir, remove_dir_all, write};
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};

#[derive(Clone, Debug)]
pub enum OutputNode {
    File {
        name: String,
        contents: Vec<u8>,
        source: PathBuf,
    },
    Directory {
        name: String,
        children: Vec<OutputNode>,
        source: PathBuf,
    },
}

impl OutputNode {
    pub fn new(root: impl AsRef<Path>) -> Result<Self, Error> {
        let root = root
            .as_ref()
            .to_path_buf()
            .canonicalize()
            .map_err(|_| Error::NotFound(root.as_ref().to_string_lossy().to_string()))?;

        Self::create_from_dir(root)
    }

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

    pub fn name(&self) -> &str {
        match self {
            Self::File { name, .. } => name,
            Self::Directory { name, .. } => name,
        }
    }

    pub fn source(&self) -> &Path {
        match self {
            Self::File { source, .. } => source,
            Self::Directory { source, .. } => source,
        }
    }

    pub fn children(&self) -> Option<&[Self]> {
        match self {
            Self::Directory { children, .. } => Some(children),
            Self::File { .. } => None,
        }
    }

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

    fn create_from_file(file: impl AsRef<Path>) -> Result<Self, Error> {
        let file = file.as_ref();
        let name = file.file_name().unwrap().to_string_lossy().to_string();
        let contents = read(&file).map_err(|_| Error::Read)?;

        Ok(Self::File {
            name,
            contents,
            source: file.to_path_buf(),
        })
    }
}
