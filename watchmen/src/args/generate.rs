use std::{error::Error, ffi::OsStr, path::PathBuf};

static CONFIG: &str = r#"[watchmen]
# The engine to use for the watchmen server
# Valid values are "sock", "socket", "redis"
# sock: Unix socket
# socket: TCP socket
# redis: Redis pub/sub
engines = ["sock"]

# The standard output of the watchmen server
# Default is None if not set
stdout = "$HOME/.watchmen/watchmen.stdout.log"

# The standard error of the watchmen server
# Default is None if not set
stderr = "$HOME/.watchmen/watchmen.stderr.log"

# The pid file of the watchmen server
# Default is `$HOME/.watchmen/watchmen.pid` if not set
pid = "$HOME/.watchmen/watchmen.pid"


[sock]
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
        let home: PathBuf = dirs::home_dir().unwrap();
        home.join(".watchmen/config.toml")
    } else {
        let path: PathBuf = PathBuf::from(path);
        if path.is_dir() {
            path.join("config.toml")
        } else {
            let ext: &OsStr = path.extension().unwrap();
            if ext == "toml" {
                path
            } else {
                return Err("Config file must be toml".into());
            }
        }
    };
    let parent = path.parent().unwrap();
    std::fs::create_dir_all(parent)?;
    std::fs::write(path, CONFIG)?;
    Ok(())
}
