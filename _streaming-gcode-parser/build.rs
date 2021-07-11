use glob::glob;
use std::env;
use std::fs::read_dir;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

fn read_files_recursive(root: &str) -> Vec<(PathBuf, String)> {
    glob(&format!("{}/**/*", root))
        .expect("Failed to read glob pattern")
        .filter_map(|item| {
            let path = item.expect("Failed to get path");

            if path.is_file() {
                Some(path)
            } else {
                None
            }
        })
        .map(|path| {
            let filename = String::from(path.strip_prefix(root).unwrap().to_str().unwrap())
                .to_lowercase()
                .replace("/", "_")
                .replace(".", "_")
                .replace("-", "_")
                .replace(" ", "_");

            (path, filename)
        })
        .collect()
}

// build script's entry point
fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let destination = Path::new(&out_dir).join("test_suites.rs");
    let mut test_file = File::create(&destination).unwrap();

    write_header(&mut test_file);

    let suite_names: Vec<String> = read_dir("../test_files/")
        .unwrap()
        .map(|full_path| String::from(full_path.unwrap().file_name().to_str().unwrap()))
        .collect();

    println!("Generating tests for suites: {:?}", suite_names);

    for suite_name in suite_names {
        let suite_files = read_files_recursive(&format!("../test_files/{}", suite_name));

        for suite_file in suite_files {
            write_test(&mut test_file, &suite_name, suite_file);
        }
    }
}

fn write_test(
    test_file: &mut File,
    suite_name: &str,
    (source_data_path, data_file_name): (PathBuf, String),
) {
    let test_name = format!("{}_{}", suite_name, data_file_name);

    write!(
        test_file,
        include_str!("./tests/file_suite_template"),
        name = test_name,
        source_data_path = source_data_path.canonicalize().unwrap().display()
    )
    .unwrap();
}

fn write_header(test_file: &mut File) {
    write!(
        test_file,
        r#"
use streaming_gcode_parser::Program;
"#
    )
    .unwrap();
}
