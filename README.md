# Watchmen

`
Watchmen 是一个守护进程管理器，可为您全天候管理和保持应用程序在线
`

[中文简体](README.md) | [English](README_EN.md)

## 二进制文件

watchmen

用于执行命令的命令行界面

daemon

使应用程序保持在线的守护进程

guard (非必须)

防止 daemon 意外退出的守护进程

## 命令 

`watchmen [OPTIONS|SUBCOMMAND] ...`

### 选项或子命令
| Option                   | Description          |
| ------------------------ | -------------------- |
| -h, --help               | 帮助信息             |
| -v, --version            | 版本信息             |
| -i, --info               | 软件信息             |
| -d, --daemon             | 启动守护进程         |
| -t, --terminated         | 终止守护进程         |
| -gd, --guard-daemon      | 启动被守护的守护进程 |
| -gt, --guardt-terminated | 终止被守护的守护进程 |
| run [oprions...]         | 创建并运行任务       |
| add [oprions...]         | 添加任务             |
| drop [oprions...]        | 停止并删除任务       |
| start [oprions...]       | 开始任务             |
| restart [oprions...]     | 重启任务             |
| stop [oprions...]        | 停止任务             |
| pause [oprions...]       | 暂停定时任务         |
| resume [oprions...]      | 继续定时任务         |
| list [oprions...]        | 查看任务             |


## 子命令

### run

`watchmen run [OPTIONS] ...`

| Option         | Description     |
| -------------- | --------------- |
| -h, --help     | 帮助信息        |
| -n, --name     | 任务名          |
| -o, --origin   | 任务开始时间    |
| -i, --interval | 任务执行周期 ms |

### add

`watchmen add [OPTIONS] ...`

| Option         | Description     |
| -------------- | --------------- |
| -h, --help     | 帮助信息        |
| -n, --name     | 任务名          |
| -o, --origin   | 任务开始时间    |
| -i, --interval | 任务执行周期 ms |

### drop

`watchmen drop [OPTIONS] ...`

| Option     | Description |
| ---------- | ----------- |
| -h, --help | 帮助信息    |
| -n, --name | 任务名      |
| -p, --pid  | 任务 pid    |

### start

`watchmen start [OPTIONS] ...`

| Option     | Description |
| ---------- | ----------- |
| -h, --help | 帮助信息    |
| -n, --name | 任务名      |
| -p, --pid  | 任务 pid    |

### restart

`watchmen restart [OPTIONS] ...`

| Option     | Description |
| ---------- | ----------- |
| -h, --help | 帮助信息    |
| -i, --name | 任务信息    |
| -n, --pid  | 任务名      |

### stop

`watchmen stop [OPTIONS] ...`

| Option     | Description |
| ---------- | ----------- |
| -h, --help | 帮助信息    |
| -n, --name | 任务名      |
| -p, --pid  | 任务 pid    |

### pause

`watchmen stop [OPTIONS] ...`

| Option     | Description |
| ---------- | ----------- |
| -h, --help | 帮助信息    |
| -n, --name | 任务名      |
| -p, --pid  | 任务 pid    |

### resume

`watchmen stop [OPTIONS] ...`

| Option     | Description |
| ---------- | ----------- |
| -h, --help | 帮助信息    |
| -n, --name | 任务名      |

### list

`watchmen list [OPTIONS] ...`

| Option       | Description |
| ------------ | ----------- |
| -h, --help   | 帮助信息    |
| -n, --name   | 任务名      |
| -s, --status | 任务状态    |
| -p, --pid    | 任务 pid    |
| -m, --more   | 更过信息    |

## 输出文件

默认输出路径: /tmp/watchmen (或 读取环境变量: WATCHMEN_PATH)
|--/tmp/watchmen/
    |--stdout/
        |--[task name].log ==> 任务日志
    |--daemon_stdout.log ==> daemon 进程输出日志
    |--daemon_stderr.log ==> daemon 进程错误日志
    |--guard.log ==> guard 进程输出日志
    |--tasks.json ==> 所有任务列表
    |--daemon.pid ==> daemon 进程 pid
    |--guard.pid => guard 进程 id
    |--watchmen.sock ==> watchmen daemon 通信 sock 文件

## 从源码构建并运行示例程序

```bash
# 下载源码
git clone https://git.ahriknow.com/ahriknow/watchmen
cd watchmen
cargo build --release

# 启动守护进程
./target/release/watchmen -d
Start daemon pid: 65535

# 查询任务
./target/release/watchmen list
------------------------------------------------------------------------------------
| ID | NAME | STATUS | PID | STARTED_AT          | STOPPED_AT          | EXIT_CODE |
------------------------------------------------------------------------------------
0 Total: 0 running, 0 stopped, 0 waiting

# 创建并运行一个任务
./watchmen run -n test sh ${watchmen_project_path}/script/task.sh

# 查询任务
./target/release/watchmen list
------------------------------------------------------------------------------------------------
| ID            | NAME | STATUS  | PID | STARTED_AT          | STOPPED_AT          | EXIT_CODE |
------------------------------------------------------------------------------------------------
| 1663924559448 | test | running | 399 | 2022-01-01 00:00:00 |                     |           |
------------------------------------------------------------------------------------------------
1 Total: 1 running, 0 stopped, 0 waiting

# 停止任务
./watchmen stop test

# 查询任务
./target/release/watchmen list
------------------------------------------------------------------------------------------------
| ID            | NAME | STATUS  | PID | STARTED_AT          | STOPPED_AT          | EXIT_CODE |
------------------------------------------------------------------------------------------------
| 1663924559448 | test | stopped | 0   | 2022-01-01 00:00:00 | 2022-01-01 00:00:05 | 0         |
------------------------------------------------------------------------------------------------
1 Total: 0 running, 1 stopped, 0 waiting

# 删除任务
./watchmen drop test

# 查询任务
./target/release/watchmen list
------------------------------------------------------------------------------------
| ID | NAME | STATUS | PID | STARTED_AT          | STOPPED_AT          | EXIT_CODE |
------------------------------------------------------------------------------------
0 Total: 0 running, 0 stopped, 0 waiting

# 停止守护进程
./target/release/watchmen -t
Terminated daemon pid: 65535

# 查看输出
# 默认输出路径是 /tmp/watchmen (或 读取环境变量: WATCHMEN_PATH)
ls /tmp/watchmen
-rw-r--r-- 1 user user    0 Sep 01 00:00 daemon_stdout.log
-rw-r--r-- 1 user user    0 Sep 01 00:00 daemon_stderr.log
drwxr-xr-x 2 user user 4096 Sep 01 00:00 stdout
-rw-r--r-- 1 user user    5 Sep 01 00:00 daemon.pid
srwxr-xr-x 1 user user    0 Sep 01 00:00 watchmen.sock

ls /tmp/watchmen/stdout
-rw-r--r-- 1 user user 130 Sep 01 00:00 test.log

cat /tmp/watchmen/stdout/test.log
Result from shell task: 1
Result from shell task: 2
Result from shell task: 3
Result from shell task: 4
Result from shell task: 5
```

## License Apache Licence 2.0
[License](./LICENSE)

## Copyright ahriknow 2022
