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
}

/// The context of the build.
pub struct StuartContext {
    /// The builder.
    pub stuart: Stuart,
    /// The scripts to run before and after the build.
    pub scripts: Scripts,
    /// The STUART_ENV environment variable.
    pub stuart_env: String,
    /// The project directory.
    pub project_dir: PathBuf,
    /// The output directory, relative to the project directory.
    pub output: String,
}

impl StuartContext {
    /// Initialises the context.
    pub fn init(
        manifest_path: &str,
        output: &str,
        stuart_env: &str,
    ) -> Result<Self, Box<dyn StuartError>> {
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

        let plugins = plugins::load(&config.dependencies, path.parent().unwrap())?;

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

        let stuart = Stuart::new(path.parent().unwrap().join("content"))
            .with_config(config)
            .with_plugins(plugins);

        Ok(StuartContext {
            stuart,
            scripts,
            stuart_env: stuart_env.into(),
            project_dir: path.parent().unwrap().to_path_buf(),
            output: output.into(),
        })
    }

    /// Builds the site with the given configuration.
    pub fn build(&mut self) -> Result<BuildInfo, Box<dyn StuartError>> {
        let pre_build_start = Instant::now();
        self.scripts.execute_pre_build()?;
        let pre_build_duration = pre_build_start.elapsed().as_micros();

        log!(
            "Building",
            "{} ({})",
            self.stuart.config.name,
            self.project_dir
                .to_string_lossy()
                .trim_start_matches("\\\\?\\")
        );

        let build_start = Instant::now();
        self.stuart.build(self.stuart_env.to_string())?;
        let build_duration = build_start.elapsed().as_micros();

        for dir in ["static", "temp"] {
            let dir_path = self.project_dir.join(dir);

            if dir_path.exists() {
                let node = Node::new(dir_path, false)?;
                self.stuart.merge_output(node)?;
            }
        }

        remove_dir_all(self.project_dir.join("temp")).ok();

        let save_start = Instant::now();
        self.stuart.save(self.project_dir.join(&self.output))?;
        let save_duration = save_start.elapsed().as_micros();

        if self.stuart.config.save_metadata {
            log!("Exporting", "metadata to `metadata.json`");

            let metadata_path = self.project_dir.join("metadata.json");
            self.stuart.save_metadata(&metadata_path)?;
        }

        let post_build_start = Instant::now();
        self.scripts.execute_post_build()?;
        let post_build_duration = post_build_start.elapsed().as_micros();

        let total_duration =
            ((pre_build_duration + build_duration + save_duration + post_build_duration) / 100)
                as f64
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
}
