# Watchmen (0.1.0)

`
Watchmen 是一个守护进程管理器，可为您全天候管理和保持应用程序在线
`

[中文简体](README.md) | [English](README_EN.md)

## 安装

### 源码构建

```shell
# 获取源码
git clone https://git.ahriknow.com/ahriknow/watchmen.git

# 进入项目目录
cd watchmen

# 安装守护进程
cargo install --path watchmend

# 安装 cli 工具
cargo install --path watchmen
```

### 从 crates.io 安装

```shell
# 安装守护进程
cargo install watchmend

# 安装 cli 工具
cargo install watchmen
```

## 开始

### 生成配置文件

> "" 默认位置 ${HOME}/.watchmen/config.toml

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

### 启动守护进程

`watchmend`

### 任务配置文件

```toml
[[task]]
id = 2
name = "Async Task"
command = "command"
args = ["arg1", "arg2"]
dir = "/path/to/directory"
env = { key1 = "value1", key2 = "value2" }
stdin = true
stdout = "output.txt"
stderr = "error.txt"
task_type = { Async = { started_at = 0, stopped_at = 0 } }
```

```ini
[Async Task]
id = 2
name = Async Task
command = command
args = arg1 arg2
dir = /path/to/directory
env = key1=value1 key2=value2
stdin = true
stdout = "output.txt"
stderr = "error.txt"
task_type = async
```

## 命令

### watchmen -h

```shell
Watchmen is a daemon process manager that for you manage and keep your application online 24/7

Usage: watchmen [OPTIONS] [COMMAND]

Commands:
  run      Add and run tasks
  add      Add tasks
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
  -d, --daemon               Start watchmen server
  -w, --guard <GUARD>        Start watchmen server with guard [possible values: true, false]
  -v, --version              Print version
  -h, --help                 Print help
```

### watchmen run -h

```shell
Add and run tasks

Usage: watchmen run [OPTIONS]

Options:
  -p, --path <PATH>        Task config directory
  -m, --mat <MAT>          Task config filename regex pattern [default: ^.*\.(toml|ini|json)$]
  -f, --config <CONFIG>    Task config file
  -n, --name <NAME>        Task name (unique)
  -c, --command <COMMAND>  Task command
  -a, --args <ARGS>        Task arguments
  -d, --dir <DIR>          Task working directory
  -e, --env <ENV>          Task environment variables
  -i, --stdin              Task standard input
  -o, --stdout <STDOUT>    Task standard output
  -r, --stderr <STDERR>    Task standard error
  -h, --help               Print help
```

### watchmen add -h

```shell
Add tasks

Usage: watchmen add [OPTIONS]

Options:
  -p, --path <PATH>        Task config directory
  -m, --mat <MAT>          Task config filename regex pattern [default: ^.*\.(toml|ini|json)$]
  -f, --config <CONFIG>    Task config file
  -n, --name <NAME>        Task name (unique)
  -c, --command <COMMAND>  Task command
  -a, --args <ARGS>        Task arguments
  -d, --dir <DIR>          Task working directory
  -e, --env <ENV>          Task environment variables
  -i, --stdin              Task standard input
  -o, --stdout <STDOUT>    Task standard output
  -r, --stderr <STDERR>    Task standard error
  -h, --help               Print help
```

### watchmen start -h

```shell
Start tasks

Usage: watchmen start [OPTIONS]

Options:
  -p, --path <PATH>        Task config directory
  -m, --pattern <PATTERN>  Task config filename regex pattern [default: ^.*\.(toml|ini|json)$]
  -f, --config <CONFIG>    Task config file
  -n, --name <NAME>        Task name (unique)
  -r, --mat                Is match regex pattern by namae
  -h, --help               Print help
```

### watchmen restart -h

```shell
Restart tasks

Usage: watchmen restart [OPTIONS]

Options:
  -p, --path <PATH>        Task config directory
  -m, --pattern <PATTERN>  Task config filename regex pattern [default: ^.*\.(toml|ini|json)$]
  -f, --config <CONFIG>    Task config file
  -n, --name <NAME>        Task name (unique)
  -r, --mat                Is match regex pattern by namae
  -h, --help               Print help
```

### watchmen stop -h

```shell
Stop tasks

Usage: watchmen stop [OPTIONS]

Options:
  -p, --path <PATH>        Task config directory
  -m, --pattern <PATTERN>  Task config filename regex pattern [default: ^.*\.(toml|ini|json)$]
  -f, --config <CONFIG>    Task config file
  -n, --name <NAME>        Task name (unique)
  -r, --mat                Is match regex pattern by namae
  -h, --help               Print help
```

### watchmen remove -h

```shell
Remove tasks

Usage: watchmen remove [OPTIONS]

Options:
  -p, --path <PATH>        Task config directory
  -m, --pattern <PATTERN>  Task config filename regex pattern [default: ^.*\.(toml|ini|json)$]
  -f, --config <CONFIG>    Task config file
  -n, --name <NAME>        Task name (unique)
  -r, --mat                Is match regex pattern by namae
  -h, --help               Print help
```

### watchmen list -h

```shell
Get tasks list

Usage: watchmen list [OPTIONS]

Options:
  -p, --path <PATH>        Task config directory
  -m, --pattern <PATTERN>  Task config filename regex pattern [default: ^.*\.(toml|ini|json)$]
  -f, --config <CONFIG>    Task config file
  -n, --name <NAME>        Task name (unique)
  -r, --mat                Is match regex pattern by namae
  -h, --help               Print help
```

## License Apache Licence 2.0
[License](./LICENSE)

## Copyright ahriknow 2022
