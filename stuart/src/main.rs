//! Stuart: A Blazingly-Fast Static Site Generator.

#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

#[macro_use]
mod logger;

mod build;
mod config;
mod error;
mod new;
mod plugins;
mod scripts;
mod serve;

use crate::error::StuartError;
use crate::logger::{LogLevel, Logger, Progress, LOGGER};

use clap::{App, Arg, ArgMatches, Command};

use std::fs::{remove_dir_all, remove_file};
use std::path::PathBuf;
use std::sync::atomic::Ordering;

fn main() {
    let matches = App::new("Stuart")
        .version(env!("CARGO_PKG_VERSION"))
        .author("William Henderson <william-henderson@outlook.com>")
        .about("A Blazingly-Fast Static Site Generator")
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .help("Suppress all output except errors")
                .conflicts_with("verbose"),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Output verbose information"),
        )
        .subcommand(
            Command::new("build")
                .about("Builds the site")
                .arg(
                    Arg::new("manifest-path")
                        .long("manifest-path")
                        .help("Path to the manifest file")
                        .default_value("stuart.toml"),
                )
                .arg(
                    Arg::new("output")
                        .long("output")
                        .short('o')
                        .help("Output directory")
                        .default_value("dist"),
                ),
        )
        .subcommand(
            Command::new("dev")
                .about("Starts the development server")
                .arg(
                    Arg::new("manifest-path")
                        .long("manifest-path")
                        .help("Path to the manifest file")
                        .default_value("stuart.toml"),
                )
                .arg(
                    Arg::new("output")
                        .long("output")
                        .short('o')
                        .help("Output directory")
                        .default_value("dist"),
                ),
        )
        .subcommand(
            Command::new("new")
                .about("Creates a new site")
                .arg(Arg::new("name").help("Name of the site").required(true))
                .arg(
                    Arg::new("no-git")
                        .long("no-git")
                        .help("Don't initialize a Git repository"),
                ),
        )
        .subcommand(
            Command::new("bench")
                .about("Performs a basic benchmark test")
                .arg(
                    Arg::new("iterations")
                        .short('i')
                        .long("iters")
                        .help("Number of iterations to perform")
                        .takes_value(true)
                        .default_value("10"),
                ),
        )
        .subcommand(
            Command::new("clean").about("Removes the output directory and generated metadata"),
        )
        .subcommand_required(true)
        .get_matches();

    let log_level = if matches.is_present("quiet") {
        LogLevel::Quiet
    } else if matches.is_present("verbose") {
        LogLevel::Verbose
    } else {
        LogLevel::Normal
    };

    Logger::new(log_level).register();

    #[allow(clippy::unit_arg)]
    let result = match matches.subcommand() {
        Some(("build", args)) => build(args),
        Some(("dev", args)) => serve::serve(args.clone()),
        Some(("new", args)) => new::new(args),
        Some(("bench", args)) => bench(args),
        Some(("clean", _)) => clean(),
        _ => unreachable!(),
    };

    if let Err(e) = result {
        if LOGGER.get().unwrap().has_logged() {
            println!();
        }

        e.print();
        std::process::exit(1);
    }
}

/// Runs the build command with the given arguments.
fn build(args: &ArgMatches) -> Result<(), Box<dyn StuartError>> {
    let manifest_path: &str = args.value_of("manifest-path").unwrap();
    let output: &str = args.value_of("output").unwrap();

    build::build(manifest_path, output, "production").map(|_| ())
}

/// Runs the benchmark command with the given arguments.
fn bench(args: &ArgMatches) -> Result<(), Box<dyn StuartError>> {
    let iters: usize = args
        .value_of("iterations")
        .unwrap()
        .parse()
        .map_err(|_| "invalid value for iterations")?;

    let mut total = 0.0;
    let mut total_build = 0.0;
    let mut total_scripts = 0.0;
    let mut total_fs = 0.0;
    let mut total_plugins = 0.0;

    LOGGER.get().unwrap().enabled.store(false, Ordering::SeqCst);

    let mut progress = Progress::new("Processing", iters);
    progress.print();

    for _ in 1..=iters {
        let result = build::build("stuart.toml", "dist", "benchmark")?;

        total += result.total_duration;
        total_build += result.build_duration;
        total_scripts += result.scripts_duration;
        total_fs += result.fs_duration;
        total_plugins += result.plugins_duration;

        progress.next();
    }

    println!();

    LOGGER.get().unwrap().enabled.store(true, Ordering::SeqCst);

    let avg = total / (iters as f64);
    let avg_build = total_build / (iters as f64);
    let avg_scripts = total_scripts / (iters as f64);
    let avg_fs = total_fs / (iters as f64);
    let avg_plugins = total_plugins / (iters as f64);

    log!("Total:", "{:.2}ms mean", avg);
    log!("Build:", "{:.2}ms mean", avg_build);
    log!("Scripts:", "{:.2}ms mean", avg_scripts);
    log!("Filesystem:", "{:.2}ms mean", avg_fs);
    log!("Plugins:", "{:.2}ms mean", avg_plugins);

    Ok(())
}

/// Removes the output directory and generated metadata.
fn clean() -> Result<(), Box<dyn StuartError>> {
    if !PathBuf::from("stuart.toml").exists() {
        return Err("current working directory is not a Stuart project".into());
    }

    if PathBuf::from("dist").exists() {
        remove_dir_all("dist").map_err(|_| "failed to remove output directory")?;
    }

    if PathBuf::from("_build").exists() {
        remove_dir_all("_build").map_err(|_| "failed to remove build directory")?;
    }

    if PathBuf::from("metadata.json").exists() {
        remove_file("metadata.json").map_err(|_| "failed to remove metadata file")?;
    }

    Ok(())
}
