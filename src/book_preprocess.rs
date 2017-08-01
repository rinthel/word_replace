extern crate toml;
extern crate regex;

use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use self::toml::Value;
use self::regex::Regex;

pub fn get_dictionary(language: &str) -> Result<HashMap<String, String>, String> {
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

pub fn process_file(src_filepath: &Path, dst_filepath: &Path, dictionary_map: &HashMap<String, String>) {
    let re = Regex::new(r"@@[a-z|A-Z|\d]+@@").unwrap();

    let mut src_file = File::open(src_filepath).expect("failed to open file");
    let mut src_string = String::new();
    src_file.read_to_string(&mut src_string).expect("failed to read file");
    let mut dst_string = String::new();

    let mut last_index = 0;
    for c in re.captures_iter(&src_string) {
        let m = c.get(0).unwrap();
        let key = &src_string[m.start()+2..m.end()-2];

        dst_string += &src_string[last_index..m.start()];
        dst_string += match dictionary_map.get(key) {
            Some(word) => word,
            _ => m.as_str(),
        };
        last_index = m.end();
    }
    dst_string += &src_string[last_index..];

    let mut dst_file = File::create(dst_filepath).expect("failed to open destination file");
    dst_file.write_all(dst_string.as_bytes()).expect("failed to write destination file");
}