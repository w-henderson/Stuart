#[macro_use]
mod r#macro;

use crate::{Config, Node, OutputNode, SpecialFiles, Stuart};

use std::path::PathBuf;

define_testcases![for_loop_markdown, for_loop_json_file, for_loop_json_object];

pub struct Testcase {
    context: Node,
    input: Node,
    output: OutputNode,
}

impl Testcase {
    pub fn new(name: &str) -> Self {
        // Load the base context from the `_base` testcase.
        let mut context = load_base();

        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/tests/testcases")
            .join(name);

        let input = Node::create_from_file(path.join("in.html")).unwrap();
        let output = OutputNode::create_from_file(path.join("out.html")).unwrap();

        // Add the input to the base context.
        match context {
            Node::Directory {
                ref mut children, ..
            } => children.push(input.clone()),
            _ => panic!("Base node is not a directory"),
        }

        Self {
            context,
            input,
            output,
        }
    }

    pub fn run(&self) {
        // Create a mock processing scenario.
        let stuart = Stuart::new(self.context.clone(), Config::default());
        let specials = SpecialFiles {
            root: self
                .context
                .get_at_path(&PathBuf::from("root.html"))
                .unwrap()
                .parsed_contents()
                .tokens(),
            md: self
                .context
                .get_at_path(&PathBuf::from("md.html"))
                .unwrap()
                .parsed_contents()
                .tokens(),
        };

        // Process the input node.
        let out = self.input.process(&stuart, specials).unwrap();

        match (&out, &self.output) {
            (
                OutputNode::File { contents, .. },
                OutputNode::File {
                    contents: expected_contents,
                    ..
                },
            ) => {
                // Check the two outputs match.
                // Newlines and carriage returns are removed since Stuart (currently) makes no guarantees about how it outputs them.
                // The arrays are converted to strings purely so the error messages are easier to read; it has no effect on the actual comparison.
                assert_eq!(
                    std::str::from_utf8(contents)
                        .unwrap()
                        .replace('\n', "")
                        .replace('\r', ""),
                    std::str::from_utf8(expected_contents)
                        .unwrap()
                        .replace('\n', "")
                        .replace('\r', "")
                );
            }
            _ => panic!("Not both files"),
        }
    }
}

fn load_base() -> Node {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/tests/testcases/_base");
    Node::create_from_dir(path).unwrap()
}
