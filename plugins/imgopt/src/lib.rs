use stuart_core::plugins::{NodeParser, NodeProcessor};
use stuart_core::process::ProcessOutput;
use stuart_core::{declare_plugin, Environment, Stuart};

use oxipng::{optimize_from_memory, Options};

use std::path::Path;

declare_plugin! {
    name: "imgopt",
    version: "0.1.0",
    functions: [],
    parsers: [PngParser],
}

struct PngParser;
struct PngProcessor(Vec<u8>);

impl NodeParser for PngParser {
    fn extensions(&self) -> Vec<&'static str> {
        vec!["png"]
    }

    fn parse(&self, contents: &[u8], _: &Path) -> Result<Box<dyn NodeProcessor>, String> {
        Ok(Box::new(PngProcessor(contents.to_vec())))
    }
}

impl NodeProcessor for PngProcessor {
    fn process(&self, _: &Stuart, _: Environment) -> Result<ProcessOutput, String> {
        let opts = Options::from_preset(3);
        let optimized = optimize_from_memory(&self.0, &opts)
            .map_err(|e| format!("png optimization error: {}", e))?;

        Ok(ProcessOutput {
            new_contents: Some(optimized),
            new_name: None,
        })
    }
}
