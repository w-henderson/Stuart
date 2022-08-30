use crate::config;
use crate::error::StuartError;
use crate::scripts::Scripts;

use stuart_core::{Node, OutputNode, Stuart, TracebackError};

use std::fs::{read_to_string, remove_dir_all};
use std::path::PathBuf;
use std::time::Instant;

pub struct BuildInfo {
    pub total_duration: f64,
    pub build_duration: f64,
    pub scripts_duration: f64,
    pub fs_duration: f64,
}

pub fn build(manifest_path: &str, output: &str) -> Result<BuildInfo, Box<dyn StuartError>> {
    let path = PathBuf::try_from(&manifest_path)
        .ok()
        .and_then(|path| path.canonicalize().ok())
        .ok_or_else(|| "invalid manifest path".to_string())?;

    let manifest =
        read_to_string(&path).map_err(|e| format!("failed to read manifest:\n  {}", e))?;

    let config = match config::load(&manifest) {
        Ok(config) => config,
        Err(e) => match e.line_col() {
            Some((line, col)) => {
                return Err(Box::new(TracebackError {
                    path,
                    line: line as u32 + 1,
                    column: col as u32 + 1,
                    kind: e.to_string(),
                }))
            }
            _ => return Err(Box::new(format!("failed to parse manifest:\n  {}", e))),
        },
    };

    let scripts = Scripts::from_directory(path.parent().unwrap().join("scripts"));

    let pre_build_start = Instant::now();
    scripts.execute_pre_build()?;
    let pre_build_duration = pre_build_start.elapsed().as_micros();

    log!(
        "Building",
        "{} ({})",
        config.name,
        path.parent()
            .unwrap()
            .to_string_lossy()
            .trim_start_matches("\\\\?\\")
    );

    let build_start = Instant::now();
    let fs = Node::new(path.parent().unwrap().join("content"))?;
    let mut stuart = Stuart::new(fs, config.clone());
    stuart.build()?;
    let build_duration = build_start.elapsed().as_micros();

    for dir in ["static", "temp"] {
        let dir_path = path.parent().unwrap().join(dir);

        if dir_path.exists() {
            let node = OutputNode::new(path.parent().unwrap().join(dir))?;
            stuart.merge_output(node)?;
        }
    }

    remove_dir_all(path.parent().unwrap().join("temp")).ok();

    let save_start = Instant::now();
    stuart.save(path.parent().unwrap().join(output))?;
    let save_duration = save_start.elapsed().as_micros();

    if config.save_metadata {
        log!("Exporting", "metadata to `metadata.json`");

        let metadata_path = path.parent().unwrap().join("metadata.json");
        stuart.save_metadata(&metadata_path)?;
    }

    let post_build_start = Instant::now();
    scripts.execute_post_build()?;
    let post_build_duration = post_build_start.elapsed().as_micros();

    let total_duration =
        ((pre_build_duration + build_duration + save_duration + post_build_duration) / 100) as f64
            / 10.0;
    let build_duration = (build_duration / 100) as f64 / 10.0;
    let fs_duration = (save_duration / 100) as f64 / 10.0;
    let scripts_duration = ((pre_build_duration + post_build_duration) / 100) as f64 / 10.0;

    log!(
        "Finished",
        "build in {}ms ({}ms build, {}ms scripts, {}ms filesystem)",
        total_duration,
        build_duration,
        scripts_duration,
        fs_duration
    );

    Ok(BuildInfo {
        total_duration,
        build_duration,
        scripts_duration,
        fs_duration,
    })
}
