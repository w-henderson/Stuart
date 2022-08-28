/*use stuart::config::Config;
use stuart::fs::Node;
use stuart::Stuart;

static IN: &str = "C:/Users/willi/OneDrive/StuartPortfolio";
static OUT: &str = "C:/Users/willi/OneDrive/StuartPortfolio/dist";

fn main() {
    let start = std::time::Instant::now();
    let fs = Node::new(IN).unwrap();
    let mut stuart = Stuart::new(fs, Config::default());
    stuart.build(OUT).unwrap();
    let duration = start.elapsed().as_micros();
    println!("took {}us", duration);
}
*/

use clap::{App, Arg, ArgMatches, Command};
use stuart::{Config, Node, Stuart, StuartError};

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

    let config =
        Config::load(&manifest).map_err(|e| format!("failed to parse manifest:\n  {}", e))?;

    let fs = Node::new(path.parent().unwrap().join("content"))?;

    let mut stuart = Stuart::new(fs, config);
    stuart.build(output)?;

    Ok(())
}

fn new(args: &ArgMatches) -> Result<(), Box<dyn StuartError>> {
    Ok(())
}
