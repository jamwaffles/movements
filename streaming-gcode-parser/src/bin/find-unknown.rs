//! Find words that are parsed but are unknown.

use glob::glob;
use std::fs::read_to_string;

use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;
use streaming_gcode_parser::{Program, Statement};

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
    // let filter = std::env::args().nth(1);
    // let out_dir = env::var("OUT_DIR").unwrap();

    // let suite_names: Vec<String> = read_dir("../test_files/")
    //     .unwrap()
    //     .map(|full_path| String::from(full_path.unwrap().file_name().to_str().unwrap()))
    //     .collect();

    //     let files = suite_names.map(|n| {

    //     })

    // for suite_name in suite_names {
    //     let suite_files = read_files_recursive(&format!("../test_files/{}", suite_name));

    //     for suite_file in suite_files {
    //         write_test(&mut test_file, &suite_name, suite_file);
    //     }
    // }

    let mut files = HashMap::new();
    let mut all = HashMap::new();
    let mut failures = Vec::new();

    let paths = read_files_recursive(&"../test_files/");

    for (path, _case_name) in paths.iter() {
        println!("Searching {:?}...", path);

        let mut found = HashMap::new();

        if let Ok((_remaining, parsed)) = Program::parse_complete(&read_to_string(path).unwrap()) {
            let blocks = parsed.blocks;

            for token in blocks.into_iter().map(|b| b.words.into_iter()).flatten() {
                let statement = token.statement;

                if let Statement::Dynamic { letter, number } = statement {
                    let file_found = found.entry(letter).or_insert_with(HashSet::new);
                    let all_entry = all.entry(letter).or_insert_with(HashSet::new);

                    file_found.insert(number.to_string());
                    all_entry.insert(number.to_string());
                }
            }
        } else {
            failures.push(path);
        }

        for (letter, _value) in found.iter() {
            println!("    {}", letter);
        }

        files.insert(path, found);
    }

    println!("\nFailures\n");

    for path in failures.iter() {
        println!("    {:?}", path);
    }

    println!("\nSummary");

    for (letter, items) in all.iter() {
        println!();
        println!("    {}", letter);

        for item in items.iter().take(10) {
            println!("      {}", item);
        }
    }
    println!();
}
