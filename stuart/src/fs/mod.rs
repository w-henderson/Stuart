use std::fmt::Debug;
use std::fs::{create_dir, read, read_dir, remove_dir_all, write};
use std::path::Path;

#[derive(Clone)]
pub enum Node {
    File { name: String, contents: Vec<u8> },
    Directory { name: String, children: Vec<Node> },
}

#[derive(Clone, Copy, Debug)]
pub enum Error {
    Path,
    NotFound,
    Read,
    Write,
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
            Self::File { name, contents } => {
                write(path.join(name), contents).map_err(|_| Error::Write)?;
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
        let contents = read(&file).map_err(|_| Error::Read)?;

        Ok(Node::File {
            name: file.file_name().unwrap().to_string_lossy().to_string(),
            contents,
        })
    }
}

impl Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::File { name, contents } => f
                .debug_struct("File")
                .field("name", name)
                .field("contents", &format!("{} bytes", contents.len()))
                .finish(),
            Self::Directory { name, children } => f
                .debug_struct("Directory")
                .field("name", name)
                .field("children", children)
                .finish(),
        }
    }
}
