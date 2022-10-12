# Watchmen

`
Watchmen is a daemon process manager that for you manage and keep your application online 24/7
`

[中文简体](README.md) | [English](README_EN.md)

## Binaries

`watchmen`

The cli to execute commands

`daemon`

The daemon process that will keep your application online

`guard` (Not required)

The guard process that will the daemon process online

### Command 

`watchmen [OPTIONS|SUBCOMMAND] ...`

## Options
| Option                   | Description             |
| ------------------------ | ----------------------- |
| -h, --help               | show help               |
| -v, --version            | Show version            |
| -i, --info               | Show info               |
| -d, --daemon             | startup daemon          |
| -t, --terminated         | terminated daemon       |
| -gd, --guard-daemon      | startup guard daemon    |
| -gt, --guardt-terminated | terminated guard daemon |
| run [oprions...]         | create and run task     |
| add [oprions...]         | add task                |
| drop [oprions...]        | stop and drop task      |
| start [oprions...]       | start task              |
| restart [oprions...]     | restart task            |
| stop [oprions...]        | stop task               |
| pause [oprions...]       | pause scheduled task    |
| resume [oprions...]      | resume scheduled task   |
| list [oprions...]        | list tasks              |


## SubCommands

### run

`watchmen run [OPTIONS] ...`

| Option         | Description         |
| -------------- | ------------------- |
| -h, --help     | show help           |
| -n, --name     | task name           |
| -o, --origin   | task start datetime |
| -i, --interval | task time interval  |

### add

`watchmen add [OPTIONS] ...`

| Option         | Description              |
| -------------- | ------------------------ |
| -h, --help     | show help                |
| -n, --name     | task name                |
| -o, --origin   | task start datetime      |
| -i, --interval | task time interval       |
| -t, --timing   | exec time of timing task |

> -o, --origin
> 
> format: YYYYMMDD.HHMMSS | YYYYMMDD | MMDD | MMDD.HHMMSS | HHMMSS
>
> example: 20201231.235959 | 20201231 | 1231 | 1231.235959 | 235959
> 
> input => auto into\
> 20201231.235959 => 20201231.235959\
> 20201231 => 20201231.000000\
> 1231 => [current year]1231.000000\
> 1231.235959 => [current year]1231.235959\
> 235959 => [current year][current month][today].235959

> -i, --interval
> 
> format: 1d2h3m4s5 | 3m4s5 | 4s5 | 5 ...

> -t, --timing
> 
> format: split by ',' YYYYMMDD.HHMMSS | YYYYMMDD | MMDD | MMDD.HHMMSS | HHMMSS
> 
> example: 20210101.000000,20210102.000000,20210103

### drop

`watchmen drop [OPTIONS] ...`

| Option     | Description |
| ---------- | ----------- |
| -h, --help | show help   |
| -n, --name | task name   |
| -p, --pid  | task pid    |

### start

`watchmen start [OPTIONS] ...`

| Option     | Description |
| ---------- | ----------- |
| -h, --help | show help   |
| -n, --name | task name   |
| -p, --pid  | task pid    |

### restart

`watchmen restart [OPTIONS] ...`

| Option     | Description |
| ---------- | ----------- |
| -h, --help | show help   |
| -i, --name | task id     |
| -n, --pid  | task name   |

### stop

`watchmen stop [OPTIONS] ...`

| Option     | Description |
| ---------- | ----------- |
| -h, --help | show help   |
| -n, --name | task name   |
| -p, --pid  | task pid    |

### pause

`watchmen stop [OPTIONS] ...`

| Option     | Description |
| ---------- | ----------- |
| -h, --help | show help   |
| -n, --name | task name   |
| -p, --pid  | task pid    |

### resume

`watchmen stop [OPTIONS] ...`

| Option     | Description |
| ---------- | ----------- |
| -h, --help | show help   |
| -n, --name | task name   |

### list

`watchmen list [OPTIONS] ...`

| Option       | Description |
| ------------ | ----------- |
| -h, --help   | show help   |
| -n, --name   | task name   |
| -s, --status | task status |
| -p, --pid    | task pid    |
| -m, --more   | more info   |

## Task status

- added: task added
- running: task running
- stopped: task stopped
- interval: task interval
- paused: task paused

## Output files

Default output directory: /tmp/watchmen (OR environment name: WATCHMEN_PATH)
|--/tmp/watchmen/
    |--stdout/
        |--[task name].log ==> tasks log
    |--daemon_stdout.log ==> daemon process output log
    |--daemon_stderr.log ==> daemon process error log
    |--guard.log ==> guard process output log
    |--tasks.json ==> all tasks list
    |--daemon.pid ==> daemon process id
    |--guard.pid => guard process id
    |--watchmen.sock ==> sock file of watchmen / daemon process

## Build and run Examples

```bash
# download source code
git clone https://git.ahriknow.com/ahriknow/watchmen
cd watchmen
cargo build --release

# start the daemon
./target/release/watchmen -d
Start daemon pid: 65535

# show tasks
./target/release/watchmen list
------------------------------------------------------------------------------------
| ID | NAME | STATUS | PID | STARTED_AT          | STOPPED_AT          | EXIT_CODE |
------------------------------------------------------------------------------------
0 Total: 0 running, 0 stopped, 0 waiting

# create a task
./watchmen run -n test sh ${watchmen_project_path}/script/task.sh

# show tasks
./target/release/watchmen list
------------------------------------------------------------------------------------------------
| ID            | NAME | STATUS  | PID | STARTED_AT          | STOPPED_AT          | EXIT_CODE |
------------------------------------------------------------------------------------------------
| 1663924559448 | test | running | 399 | 2022-01-01 00:00:00 |                     |           |
------------------------------------------------------------------------------------------------
1 Total: 1 running, 0 stopped, 0 waiting

# stop the task
./watchmen stop test

# show tasks
./target/release/watchmen list
------------------------------------------------------------------------------------------------
| ID            | NAME | STATUS  | PID | STARTED_AT          | STOPPED_AT          | EXIT_CODE |
------------------------------------------------------------------------------------------------
| 1663924559448 | test | stopped | 0   | 2022-01-01 00:00:00 | 2022-01-01 00:00:05 | 0         |
------------------------------------------------------------------------------------------------
1 Total: 0 running, 1 stopped, 0 waiting

# drop the task
./watchmen drop test

# show tasks
./target/release/watchmen list
------------------------------------------------------------------------------------
| ID | NAME | STATUS | PID | STARTED_AT          | STOPPED_AT          | EXIT_CODE |
------------------------------------------------------------------------------------
0 Total: 0 running, 0 stopped, 0 waiting

# terminated the daemon
./target/release/watchmen -t
Terminated daemon pid: 65535

# show output
# the default output path is /tmp/watchmen (or environment: WATCHMEN_PATH)
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
