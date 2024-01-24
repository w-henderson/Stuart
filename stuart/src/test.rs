#![allow(clippy::redundant_closure_call)]

use crate::{app, build};

use std::fs::{remove_dir_all, remove_file};
use std::path::Path;
use std::process::exit;

macro_rules! test {
    ($name:ident, $manifest_path:expr, $post_build_checks:expr) => {
        #[test]
        fn $name() {
            let result = full_build(concat!(
                env!("CARGO_MANIFEST_DIR"),
                $manifest_path,
                "/stuart.toml"
            ));

            if !result {
                cleanup(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    $manifest_path,
                    "/stuart.toml"
                ));

                exit(1);
            }

            let index = std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                $manifest_path,
                "/dist/index.html"
            ))
            .unwrap();

            cleanup(concat!(
                env!("CARGO_MANIFEST_DIR"),
                $manifest_path,
                "/stuart.toml"
            ));

            $post_build_checks(index.trim());
        }
    };
}

test!(basic, "/tests/basic", |_| ());

#[cfg(feature = "js")]
test!(js, "/tests/js", |index: &str| {
    let mut lines = index.lines().map(|s| s.trim());
    assert_eq!(lines.next().unwrap(), "5"); // add(2, 3)
    assert_eq!(lines.next().unwrap(), "1,3,4,5,6,8"); // sort(1, 5, 3, 6, 8, 4)
    assert_eq!(lines.next().unwrap(), "0 1 2"); // inc() inc() inc()
    assert_eq!(lines.next().unwrap(), "5"); // magnitude({ x: 3, y: 4 })
    assert_eq!(lines.next().unwrap(), "set by JavaScript!"); // set()
    assert_eq!(lines.next().unwrap(), "set by JavaScript!"); // get()
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
