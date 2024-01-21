use crate::{app, build};

use std::fs::{remove_dir_all, remove_file};
use std::path::Path;
use std::process::exit;

macro_rules! test {
    ($name:ident, $manifest_path:expr) => {
        #[test]
        fn $name() {
            full_build(concat!(env!("CARGO_MANIFEST_DIR"), $manifest_path));
        }
    };
}

test!(basic, "/tests/basic/stuart.toml");

fn full_build(manifest_path: &str) {
    let args = app().get_matches_from(vec!["stuart", "build", "--manifest-path", manifest_path]);
    let result = match args.subcommand() {
        Some(("build", args)) => build(args),
        _ => unreachable!(),
    };
    cleanup(manifest_path);

    if let Err(e) = result {
        e.print();
        exit(1);
    }
}

fn cleanup(manifest_path: &str) {
    let path = Path::new(manifest_path);
    let dist = path.parent().unwrap().join("dist");
    let _ = remove_dir_all(dist);
    let _ = remove_file(path.parent().unwrap().join("metadata.json"));
}
