use std::{error::Error, fs, path::Path};

use crate::common::handle::Response;
use colored::Colorize;
use regex::Regex;

pub fn recursive_search_files(folder_path: &str, regex: &Regex, matched_files: &mut Vec<String>) {
    if let Ok(entries) = fs::read_dir(folder_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();

                if path.is_dir() {
                    recursive_search_files(path.to_str().unwrap(), regex, matched_files);
                } else if let Some(file_name) = path.file_name() {
                    let file_name_str = file_name.to_string_lossy().to_string();
                    if regex.is_match(&file_name_str) {
                        let full_path = path.to_string_lossy().to_string();
                        matched_files.push(full_path);
                    }
                }
            }
        }
    }
}

pub async fn print_result(res: Vec<Response>) {
    for r in res {
        let result = format!("{}\t{}\t{}", r.code, r.msg, r.data.unwrap_or_default());
        match r.code {
            10000 => println!("{}", result.green()),
            40000 => println!("{}", result.yellow()),
            50000 => println!("{}", result.red()),
            _ => println!("{}", result),
        }
    }
}

pub async fn get_ext(path: &Path) -> Result<&str, Box<dyn Error>> {
    let ext = match path.extension() {
        Some(ex) => match ex.to_str() {
            Some(e) => e,
            None => {
                return Err(
                    format!("Cannot convert extension {:?} to string", path.extension()).into(),
                )
            }
        },
        None => {
            return Err(format!("Cannot get extension from {:?}", path).into());
        }
    };
    Ok(ext)
}
