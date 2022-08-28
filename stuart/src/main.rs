use clap::{App, Arg, ArgMatches, Command};
use stuart::{fs, Config, Node, OutputNode, Scripts, Stuart, StuartError, TracebackError};

use std::fs::{create_dir, read_to_string, remove_dir_all, write};
use std::io::Write;
use std::path::PathBuf;

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
                .arg(Arg::new("name").help("Name of the site").required(true))
                .arg(
                    Arg::new("no-git")
                        .long("no-git")
                        .help("Don't initialize a Git repository"),
                ),
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
    stuart.build()?;

    for dir in ["static", "temp"] {
        let dir_path = path.parent().unwrap().join(dir);

        if dir_path.exists() {
            let node = OutputNode::new(path.parent().unwrap().join(dir))?;
            stuart.merge_output(node)?;
        }
    }

    remove_dir_all(path.parent().unwrap().join("temp")).ok();

    stuart.save(path.parent().unwrap().join(output))?;

    scripts.execute_post_build()?;

    Ok(())
}

fn new(args: &ArgMatches) -> Result<(), Box<dyn StuartError>> {
    let name = args.value_of("name").unwrap();
    let path = PathBuf::try_from(name).map_err(|_| fs::Error::Write)?;
    let no_git = args.is_present("no-git");

    let mut manifest: Vec<u8> = format!("[site]\nname = \"{}\"", name).as_bytes().to_vec();

    if let Some((name, email)) = stuart::config::git::get_user_name()
        .and_then(|name| stuart::config::git::get_user_email().map(|email| (name, email)))
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
        stuart::config::git::init_repository(&format!("./{}", name));
    }

    Ok(())
}
