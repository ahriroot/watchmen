use std::{error::Error, path::PathBuf, fs};

use colored::Colorize;
use common::{config::Config, handle::Response};
use regex::Regex;

pub fn get_config(path: Option<String>) -> Result<Config, Box<dyn Error>> {
    let path: PathBuf = if let Some(path) = path {
        PathBuf::from(path)
    } else {
        let home: PathBuf = dirs::home_dir().unwrap();
        let config_path = home.join(".watchmen/config.toml");
        if config_path.exists() {
            config_path
        } else {
            // /etc/watchmen/config.toml
            let config_path = PathBuf::from("/etc/watchmen/config.toml");
            if config_path.exists() {
                config_path
            } else {
                return Err("No config file found".into());
            }
        }
    };
    let config: Config = path.into();
    Ok(config)
}

pub fn get_with_home_path(input: String) -> PathBuf {
    let home_dir = dirs::home_dir().unwrap();
    let home_dir_str = home_dir.to_string_lossy().into_owned();
    let replaced = input.replace("$HOME", &home_dir_str);
    PathBuf::from(replaced)
}

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
