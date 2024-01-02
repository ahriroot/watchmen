# Watchmen (0.0.1)

`
Watchmen is a daemon process manager that for you manage and keep your application online 24/7
`

[中文简体](README.md) | [English](README_EN.md)

## Install

### Build from source

```shell
# Clone the repository
git clone https://github.com/ahriroot/watchmen.git

# Go into the repository
cd watchmen

# Install watchmen daemon
cargo install --path watchmend

# Install cli tool
cargo install --path watchmen
```

### Install from crates.io

```shell
# Install watchmen daemon
cargo install watchmend

# Install cli tool
cargo install watchmen
```

## Getting Started

### Generate config file

> "" Default is ${HOME}/.watchmen/config.toml

`watchmen -g ""`

```toml
[watchmen]
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
```

### Start watchmen daemon

`watchmend`

### Task Config file

```toml
[[task]]
id = 1
name = "Async Task 1"
command = "command"
args = ["arg1", "arg2"]
dir = "/path/to/directory"
env = { key1 = "value1", key2 = "value2" }
stdin = true
stdout = "output.txt"
stderr = "error.txt"
task_type = { Async = { max_restart = 2, has_restart = 0, started_at = 0, stopped_at = 0 } }

[[task]]
id = 2
name = "Periodic Task 1"
command = "command"
args = ["arg1", "arg2"]
dir = "/path/to/directory"
env = { key1 = "value1", key2 = "value2" }
stdin = false
stdout = "output.txt"
stderr = "error.txt"
task_type = { Periodic = { started_after = 0, interval = 60, last_run = 0, sync = false } }
```

```ini
[Async Task]
id = 1
name = Async Task 1
command = command
args = arg1 arg2
dir = /path/to/directory
env = key1=value1 key2=value2
stdin = true
stdout = "output.txt"
stderr = "error.txt"
task_type = async
max_restart = 2

[Periodic Task]
id = 2
name = Periodic Task 1
command = command
args = arg1 arg2
dir = /path/to/directory
env = key1=value1 key2=value2
stdin = false
stdout = "output.txt"
stderr = "error.txt"
task_type = periodic
started_after = 0
interval = 60
sync = false
```

```json
[
    {
        "id": 1,
        "name": "Async Task 1",
        "command": "command",
        "args": ["arg1", "arg2"],
        "dir": "/path/to/directory",
        "env": {},
        "stdin": true,
        "stdout": "output.txt",
        "stderr": "error.txt",
        "created_at": 0,
        "task_type": { "Async": { "max_restart": 2, "has_restart": 0, "started_at": 0, "stopped_at": 0 } }
    },
    {
        "id": 2,
        "name": "Periodic Task 1",
        "command": "command",
        "args": ["arg1", "arg2"],
        "dir": "/path/to/directory",
        "env": {},
        "stdin": false,
        "stdout": "output.txt",
        "stderr": "error.txt",
        "created_at": 0,
        "task_type": { "Periodic": { "started_after": 0, "interval": 60, "last_run": 0, "sync": false } }
    }
]
```

## Command

### watchmen -h

```shell
Watchmen is a daemon process manager that for you manage and keep your application online 24/7

Usage: watchmen [OPTIONS] [COMMAND]

Commands:
  run      Add and run tasks
  add      Add tasks
  reload   Reload tasks
  start    Start tasks
  restart  Restart tasks
  stop     Stop tasks
  remove   Remove tasks
  list     Get tasks list
  help     Print this message or the help of the given subcommand(s)

Options:
  -c, --config <CONFIG>      Config file path. Default: $HOME/.watchmen/config.toml
  -g, --generate <GENERATE>  Generate config file
  -e, --engine <ENGINE>      Engine for send message [default: sock]
  -v, --version              Print version
  -h, --help                 Print help
```

### watchmen run -h

```shell
Add and run tasks

Usage: watchmen run [OPTIONS]

Options:
  -p, --path <PATH>        Task config directory
  -r, --regex <REGEX>      Task config filename regex pattern [default: ^.*\.(toml|ini|json)$]
  -f, --config <CONFIG>    Task config file
  -n, --name <NAME>        Task name (unique)
  -c, --command <COMMAND>  Task command
  -a, --args <ARGS>        Task arguments
  -d, --dir <DIR>          Task working directory
  -e, --env <ENV>          Task environment variables
  -i, --stdin              Task standard input
  -o, --stdout <STDOUT>    Task standard output
  -w, --stderr <STDERR>    Task standard error
  -h, --help               Print help
```

### watchmen add -h

```shell
Add tasks

Usage: watchmen add [OPTIONS]

Options:
  -p, --path <PATH>        Task config directory
  -r, --regex <REGEX>      Task config filename regex pattern [default: ^.*\.(toml|ini|json)$]
  -f, --config <CONFIG>    Task config file
  -n, --name <NAME>        Task name (unique)
  -c, --command <COMMAND>  Task command
  -a, --args <ARGS>        Task arguments
  -d, --dir <DIR>          Task working directory
  -e, --env <ENV>          Task environment variables
  -i, --stdin              Task standard input
  -o, --stdout <STDOUT>    Task standard output
  -w, --stderr <STDERR>    Task standard error
  -h, --help               Print help
```

### watchmen reload -h

```shell
Reload tasks

Usage: watchmen reload [OPTIONS]

Options:
  -p, --path <PATH>        Task config directory
  -r, --regex <REGEX>      Task config filename regex pattern [default: ^.*\.(toml|ini|json)$]
  -f, --config <CONFIG>    Task config file
  -n, --name <NAME>        Task name (unique)
  -c, --command <COMMAND>  Task command
  -a, --args <ARGS>        Task arguments
  -d, --dir <DIR>          Task working directory
  -e, --env <ENV>          Task environment variables
  -i, --stdin              Task standard input
  -o, --stdout <STDOUT>    Task standard output
  -w, --stderr <STDERR>    Task standard error
  -h, --help               Print help
```

### watchmen start -h

```shell
Start tasks

Usage: watchmen start [OPTIONS]

Options:
  -p, --path <PATH>      Task config directory
  -r, --regex <REGEX>    Task config filename regex pattern [default: ^.*\.(toml|ini|json)$]
  -f, --config <CONFIG>  Task config file
  -i, --id <ID>          Task id (unique)
  -n, --name <NAME>      Task name (unique)
  -m, --mat              Is match regex pattern by namae
  -h, --help             Print help
```

### watchmen restart -h

```shell
Restart tasks

Usage: watchmen restart [OPTIONS]

Options:
  -p, --path <PATH>      Task config directory
  -r, --regex <REGEX>    Task config filename regex pattern [default: ^.*\.(toml|ini|json)$]
  -f, --config <CONFIG>  Task config file
  -i, --id <ID>          Task id (unique)
  -n, --name <NAME>      Task name (unique)
  -m, --mat              Is match regex pattern by namae
  -h, --help             Print help
```

### watchmen stop -h

```shell
Stop tasks

Usage: watchmen stop [OPTIONS]

Options:
  -p, --path <PATH>      Task config directory
  -r, --regex <REGEX>    Task config filename regex pattern [default: ^.*\.(toml|ini|json)$]
  -f, --config <CONFIG>  Task config file
  -i, --id <ID>          Task id (unique)
  -n, --name <NAME>      Task name (unique)
  -m, --mat              Is match regex pattern by namae
  -h, --help             Print help
```

### watchmen remove -h

```shell
Remove tasks

Usage: watchmen remove [OPTIONS]

Options:
  -p, --path <PATH>      Task config directory
  -r, --regex <REGEX>    Task config filename regex pattern [default: ^.*\.(toml|ini|json)$]
  -f, --config <CONFIG>  Task config file
  -i, --id <ID>          Task id (unique)
  -n, --name <NAME>      Task name (unique)
  -m, --mat              Is match regex pattern by namae
  -h, --help             Print help
```

### watchmen pause -h

```shell
Pause interval tasks

Usage: watchmen pause [OPTIONS]

Options:
  -p, --path <PATH>      Task config directory
  -r, --regex <REGEX>    Task config filename regex pattern [default: ^.*\.(toml|ini|json)$]
  -f, --config <CONFIG>  Task config file
  -i, --id <ID>          Task id (unique)
  -n, --name <NAME>      Task name (unique)
  -m, --mat              Is match regex pattern by namae
  -h, --help             Print help
```

### watchmen resume -h

```shell
Resume interval tasks

Usage: watchmen resume [OPTIONS]

Options:
  -p, --path <PATH>      Task config directory
  -r, --regex <REGEX>    Task config filename regex pattern [default: ^.*\.(toml|ini|json)$]
  -f, --config <CONFIG>  Task config file
  -i, --id <ID>          Task id (unique)
  -n, --name <NAME>      Task name (unique)
  -m, --mat              Is match regex pattern by namae
  -h, --help             Print help
```

### watchmen list -h

```shell
Get tasks list

Usage: watchmen list [OPTIONS]

Options:
  -p, --path <PATH>      Task config directory
  -r, --regex <REGEX>    Task config filename regex pattern [default: ^.*\.(toml|ini|json)$]
  -f, --config <CONFIG>  Task config file
  -i, --id <ID>          Task id (unique)
  -n, --name <NAME>      Task name (unique)
  -R, --mat              Is match regex pattern by name
  -m, --more             Show more info
  -l, --less             Show less info
  -h, --help             Print help
```

## Apache License 2.0
[License](./LICENSE)

## Copyright ahriknow 2022
