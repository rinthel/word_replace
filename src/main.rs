extern crate mdbook;
extern crate clap;

use mdbook::MDBook;
use clap::{Arg, App, SubCommand};

use std::env;
use std::path::Path;

mod book_preprocess;
use book_preprocess::*;

fn main() {
    let matches = App::new("mdBook with dictionary replacement")
        .version("0.1.0")
        .author("Rinthel Kwon")
        .subcommand(SubCommand::with_name("build")
            .about("build documents"))
        .get_matches();

    // println!("current path: {}", env::current_dir().unwrap().display());
    // println!("{:?}", matches);
    let toml_value = read_dictionary_toml_file(Path::new("dic.toml"));
    let dictionary_map = get_dictionary(&toml_value, "ko").expect("cannot read dictionary");
    let suffix_pairs = get_suffix_pairs(&toml_value, "ko").expect("cannot read suffix");

    println!("suffix pairs: {:?}", suffix_pairs);

    // open dir and scan all .md files
    {        
        std::fs::create_dir_all("example/src").expect("cannot create src_temp directory");
        let files = std::fs::read_dir(Path::new("example/src_pre")).expect("failed to read dir");
        for file in files {
            let f = file.expect("failed to get file");
            let src_filepath = f.path();
            let dst_filepath = Path::new("example/src").join(src_filepath.strip_prefix("example/src_pre").unwrap());
            process_file(&src_filepath, &dst_filepath, &dictionary_map);
        }
    }

    let mut book = MDBook::new(Path::new("example"))
        .set_src(Path::new("src"))
        .set_dest(Path::new("book"))
        .read_config();
    book.build().unwrap();
}
