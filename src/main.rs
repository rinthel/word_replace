extern crate clap;

mod book_preprocess;
use book_preprocess::*;

use std::path::Path;
use clap::{Arg, App};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");

fn main() {
    let matches = App::new("mdbook with dictionary - word replacement")
        .version(VERSION)
        .author(AUTHORS)
        .arg(Arg::with_name("INPUT"))
        .arg(Arg::with_name("LANGUAGE"))
        .get_matches();

    // println!("current path: {}", env::current_dir().unwrap().display());
    // println!("{:?}", matches);
    let toml_value = read_dictionary_toml_file(Path::new("example/dict.toml"));
    let dictionary_map = get_dictionary(&toml_value, "ko").expect("cannot read dictionary");
    let suffix_pairs = get_suffix_pairs(&toml_value, "ko").expect("cannot read suffix");

    // open dir and scan all .md files
    std::fs::create_dir_all("example/src").expect("cannot create src_temp directory");
    let files = std::fs::read_dir(Path::new("example/src_pre")).expect("failed to read dir");
    for file in files {
        let f = file.expect("failed to get file");
        let src_filepath = f.path();
        let dst_filepath = Path::new("example/src").join(src_filepath.strip_prefix("example/src_pre").unwrap());
        process_file(&src_filepath, &dst_filepath, &dictionary_map, &suffix_pairs);
    }
}
