use std::{error::Error, ffi::OsStr, path::PathBuf};

static CONFIG: &str = r#"[watchmen]
# The engine to use for the watchmen server
# Valid values are "sock", "socket", "http", "redis"
# sock: Unix socket
# socket: TCP socket
# http: HTTP Api (Include Web panel)
# redis: Redis pub/sub
engines = ["sock"]

# The default engine to use for connecting to the watchmen server
engine = "sock"

# The log directory of the watchmen server
log_dir = "$HOME/.watchmen/logs"

# The log level of the watchmen server
# Valid values are "debug", "info", "warn", "error". Default is "info"
log_level = "info"

# The standard output of the watchmen server
# Default is None
stdout = "$HOME/.watchmen/watchmen.stdout.log"

# The standard error of the watchmen server
# Default is None
stderr = "$HOME/.watchmen/watchmen.stderr.log"

# The pid file of the watchmen server
# Default is `$HOME/.watchmen/watchmen.pid`
pid = "$HOME/.watchmen/watchmen.pid"

# The task config file name matching pattern
# Default is `^.*\\.(toml|ini|json)$`
mat = "^.*\\.(toml|ini|json)$"

# Tasks cache file, json format
cache = "$HOME/.watchmen/cache.json"


[sock]
# The unix socket path of the watchmen server
path = "/tmp/watchmen.sock"


[socket]
host = "127.0.0.1"
port = 1949


[http]
host = "127.0.0.1"
port = 1997


[redis]
host = "localhost"
port = 6379
username = ""
password = ""
queue_index = 0
queue_name = "watchmen"
subscribe_channels = ["watchmen"]
subscribe_name = "watchmen"
"#;

pub fn generate(path: &str) -> Result<(), Box<dyn Error>> {
    let path: PathBuf = if path == "" {
        let home: PathBuf = match dirs::home_dir() {
            Some(home) => home,
            None => return Err("Cannot find home directory".into()),
        };
        home.join(".watchmen/config.toml")
    } else {
        let path: PathBuf = PathBuf::from(path);
        if path.is_dir() {
            path.join("config.toml")
        } else {
            let ext: &OsStr = path.extension().unwrap_or(OsStr::new(""));
            if ext == "toml" {
                path
            } else {
                return Err("Config file must be toml".into());
            }
        }
    };
    let parent = match path.parent() {
        Some(parent) => parent,
        None => return Err(format!("Cannot find parent directory of {}", path.display()).into()),
    };
    if !parent.exists() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, CONFIG)?;
    Ok(())
}
