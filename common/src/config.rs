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

pub fn get_with_home_path(input: String) -> PathBuf {
    let home_dir = dirs::home_dir().unwrap();
    let home_dir_str = home_dir.to_string_lossy().into_owned();
    let replaced = input.replace("$HOME", &home_dir_str);
    PathBuf::from(replaced)
}

impl From<PathBuf> for Config {
    fn from(s: PathBuf) -> Self {
        let config_str = std::fs::read_to_string(s).unwrap();
        let mut config: Config = toml::from_str(&config_str).unwrap();
        if let Some(stdout) = &config.watchmen.stdout {
            if stdout.starts_with("$HOME") {
                let stdout = get_with_home_path(stdout.to_string());
                let parent = stdout.parent().unwrap();
                if parent.exists() {
                    std::fs::create_dir_all(parent).unwrap();
                }
                config.watchmen.stdout = Some(stdout.to_str().unwrap().to_string());
            }
        }
        if let Some(stderr) = &config.watchmen.stderr {
            if stderr.starts_with("$HOME") {
                let stderr = get_with_home_path(stderr.to_string());
                let parent = stderr.parent().unwrap();
                if parent.exists() {
                    std::fs::create_dir_all(parent).unwrap();
                }
                config.watchmen.stderr = Some(stderr.to_str().unwrap().to_string());
            }
        }
        if let Some(pid) = &config.watchmen.pid {
            if pid.starts_with("$HOME") {
                let pid = get_with_home_path(pid.to_string());
                let parent = pid.parent().unwrap();
                if parent.exists() {
                    std::fs::create_dir_all(parent).unwrap();
                }
                config.watchmen.pid = Some(pid.to_str().unwrap().to_string());
            }
        }
        if config.sock.path.starts_with("$HOME") {
            let path = get_with_home_path(config.sock.path.to_string());
            let parent = path.parent().unwrap();
            if parent.exists() {
                std::fs::create_dir_all(parent).unwrap();
            }
            config.sock.path = path.to_str().unwrap().to_string();
        }
        config
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Watchmen {
    pub engines: Vec<String>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub pid: Option<String>,
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
