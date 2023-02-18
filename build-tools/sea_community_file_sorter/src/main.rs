//! The MSRV is: 1.56.1
//!
//! This file to to manage the `COMMUNITY.md` file, which is a list of all the community projects and resources.
//! The community file structure is as follows:
//! ```md
//! # Community
//! ## Built with SeaORM
//! <!-- Here is the description of the built with SeaORM section -->
//! <!-- Then if the line starts with 3 hashes, it's a new section -->
//! <!-- Then if the line starts with 4 hashes, it's a sub section -->
//! <!-- Then if the line starts with dash, it's a project, project for the section above -->
//! <!-- If found anything else, will return an error -->
//! ## Learning Resources
//! <!-- Anything here will be ignored (for now) -->
//! ```
//! Note: This will not do anything with the `Learning Resources` section for now.
use std::{fs, path::Path};
mod parser;
mod sorter;
use sorter::AlphapeticalSorter;

/// The main function, this will parse the community file and sort it.
/// There are 2 arguments:
/// - The path to the community file
/// - `--check` or `-c` to check if the community file is sorted (optional)
/// Will sort the community file if it's not sorted, will panic if the community file is not in the correct format.
fn main() {
    let argv: Vec<String> = std::env::args().collect();
    let file_path = Path::new(&argv[1]);
    let check = matches!(argv.get(2), Some(arg) if arg == "--check" || arg == "-c");
    let community = parser::Community::parse(file_path);
    if check {
        if let Err(err) = community.check_sorted() {
            eprintln!("{}", err);
            std::process::exit(1);
        } else {
            println!("The community file is sorted");
        }
    } else {
        fs::write(file_path, community.sort().to_string()).expect("Failed to write to file");
        println!("The community file is sorted successfully");
    }
}
