# Watchmen

`
Watchmen is a daemon process manager that for you manage and keep your application online 24/7
`

## Command 

`watchmen [OPTIONS|COMMAND] ...`

## Options
| Option                   | Description             |
| ------------------------ | ----------------------- |
| -h, --help               | Show help               |
| -v, --version            | Show version            |
| -i, --info               | Show info               |
| -d, --daemon             | startup daemon          |
| -gd, --guard-daemon      | startup guard daemon    |
| -t, --terminated         | terminated daemon       |
| -gt, --guardt-terminated | terminated guard daemon |
| run [oprions...]         | create and run task     |
| add [oprions...]         | add task                |
| drop [oprions...]        | stop and drop task      |
| start [oprions...]       | start task              |
| restart [oprions...]     | restart task            |
| stop [oprions...]        | stop task               |
| list [oprions...]        | list tasks              |


## SubCommands

### run

`watchmen run [OPTIONS] ...`

| Option         | Description         |
| -------------- | ------------------- |
| -h, --help     | Show help           |
| -n, --name     | task name           |
| -o, --origin   | task start datetime |
| -i, --interval | task time interval  |

### add

`watchmen add [OPTIONS] ...`

| Option         | Description         |
| -------------- | ------------------- |
| -h, --help     | Show help           |
| -n, --name     | task name           |
| -o, --origin   | task start datetime |
| -i, --interval | task time interval  |

### drop

`watchmen drop [OPTIONS] ...`

| Option     | Description |
| ---------- | ----------- |
| -h, --help | Show help   |
| -n, --name | task name   |
| -p, --pid  | task pid    |

### start

`watchmen start [OPTIONS] ...`

| Option     | Description |
| ---------- | ----------- |
| -h, --help | Show help   |
| -n, --name | task name   |
| -p, --pid  | task pid    |

### restart

`watchmen restart [OPTIONS] ...`

| Option     | Description |
| ---------- | ----------- |
| -h, --help | Show help   |
| -i, --name | task id     |
| -n, --pid  | task name   |

### stop

`watchmen stop [OPTIONS] ...`

| Option     | Description |
| ---------- | ----------- |
| -h, --help | Show help   |
| -n, --name | task name   |
| -p, --pid  | task pid    |

### list

`watchmen list [OPTIONS] ...`

| Option       | Description |
| ------------ | ----------- |
| -h, --help   | Show help   |
| -n, --name   | task name   |
| -s, --status | task status |
| -p, --pid    | task pid    |
| -m, --more   | more info   |

## Build and run Examples

```bash
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
-rw-r--r-- 1 user user    0 Sep 01 00:00 daemon.log
drwxr-xr-x 2 user user 4096 Sep 01 00:00 stdout
-rw-r--r-- 1 user user    5 Sep 01 00:00 watchmen.pid
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
