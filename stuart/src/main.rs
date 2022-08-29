#[macro_use]
mod logger;

mod build;
mod config;
mod error;
mod scripts;
mod serve;

use crate::error::StuartError;
use crate::logger::{LogLevel, Logger, LOGGER};

use clap::{App, Arg, ArgMatches, Command};
use stuart_core::fs;

use std::fs::{create_dir, write};
use std::io::Write;
use std::path::PathBuf;

fn main() {
    let matches = App::new("Stuart")
        .version(env!("CARGO_PKG_VERSION"))
        .author("William Henderson <william-henderson@outlook.com>")
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
        .subcommand(Command::new("dev").about("Starts the development server"))
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
        Some(("dev", _)) => Ok(serve::serve()),
        Some(("new", args)) => new(args),
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

fn build(args: &ArgMatches) -> Result<(), Box<dyn StuartError>> {
    let manifest_path: &str = args.value_of("manifest-path").unwrap();
    let output: &str = args.value_of("output").unwrap();

    build::build(manifest_path, output)
}

fn new(args: &ArgMatches) -> Result<(), Box<dyn StuartError>> {
    let name = args.value_of("name").unwrap();
    let path = PathBuf::try_from(name).map_err(|_| fs::Error::Write)?;
    let no_git = args.is_present("no-git");

    let mut manifest: Vec<u8> = format!("[site]\nname = \"{}\"", name).as_bytes().to_vec();

    if let Some((name, email)) = config::git::get_user_name()
        .and_then(|name| config::git::get_user_email().map(|email| (name, email)))
    {
        write!(&mut manifest, "\nauthor = \"{} <{}>\"", name, email).unwrap();
    }

    manifest.push(b'\n');

    create_dir(&path).map_err(|_| fs::Error::Write)?;
    create_dir(path.join("content")).map_err(|_| fs::Error::Write)?;
    create_dir(path.join("static")).map_err(|_| fs::Error::Write)?;

    write(path.join("stuart.toml"), manifest).map_err(|_| fs::Error::Write)?;
    write(
        path.join("content/index.html"),
        include_bytes!("../default_project/index.html"),
    )
    .map_err(|_| fs::Error::Write)?;
    write(
        path.join("content/root.html"),
        include_bytes!("../default_project/root.html"),
    )
    .map_err(|_| fs::Error::Write)?;
    write(
        path.join("static/ferris.svg"),
        include_bytes!("../default_project/ferris.svg"),
    )
    .map_err(|_| fs::Error::Write)?;

    if !no_git {
        config::git::init_repository(&format!("./{}", name));
    }

    log!("Created", "new Stuart website `{}`", name);

    Ok(())
}
