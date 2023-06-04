use std::{error::Error, path::PathBuf};

use common::config::Config;

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
