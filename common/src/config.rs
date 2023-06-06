use std::{error::Error, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    pub watchmen: Watchmen,
    pub sock: Sock,
    pub socket: Socket,
    pub http: Http,
    pub redis: Redis,
}

impl Config {
    pub fn init(path: Option<String>) -> Result<Config, Box<dyn Error>> {
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
}

pub fn get_with_home(input: &str) -> String {
    let home_dir = dirs::home_dir().unwrap();
    let home_dir_str = home_dir.to_string_lossy().into_owned();
    if input.starts_with("$HOME") {
        let replaced = input.replace("$HOME", &home_dir_str);
        replaced
    } else if input.starts_with("~/") {
        let replaced = input.replace("~/", &home_dir_str);
        replaced
    } else {
        input.to_string()
    }
}

pub fn get_with_home_path(input: &str) -> PathBuf {
    PathBuf::from(get_with_home(input))
}

impl From<PathBuf> for Config {
    fn from(s: PathBuf) -> Self {
        let config_str = std::fs::read_to_string(s).unwrap();
        let mut config: Config = toml::from_str(&config_str).unwrap();
        if let Some(log_dir) = &config.watchmen.log_dir {
            let log_dir = get_with_home_path(log_dir);
            let parent = log_dir.parent().unwrap();
            if parent.exists() {
                std::fs::create_dir_all(parent).unwrap();
            }
            config.watchmen.log_dir = Some(log_dir.to_str().unwrap().to_string());
        }
        if let Some(log_level) = &config.watchmen.log_level {
            let allowed = ["debug", "info", "warn", "error"];
            if !allowed.contains(&log_level.as_str()) {
                config.watchmen.log_level = Some("info".to_string());
            } else {
                config.watchmen.log_level = Some(log_level.to_string());
            }
        }
        if let Some(stdout) = &config.watchmen.stdout {
            let stdout = get_with_home_path(stdout);
            let parent = stdout.parent().unwrap();
            if parent.exists() {
                std::fs::create_dir_all(parent).unwrap();
            }
            config.watchmen.stdout = Some(stdout.to_str().unwrap().to_string());
        }
        if let Some(stderr) = &config.watchmen.stderr {
            let stderr = get_with_home_path(stderr);
            let parent = stderr.parent().unwrap();
            if parent.exists() {
                std::fs::create_dir_all(parent).unwrap();
            }
            config.watchmen.stderr = Some(stderr.to_str().unwrap().to_string());
        }
        if let Some(pid) = &config.watchmen.pid {
            let pid = get_with_home_path(pid);
            let parent = pid.parent().unwrap();
            if parent.exists() {
                std::fs::create_dir_all(parent).unwrap();
            }
            config.watchmen.pid = Some(pid.to_str().unwrap().to_string());
        }
        if config.watchmen.mat.is_none() {
            config.watchmen.mat = Some(r"^.*\.(toml|ini|json)$".to_string());
        }
        let path = get_with_home_path(&config.sock.path);
        let parent = path.parent().unwrap();
        if parent.exists() {
            std::fs::create_dir_all(parent).unwrap();
        }
        config.sock.path = path.to_str().unwrap().to_string();
        config
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Watchmen {
    pub engine: String,
    pub engines: Vec<String>,
    pub log_dir: Option<String>,
    pub log_level: Option<String>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub pid: Option<String>,
    pub mat: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Sock {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Socket {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Http {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Redis {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub queue_index: u8,
    pub queue_name: String,
    pub subscribe_channels: Vec<String>,
    pub subscribe_name: String,
}
