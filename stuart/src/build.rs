//! Provides the `stuart build` functionality.

use crate::error::StuartError;
use crate::scripts::Scripts;
use crate::{config, plugins};

use stuart_core::{Config, Node, Stuart, TracebackError};

use std::fs::{read_to_string, remove_dir_all};
use std::path::PathBuf;
use std::time::Instant;

/// Contains information about a successful build.
pub struct BuildInfo {
    /// The total time taken to build the site, in milliseconds.
    pub total_duration: f64,
    /// The time taken to build the site's content, in milliseconds.
    pub build_duration: f64,
    /// The time taken to execute all build scripts, in milliseconds.
    pub scripts_duration: f64,
    /// The time taken to write the site to disk, in milliseconds.
    pub fs_duration: f64,
    /// The time taken to download and compile any plugins, in milliseconds.
    pub plugins_duration: f64,
}

/// Builds the site with the given configuration.
pub fn build(
    manifest_path: &str,
    output: &str,
    stuart_env: &str,
) -> Result<BuildInfo, Box<dyn StuartError>> {
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

    let plugins_start = Instant::now();
    let plugins = plugins::load(&config.dependencies, path.parent().unwrap())?;
    let plugins_duration = plugins_start.elapsed().as_micros();

    let config: Config = config.into();

    let scripts = Scripts::from_directory(path.parent().unwrap().join("scripts"))
        .with_environment_variables(vec![
            (
                "STUART_MANIFEST_PATH".into(),
                path.to_string_lossy()
                    .trim_start_matches("\\\\?\\")
                    .to_string(),
            ),
            (
                "STUART_MANIFEST_DIR".into(),
                path.parent()
                    .unwrap()
                    .to_string_lossy()
                    .trim_start_matches("\\\\?\\")
                    .to_string(),
            ),
            (
                "STUART_TEMP_DIR".into(),
                path.parent()
                    .unwrap()
                    .join("temp")
                    .to_string_lossy()
                    .trim_start_matches("\\\\?\\")
                    .to_string(),
            ),
            (
                "STUART_OUT_DIR".into(),
                path.parent()
                    .unwrap()
                    .join(output)
                    .to_string_lossy()
                    .trim_start_matches("\\\\?\\")
                    .to_string(),
            ),
            ("STUART_ENV".into(), stuart_env.into()),
        ]);

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

    let mut stuart = Stuart::new(path.parent().unwrap().join("content"))
        .with_config(config.clone())
        .with_plugins(plugins);
    stuart.build(stuart_env.to_string())?;

    let build_duration = build_start.elapsed().as_micros();

    for dir in ["static", "temp"] {
        let dir_path = path.parent().unwrap().join(dir);

        if dir_path.exists() {
            let node = Node::new(path.parent().unwrap().join(dir), false)?;
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

    let total_duration = ((plugins_duration
        + pre_build_duration
        + build_duration
        + save_duration
        + post_build_duration)
        / 100) as f64
        / 10.0;
    let build_duration = (build_duration / 100) as f64 / 10.0;
    let fs_duration = (save_duration / 100) as f64 / 10.0;
    let scripts_duration = ((pre_build_duration + post_build_duration) / 100) as f64 / 10.0;
    let plugins_duration = (plugins_duration / 100) as f64 / 10.0;

    log!(
        "Finished",
        "build in {}ms ({}ms build, {}ms scripts, {}ms filesystem, {}ms plugins)",
        total_duration,
        build_duration,
        scripts_duration,
        fs_duration,
        plugins_duration
    );

    Ok(BuildInfo {
        total_duration,
        build_duration,
        scripts_duration,
        fs_duration,
        plugins_duration,
    })
}
