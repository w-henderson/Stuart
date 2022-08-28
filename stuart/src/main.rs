use clap::{App, Arg, ArgMatches, Command};
use stuart::{Config, Node, Scripts, Stuart, StuartError, TracebackError};

use std::{fs::read_to_string, path::PathBuf};

fn main() {
    let matches = App::new("Stuart")
        .version(env!("CARGO_PKG_VERSION"))
        .author("William Henderson <william-henderson@outlook.com>")
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
            Command::new("new")
                .about("Creates a new site")
                .arg(Arg::new("name").help("Name of the site").required(true)),
        )
        .subcommand_required(true)
        .get_matches();

    let result = match matches.subcommand() {
        Some(("build", args)) => build(args),
        Some(("new", args)) => new(args),
        _ => unreachable!(),
    };

    if let Err(e) = result {
        e.print();
        std::process::exit(1);
    }
}

fn build(args: &ArgMatches) -> Result<(), Box<dyn StuartError>> {
    let manifest_path: &str = args.value_of("manifest-path").unwrap();
    let output: &str = args.value_of("output").unwrap();

    let path = PathBuf::try_from(&manifest_path)
        .ok()
        .and_then(|path| path.canonicalize().ok())
        .ok_or_else(|| "invalid manifest path".to_string())?;

    let manifest =
        read_to_string(&path).map_err(|e| format!("failed to read manifest:\n  {}", e))?;

    let config = match Config::load(&manifest) {
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

    scripts.execute_pre_build()?;

    let fs = Node::new(path.parent().unwrap().join("content"))?;

    let mut stuart = Stuart::new(fs, config);
    stuart.build(output)?;

    scripts.execute_post_build()?;

    Ok(())
}

fn new(args: &ArgMatches) -> Result<(), Box<dyn StuartError>> {
    Ok(())
}
