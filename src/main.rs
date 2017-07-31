extern crate mdbook;
extern crate clap;
extern crate toml;
extern crate regex;

use mdbook::MDBook;
use std::env;
use clap::{Arg, App, SubCommand};

use std::collections::HashMap;
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;
use regex::Regex;
use toml::Value;

fn get_dictionary(language: &str) -> Result<HashMap<String, String>, String> {
    let mut dic_file = File::open("dic.toml").expect("could not open dic.toml");
    let mut dic_string = String::new();
    dic_file.read_to_string(&mut dic_string).expect("could not read contents from dic.toml");
    let dic_value = dic_string.parse::<Value>().expect("could not parse dic.toml");

    let mut dictionary_map: HashMap<String, String> = HashMap::new();
    if let Value::Table(dic_table) = dic_value {
        match dic_table.get(language) {
            Some(dic_contents) => {
                if let Value::Table(ref dic) = *dic_contents {
                    for (keyword, translate) in dic {
                        match *translate {
                            Value::String(ref x) => {
                                dictionary_map.insert(keyword.clone(), x.clone());
                            },
                            _ => (),
                        };
                    }
                }
            }
            None => {
                return Err(String::from("cannot find appropriate language from dictionary"));
            }
        }
    }

    Ok(dictionary_map)
}

fn process_file(src_filepath: &Path, dst_filepath: &Path, dictionary_map: &HashMap<String, String>) {
    let re = Regex::new(r"@@[a-z|A-Z|\d]+@@").unwrap();

    let mut src_file = File::open(src_filepath).expect("failed to open file");
    let mut src_string = String::new();
    src_file.read_to_string(&mut src_string).expect("failed to read file");
    let mut dst_string = String::new();

    let mut last_index = 0;
    for c in re.captures_iter(&src_string) {
        let m = c.get(0).unwrap();
        println!("matches: {}, start: {}, end: {}",
            m.as_str(), m.start(), m.end());
        let key = &src_string[m.start()+2..m.end()-2];

        dst_string += &src_string[last_index..m.start()];
        match dictionary_map.get(key) {
            Some(word) => {
                println!("key: {} -> {}", key, word);
                dst_string += word;
            }
            _ => {
                dst_string += m.as_str();
            },
        };
        last_index = m.end();
    }
    dst_string += &src_string[last_index..];

    let mut dst_file = File::create(dst_filepath).expect("failed to open destination file");
    dst_file.write_all(dst_string.as_bytes());
}

fn main() {
    let matches = App::new("mdBook with dictionary replacement")
        .version("0.1.0")
        .author("Rinthel Kwon")
        .subcommand(SubCommand::with_name("build")
            .about("build documents"))
        .get_matches();

    // println!("current path: {}", env::current_dir().unwrap().display());
    // println!("{:?}", matches);

    let dictionary_map = get_dictionary("ko").expect("cannot read dictionary");

    // open dir and scan all .md files
    {
        use std::fs::DirEntry;
        use std::fs::ReadDir;
        use std::path::Path;
        use std::fs::File;
        use regex::Regex;

        let re = Regex::new(r"@@[a-z|A-Z|\d]+@@").unwrap();
        
        std::fs::create_dir_all("example/temp").expect("cannot create src_temp directory");
        let files = std::fs::read_dir(Path::new("example/src")).expect("failed to read dir");
        for file in files {
            let f = file.expect("failed to get file");
            let src_filepath = f.path();
            let dst_filepath = Path::new("example/temp").join(src_filepath.strip_prefix("example/src").unwrap());
            process_file(&src_filepath, &dst_filepath, &dictionary_map);
        }
    }

    let mut book = MDBook::new(Path::new("example"))
        .set_src(Path::new("src_temp"))
        .set_dest(Path::new("book"))
        .read_config();
    book.build().unwrap();
}
