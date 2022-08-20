pub mod fs;
pub mod parse;

use crate::fs::Node;

use std::path::Path;

// Special files:
// - md.html
// - root.html

pub struct Stuart {
    fs: Node,
}

impl Stuart {
    pub fn new(fs: Node) -> Self {
        Self { fs }
    }

    pub fn build(&mut self) {}

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), fs::Error> {
        self.fs.save(path)
    }
}
