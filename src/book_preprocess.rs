extern crate toml;
extern crate regex;

use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use self::toml::Value;
use self::regex::Regex;

pub fn read_dictionary_toml_file(filename: &Path) -> Value {
    let mut dic_file = File::open(filename).expect("could not open dic.toml");
    let mut dic_string = String::new();
    dic_file.read_to_string(&mut dic_string).expect("could not read contents from dic.toml");
    dic_string.parse::<Value>().expect("could not parse dic.toml")
}

pub fn get_dictionary(toml_value: &Value, language: &str) -> Result<HashMap<String, String>, &'static str> {
    let mut dictionary_map: HashMap<String, String> = HashMap::new();
    if let &Value::Table(ref dic_table) = toml_value {
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
                return Err("cannot find appropriate language from dictionary");
            }
        }
        Ok(dictionary_map)
    }
    else {
        Err("cannot find toml tables")
    }
}

#[derive(Debug,Clone)]
pub struct SuffixPair {
    pub left: String,
    pub right: String,
}

pub trait SuffixPairArrayInterface {
    fn find(&self, s: &str) -> Option<&SuffixPair>;
}

impl SuffixPairArrayInterface for Vec<SuffixPair> {
    fn find(&self, s: &str) -> Option<&SuffixPair> {
        for pair in self {
            if pair.left == s || pair.right == s {
                return Some(&pair);
            }
        }
        None
    }
}

pub fn get_suffix_pairs(toml_value: &Value, language: &str) -> Result<Vec<SuffixPair>, &'static str> {
    let mut suffix_pairs = Vec::<SuffixPair>::new();
    if let &Value::Table(ref all_tables) = toml_value {
        match all_tables.get(&(String::from(language) + "-suffix")) {
            Some(suffix_contents) => {
                let suffix_table = match suffix_contents { &Value::Table(ref t) => Some(t), _ => None }.unwrap();
                let suffix_pairs_toml = match suffix_table.get("suffix").unwrap() { &Value::Array(ref t) => Some(t), _ => None}.unwrap();
                for pair in suffix_pairs_toml {
                    let p = match pair { &Value::Array(ref t) => Some(t), _ => None}.unwrap();
                    let left = match p.get(0).unwrap() { &Value::String(ref s) => Some(s), _ => None}.unwrap();
                    let right = match p.get(1).unwrap() { &Value::String(ref s) => Some(s), _ => None}.unwrap();
                    suffix_pairs.push(SuffixPair { left:left.clone(), right:right.clone(), });
                }
            }
            None => {
                return Err("cannot find appropriate language from dictionary");
            }
        }
        Ok(suffix_pairs)
    }
    else {
        Err("cannot find toml tables")
    }
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
        match dictionary_map.get(key) {
            Some(word) => {
                dst_string += word;
                let mut next = &src_string[m.end()..].split_whitespace().next();
                match *next {
                    Some(suffix) => {
                        println!(" suffix found: {}", suffix);
                    },
                    None => {},
                };
            },
            _ => {
                dst_string += m.as_str();
            },
        };
        last_index = m.end();
    }
    dst_string += &src_string[last_index..];

    let mut dst_file = File::create(dst_filepath).expect("failed to open destination file");
    dst_file.write_all(dst_string.as_bytes()).expect("failed to write destination file");
}
