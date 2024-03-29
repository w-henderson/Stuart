//! Provides the `stuart new` functionality.

use crate::config::git;
use crate::error::StuartError;

use stuart_core::error::FsError;

use clap::ArgMatches;
use include_dir::{include_dir, Dir, DirEntry};

use std::fs::{create_dir, write};
use std::io::Write;
use std::path::{Path, PathBuf};

/// The directory containing the default site template, built into the binary when compiled.
static DEFAULT_PROJECT: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/tests/basic");

/// Creates a new site with the given arguments.
pub fn new(args: &ArgMatches) -> Result<(), Box<dyn StuartError>> {
    let name = args.value_of("name").unwrap();
    let path = PathBuf::from(name);
    let no_git = args.is_present("no-git");

    let mut manifest: Vec<u8> = format!("[site]\nname = \"{}\"", name).as_bytes().to_vec();

    if let Some((name, email)) =
        git::get_user_name().and_then(|name| git::get_user_email().map(|email| (name, email)))
    {
        write!(&mut manifest, "\nauthor = \"{} <{}>\"", name, email).unwrap();
    }

    manifest.push(b'\n');

    create_dir(&path).map_err(|_| FsError::Write)?;
    create_dir(path.join("content")).map_err(|_| FsError::Write)?;
    create_dir(path.join("static")).map_err(|_| FsError::Write)?;
    write(path.join("stuart.toml"), manifest).map_err(|_| FsError::Write)?;

    extract(&path, &DEFAULT_PROJECT)?;

    if !no_git {
        git::init_repository(&format!("./{}", name));

        write(path.join(".gitignore"), b"dist/\n_build/\nmetadata.json\n")
            .map_err(|_| FsError::Write)?;
    }

    log!("Created", "new Stuart website `{}`", name);

    Ok(())
}

/// Extracts the embedded directory to the filesystem.
fn extract(root: &Path, dir: &Dir) -> Result<(), FsError> {
    for child in dir.entries() {
        match child {
            DirEntry::Dir(dir) => extract(root, dir)?,
            DirEntry::File(file) => {
                if !file.path().ends_with("stuart.toml") {
                    write(root.join(file.path()), file.contents()).map_err(|_| FsError::Write)?
                }
            }
        }
    }

    Ok(())
}
