use crate::config::Config;
use crate::fs::Error;

use std::fs::{create_dir, remove_dir_all, write};
use std::io::ErrorKind;
use std::path::Path;

#[derive(Clone)]
pub enum OutputNode {
    File {
        name: String,
        contents: Vec<u8>,
    },
    Directory {
        name: String,
        children: Vec<OutputNode>,
    },
}

impl OutputNode {
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

    fn save_recur(&self, path: impl AsRef<Path>, config: &Config) -> Result<(), Error> {
        let path = path.as_ref().to_path_buf();

        match self {
            Self::Directory { name, children } => {
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
            Self::File { name, contents } => {
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
}
