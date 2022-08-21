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

    pub fn build(&mut self) {
        let mut stack: Vec<usize> = Vec::new();

        loop {
            while stack_target(&mut self.fs, &stack)
                .map(|n| n.is_dir())
                .unwrap_or(false)
            {
                stack.push(0);
            }

            match stack_target(&mut self.fs, &stack) {
                Some(n) if n.is_file() => {
                    // process file
                    println!("{:?}", n);

                    let index = stack.pop().unwrap();
                    stack.push(index + 1);
                }
                None => {
                    stack.pop();

                    if stack.is_empty() {
                        break;
                    } else {
                        let index = stack.pop().unwrap();
                        stack.push(index + 1);
                    }
                }
                _ => unreachable!(),
            }
        }
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), fs::Error> {
        self.fs.save(path)
    }
}

fn stack_target<'a>(root: &'a mut Node, stack: &[usize]) -> Option<&'a mut Node> {
    let mut n = root;

    for child in stack {
        n = n.children_mut()?.get_mut(*child)?;
    }

    Some(n)
}
