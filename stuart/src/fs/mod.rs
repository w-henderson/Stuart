use std::fmt::Debug;
use std::fs::{read, read_dir};
use std::path::Path;

#[derive(Clone)]
pub enum Node {
    File { name: String, contents: Vec<u8> },
    Directory { name: String, children: Vec<Node> },
}

#[derive(Clone, Copy, Debug)]
pub enum Error {
    NotFound,
    Read,
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
