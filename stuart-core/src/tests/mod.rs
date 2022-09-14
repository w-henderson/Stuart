#[macro_use]
mod r#macro;

use crate::process::stack::StackFrame;
use crate::{Environment, Node, Stuart};

use std::path::PathBuf;

define_testcases![
    for_loop_markdown,
    for_loop_json_file,
    for_loop_json_object,
    for_loop_nested,
    dateformat,
    excerpt,
    ifdefined,
    conditionals
];

pub struct Testcase {
    context: Node,
    input: Node,
    output: Node,
}

impl Testcase {
    pub fn new(name: &str) -> Self {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/tests/testcases")
            .join(name);

        // Load the base context from the `_base` testcase.
        let mut context = load_base();

        // Merge with the specific context for this testcase.
        let specific_context = Node::create_from_dir(&path, true, None).unwrap();
        context.merge(specific_context).unwrap();

        let input = Node::create_from_file(path.join("in.html"), true, None).unwrap();
        let output = Node::create_from_file(path.join("out.html"), true, None).unwrap();

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
        let mut stuart = Stuart::new_from_node(self.context.clone());
        stuart.base = Some(StackFrame::new("base"));

        let env = Environment {
            vars: &[],
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
        let out = self.input.process(&stuart, env).unwrap();

        match (&out, &self.output) {
            (
                Node::File { contents, .. },
                Node::File {
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
    Node::create_from_dir(path, true, None).unwrap()
}
