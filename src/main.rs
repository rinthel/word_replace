extern crate mdbook;
extern crate clap;
extern crate toml;

use mdbook::MDBook;
use std::path::Path;
use std::env;
use clap::{Arg, App, SubCommand};

use std::fs::File;
use std::io::prelude::*;
use toml::Value;

fn main() {
    let matches = App::new("mdBook with dictionary replacement")
        .version("0.1.0")
        .author("Rinthel Kwon")
        .subcommand(SubCommand::with_name("build")
            .about("build documents"))
        .get_matches();

    println!("current path: {}", env::current_dir().unwrap().display());
    println!("{:?}", matches);

    let mut dic_file = File::open("dic.toml").expect("could not open dic.toml");
    let mut dic_string = String::new();
    dic_file.read_to_string(&mut dic_string).expect("could not read contents from dic.toml");
    let dic_value = dic_string.parse::<Value>().expect("could not parse dic.toml");
    println!("dic: {:?}", dic_value);

    if let Value::Table(dic_table) = dic_value {
        println!("dic_table: {:?}", dic_table);
        for (dic_language, dic_contents) in &dic_table {
            println!("\tlanguage: {}", dic_language);
            if let Value::Table(ref dic) = *dic_contents {
                for (keyword, translate) in dic {
                    println!("\t\t{} => {:?}", keyword, match *translate { Value::String(ref x) => x, _ => "illegel"} );
                }
            }
        }
    }

    let mut book = MDBook::new(Path::new("example"))
        .set_src(Path::new("src"))
        .set_dest(Path::new("book"))
        .read_config();
    book.build().unwrap();
}
