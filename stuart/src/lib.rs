pub mod fs;
pub mod parse;

use crate::fs::Node;

use std::path::Path;

// Special files:
// - md.html
// - root.html

#[derive(Debug)]
pub struct Stuart {
    fs: Node,
    stack: Vec<usize>,
}

#[derive(Debug)]
pub struct SpecialFiles<'a> {
    pub root: Option<&'a Node>,
    pub md: Option<&'a Node>,
}

impl Stuart {
    pub fn new(fs: Node) -> Self {
        Self {
            fs,
            stack: Vec::new(),
        }
    }

    pub fn build(&mut self) {
        loop {
            while self.stack_target().map(|n| n.is_dir()).unwrap_or(false) {
                self.stack.push(0);
            }

            match self.stack_target() {
                Some(n) if n.is_file() => {
                    // process file
                    print!("{:?}", n);
                    let special_files = self.nearest_special_files();
                    println!(": {:?}", special_files);

                    let index = self.stack.pop().unwrap();
                    self.stack.push(index + 1);
                }
                None => {
                    self.stack.pop();

                    if self.stack.is_empty() {
                        break;
                    } else {
                        let index = self.stack.pop().unwrap();
                        self.stack.push(index + 1);
                    }
                }
                _ => unreachable!(),
            }
        }
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), fs::Error> {
        self.fs.save(path)
    }

    fn stack_target(&mut self) -> Option<&mut Node> {
        let mut n = &mut self.fs;

        for child in &self.stack {
            n = n.children_mut()?.get_mut(*child)?;
        }

        Some(n)
    }

    fn nearest_special_files(&self) -> SpecialFiles {
        let mut stack = Vec::with_capacity(self.stack.len());
        let mut n = &self.fs;

        for child in &self.stack {
            stack.push(n);
            n = n.children().unwrap().get(*child).unwrap();
        }

        let mut root = None;
        let mut md = None;

        for dir in stack.into_iter().rev() {
            if root.is_none() {
                if let Some(child) = dir
                    .children()
                    .unwrap()
                    .iter()
                    .find(|c| c.name() == "root.html")
                {
                    root = Some(child);
                }
            }

            if md.is_none() {
                if let Some(child) = dir
                    .children()
                    .unwrap()
                    .iter()
                    .find(|c| c.name() == "md.html")
                {
                    md = Some(child);
                }
            }

            if root.is_some() && md.is_some() {
                break;
            }
        }

        SpecialFiles { root, md }
    }
}
