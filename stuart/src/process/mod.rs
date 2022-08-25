pub mod error;

use self::error::{ProcessError, TracebackError};

use crate::fs::Node;
use crate::SpecialFiles;

impl Node {
    pub fn process(
        &mut self,
        special_files: SpecialFiles,
    ) -> Result<(), TracebackError<ProcessError>> {
        Ok(())
    }
}
