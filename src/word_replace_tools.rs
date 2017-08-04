extern crate toml;
extern crate regex;

use std::collections::HashMap;
use std::fs::copy as file_copy;
use std::fs::create_dir_all;
use std::fs::read_dir;
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
pub struct PostPosPair {
    pub left: String,
    pub right: String,
}

pub trait PostPosPairArrayInterface {
    fn find(&self, s: &str) -> Option<&PostPosPair>;
}

impl PostPosPairArrayInterface for Vec<PostPosPair> {
    fn find(&self, s: &str) -> Option<&PostPosPair> {
        for pair in self {
            if pair.left == s || pair.right == s {
                return Some(&pair);
            }
        }
        None
    }
}

fn is_korean_character_and_has_final_jamo(c: char) -> (bool, bool) {
    let code_point = c as u32;
    let is_korean_character = code_point >= 0xAC00 && code_point <= 0xD7A3;
    if is_korean_character {
        (true, (code_point - 0xAC00) % 28 != 0)
    }
    else {
        (false, false)
    }
}

pub fn get_postpos_pairs(toml_value: &Value, language: &str) -> Option<Vec<PostPosPair>> {
    let mut postpos_pairs = Vec::<PostPosPair>::new();
    if let &Value::Table(ref all_tables) = toml_value {
        match all_tables.get(&(String::from(language) + "-postpos")) {
            Some(postpos_contents) => {
                let postpos_table = match postpos_contents { &Value::Table(ref t) => Some(t), _ => None }.unwrap();
                let postpos_pairs_toml = match postpos_table.get("postpos").unwrap() { &Value::Array(ref t) => Some(t), _ => None}.unwrap();
                for pair in postpos_pairs_toml {
                    let p = match pair { &Value::Array(ref t) => Some(t), _ => None}.unwrap();
                    let left = match p.get(0).unwrap() { &Value::String(ref s) => Some(s), _ => None}.unwrap();
                    let right = match p.get(1).unwrap() { &Value::String(ref s) => Some(s), _ => None}.unwrap();
                    postpos_pairs.push(PostPosPair { left:left.clone(), right:right.clone(), });
                }
            }
            None => {
                return None;
            }
        }
        Some(postpos_pairs)
    }
    else {
        None
    }
}

pub fn process_file(src_filepath: &Path,
    dst_filepath: &Path,
    dictionary_map: &HashMap<String, String>,
    postpos_pairs_option: Option<&Vec<PostPosPair>>,
    show_warning: bool) {
    let re = Regex::new(r"@@[a-z|A-Z|\d]+@@").unwrap();

    let mut src_file = File::open(src_filepath).expect("failed to open file");
    let mut src_string = String::new();
    let src_read_result = src_file.read_to_string(&mut src_string);
    match src_read_result {
        Err(_) => {
            if show_warning {
                println!("{} is not a text file, so just copied.", src_filepath.to_str().unwrap());
                file_copy(src_filepath, dst_filepath).expect("failed to copy file");
                return;
            }
        }
        _ => {},
    }
    
    let mut dst_string = String::new();

    let mut last_index = 0;
    for c in re.captures_iter(&src_string) {
        let m = c.get(0).unwrap();
        let key = &src_string[m.start()+2..m.end()-2];
        let mut additional_advance = 0;
        dst_string += &src_string[last_index..m.start()];
        match dictionary_map.get(key) {
            Some(word) => {
                dst_string += word;
                if let Some(postpos_pairs) = postpos_pairs_option {
                    let next = &src_string[m.end()..].split_whitespace().next();
                    match *next {
                        Some(postpos) => {
                            let found_postpos_pair = postpos_pairs.find(postpos);
                            let (word_is_korean, word_has_final_jamo) =
                                is_korean_character_and_has_final_jamo(word.chars().last().unwrap());
                            let (postpos_is_korean, _) = 
                                is_korean_character_and_has_final_jamo(postpos.chars().next().unwrap());
                            if word_is_korean {
                                if let Some(found_postpos_pair) = found_postpos_pair {
                                    dst_string += if word_has_final_jamo
                                        { found_postpos_pair.right.as_str() } else { found_postpos_pair.left.as_str() };
                                    additional_advance = postpos.len();
                                }
                                else {
                                    if postpos_is_korean && show_warning {
                                        println!("warning: undefined korean postposition at {}: {}{} => {}{}",
                                            src_filepath.to_str().unwrap(), m.as_str(), postpos,
                                            word, postpos);
                                    }
                                }
                            }
                        },
                        None => {},
                    };
                }
            },
            _ => {
                dst_string += m.as_str();
            },
        };
        last_index = m.end();
        last_index += additional_advance;
    }
    dst_string += &src_string[last_index..];

    let mut dst_file = File::create(dst_filepath).expect("failed to open destination file");
    dst_file.write_all(dst_string.as_bytes()).expect("failed to write destination file");
}


pub fn process_directory(input_directory: &Path,
    output_directory: &Path,
    dictionary_map: &HashMap<String, String>,
    postpos_pairs_option: Option<&Vec<PostPosPair>>,
    show_warning: bool) {
    create_dir_all(&output_directory).expect("cannot create output directory");
    let paths = read_dir(&input_directory).expect("failed to read input directory");
    for p in paths {
        let src_path = p.expect("failed to get source file path").path();
        let dst_path = output_directory.join(src_path.strip_prefix(&input_directory).unwrap());
        if src_path.is_dir() {
            process_directory(&src_path, &dst_path, dictionary_map, postpos_pairs_option, show_warning);
        }
        else {
            if show_warning {
                println!("processing: {}", src_path.to_str().unwrap());
            }
            process_file(&src_path, &dst_path, &dictionary_map, postpos_pairs_option, show_warning);
        }
    }
}
