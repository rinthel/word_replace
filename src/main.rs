#[macro_use]
extern crate clap;

mod book_preprocess;
use book_preprocess::*;

// use std::env;
use std::path::Path;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");

fn main() {
    let matches = clap_app!(myapp =>
        (version: VERSION)
        (author: AUTHORS)
        (about: "word replacement tool with toml-based dictionary")
        (@arg INPUT: "Set a source directory. default: src_pre")
        (@arg OUTPUT: "Set a destination directory. default: src")
        (@arg warning: -w --warning "Shows all warnings.")
        (@arg dictfile: -d +takes_value "Sets a dictionary file. default: dict.toml")
        (@arg language: -l +takes_value "Sets a language. default: ko")
        (@arg root: -r +takes_value "Sets a root directory. default: .")
    ).get_matches();

    let root_directory = Path::new(matches.value_of("root").unwrap_or("."));
    let input_directory = root_directory.join(Path::new(matches.value_of("INPUT").unwrap_or("src_pre")));
    let output_directory = root_directory.join(Path::new(matches.value_of("OUTPUT").unwrap_or("src")));
    let dictfile = root_directory.join(Path::new(matches.value_of("dictfile").unwrap_or("dict.toml")));
    let language = matches.value_of("language").unwrap_or("ko");
    let show_warning = matches.is_present("warning");

    if !input_directory.is_dir() {
        eprintln!("error: INPUT is not a directory.");
        return;
    }

    if !dictfile.is_file() {
        eprintln!("error: cannot find a dictionary file: {}", dictfile.to_str().unwrap());
        return;
    }

    // println!("current path: {}", env::current_dir().unwrap().display());
    // println!("{:?}", matches);
    let toml_value = read_dictionary_toml_file(&dictfile);
    let dictionary_map = get_dictionary(&toml_value, language).expect("cannot read dictionary");
    let suffix_pairs = get_suffix_pairs(&toml_value, language);
    let suffix_pairs_option = match suffix_pairs {
        Some(ref s) => Some(s),
        None => None,
    };

    // open dir and scan all .md files
    std::fs::create_dir_all(&output_directory).expect("cannot create output directory");
    let files = std::fs::read_dir(&input_directory).expect("failed to read input directory");
    for file in files {
        let f = file.expect("failed to get source file");
        let src_filepath = f.path();
        let dst_filepath = output_directory.join(src_filepath.strip_prefix(&input_directory).unwrap());
        if show_warning {
            println!("processing: {}", src_filepath.to_str().unwrap());
        }
        process_file(&src_filepath, &dst_filepath, &dictionary_map, suffix_pairs_option, show_warning);
    }
}
