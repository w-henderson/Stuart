#![allow(clippy::redundant_closure_call)]

use crate::{app, build};

use std::fs::{remove_dir_all, remove_file};
use std::path::Path;
use std::process::exit;

macro_rules! test {
    ($name:ident, $manifest_path:expr, $post_build_checks:expr) => {
        #[test]
        fn $name() {
            let mut result = full_build(concat!(
                env!("CARGO_MANIFEST_DIR"),
                $manifest_path,
                "/stuart.toml"
            ));

            result &= ($post_build_checks)(concat!(env!("CARGO_MANIFEST_DIR"), $manifest_path));

            cleanup(concat!(
                env!("CARGO_MANIFEST_DIR"),
                $manifest_path,
                "/stuart.toml"
            ));

            if !result {
                exit(1);
            }
        }
    };
}

test!(basic, "/tests/basic", |_| true);

#[cfg(feature = "js")]
test!(js, "/tests/js", |path| {
    return contents(path, "index.html").trim() == "5";
});

fn full_build(manifest_path: &str) -> bool {
    let args = app().get_matches_from(vec!["stuart", "build", "--manifest-path", manifest_path]);
    let result = match args.subcommand() {
        Some(("build", args)) => build(args),
        _ => unreachable!(),
    };

    if let Err(e) = result {
        e.print();
        false
    } else {
        true
    }
}

fn cleanup(manifest_path: &str) {
    let path = Path::new(manifest_path);
    let dist = path.parent().unwrap().join("dist");
    let _ = remove_dir_all(dist);
    let _ = remove_file(path.parent().unwrap().join("metadata.json"));
}

fn contents(path: &str, dist_path: &str) -> String {
    std::fs::read_to_string(Path::new(path).join("dist").join(dist_path)).unwrap()
}
