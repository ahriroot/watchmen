use std::{error::Error, fs::File, io::Read, path::Path, process::Stdio};

use configparser::ini::Ini;
use tokio::process::{Child, Command};

use crate::{
    arg::{AddArgs, FlagArgs},
    task::{AsyncTask, PeriodicTask, ScheduledTask, Task, TaskFlag, TaskType, Tasks},
};

impl TaskFlag {
    pub fn from_args(args: FlagArgs) -> Result<Vec<TaskFlag>, Box<dyn Error>> {
        let mut tasks = Vec::new();
        if let Some(id) = args.id {
            tasks.push(TaskFlag::new(id));
        } else if let Some(name) = args.name {
            tasks.push(TaskFlag {
                id: 0,
                name: Some(name),
                group: None,
                mat: false,
            });
        } else if let Some(group) = args.group {
            tasks.push(TaskFlag {
                id: 0,
                name: None,
                group: Some(group),
                mat: false,
            });
        } else {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Task name is none",
            )));
        }
        Ok(tasks)
    }

    pub fn from_file(path: &Path) -> Result<Vec<TaskFlag>, Box<dyn Error>> {
        let ext = match path.extension() {
            Some(ext) => match ext.to_str() {
                Some(ext) => ext,
                None => {
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Config file extension is none",
                    )));
                }
            },
            None => {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Invalid config file extension",
                )));
            }
        };
        match ext {
            "ini" => TaskFlag::from_ini(path),
            "toml" => TaskFlag::from_toml(path),
            "json" => TaskFlag::from_json(path),
            _ => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Invalid file extension: {}", ext),
            ))),
        }
    }

    pub fn from_ini(path: &Path) -> Result<Vec<TaskFlag>, Box<dyn Error>> {
        let mut ini = Ini::new();
        ini.load(&path)?;
        let mut tasks = Vec::new();

        for section in ini.sections() {
            let id = match ini.getint(section.as_str(), "id") {
                Ok(id) => match id {
                    Some(id) => id,
                    None => {
                        return Err(Box::new(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "Invalid config file, id is none",
                        )));
                    }
                },
                Err(_) => {
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Invalid config file, id is error",
                    )));
                }
            };
            tasks.push(TaskFlag::new(id));
        }

        Ok(tasks)
    }

    pub fn from_toml(path: &Path) -> Result<Vec<TaskFlag>, Box<dyn Error>> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let mut tasks = Vec::new();
        for i in toml::from_str::<Tasks>(&contents)?.task {
            tasks.push(TaskFlag {
                id: i.id,
                name: Some(i.name),
                group: i.group,
                mat: false,
            });
        }
        Ok(tasks)
    }

    pub fn from_json(path: &Path) -> Result<Vec<TaskFlag>, Box<dyn Error>> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let mut tasks = Vec::new();
        for i in Task::deserialize(&contents)? {
            tasks.push(TaskFlag {
                id: i.id,
                name: Some(i.name),
                group: i.group,
                mat: false,
            });
        }
        Ok(tasks)
    }
}

impl Task {
    pub fn from_args(args: AddArgs) -> Result<Tasks, Box<dyn Error>> {
        let mut task = Task::default();

        if let Some(name) = args.name {
            task.name = name;
        } else {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Task name is none",
            )));
        }

        if let Some(command) = args.command {
            task.command = command;
        } else {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Task command is none",
            )));
        }

        if let Some(ags) = args.args {
            task.args = ags;
        } else {
            task.args = Vec::new();
        }

        task.dir = args.dir;

        if args.stdin {
            task.stdin = Some(true);
        }

        task.stdout = args.stdout;
        task.stderr = args.stderr;

        let tasks = Tasks { task: vec![task] };
        Ok(tasks)
    }

    pub fn from_file(path: &Path) -> Result<Tasks, Box<dyn Error>> {
        let ext = match path.extension() {
            Some(ext) => match ext.to_str() {
                Some(ext) => ext,
                None => {
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Config file extension is none",
                    )));
                }
            },
            None => {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Invalid config file extension",
                )));
            }
        };
        match ext {
            "ini" => Task::from_ini(path),
            "toml" => Task::from_toml(path),
            "json" => Task::from_json(path),
            _ => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Invalid file extension: {}", ext),
            ))),
        }
    }

    pub fn from_ini(path: &Path) -> Result<Tasks, Box<dyn Error>> {
        let mut ini = Ini::new();
        ini.load(&path)?;
        let mut tasks = Vec::new();

        for section in ini.sections() {
            let section = section.as_str();
            let mut task = Task::default();
            task.id = if let Some(id) = ini.getint(section, "id")? {
                id
            } else {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Invalid config file, id is none",
                )));
            };
            task.name = if let Some(name) = ini.get(section, "name") {
                name
            } else {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Invalid config file, name is none",
                )));
            };
            task.command = if let Some(command) = ini.get(section, "command") {
                command
            } else {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Invalid config file, command is none",
                )));
            };
            task.args = ini
                .get(section, "args")
                .unwrap_or(String::new())
                .split(" ")
                .map(|s| s.to_string())
                .collect();
            task.group = ini.get(section, "group");
            task.dir = ini.get(section, "dir");
            if task.dir.is_none() {
                task.dir = Some(std::env::current_dir()?.to_str().unwrap().to_string());
            }
            for env in ini.get(section, "env").unwrap_or(String::new()).split(" ") {
                let kv: Vec<&str> = env.split("=").collect();
                if kv.len() == 2 {
                    task.env.insert(kv[0].to_string(), kv[1].to_string());
                }
            }
            task.stdin = ini.getbool(section, "stdin")?;
            task.stdout = ini.get(section, "stdout");
            task.stderr = ini.get(section, "stderr");
            task.status = Some("added".to_string());

            let task_type = ini.get(section, "task_type").unwrap_or("none".to_string());

            let tt = match task_type.as_str() {
                "scheduled" => {
                    let mut tt = ScheduledTask {
                        year: None,
                        month: None,
                        day: None,
                        hour: None,
                        minute: None,
                        second: None,
                    };
                    if let Some(year) = ini.getint(section, "year")? {
                        if year < 1970 {
                            return Err(Box::new(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("Invalid year: {}", year),
                            )));
                        }
                        tt.year = Some(year as i32);
                    }
                    if let Some(month) = ini.getint(section, "month")? {
                        if month < 1 || month > 12 {
                            return Err(Box::new(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("Invalid month: {}", month),
                            )));
                        }
                        tt.month = Some(month as u32);
                    }
                    if let Some(day) = ini.getint(section, "day")? {
                        if day < 1 || day > 31 {
                            return Err(Box::new(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("Invalid day: {}", day),
                            )));
                        }
                        tt.day = Some(day as u32);
                    }
                    if let Some(hour) = ini.getint(section, "hour")? {
                        if hour < 0 || hour > 23 {
                            return Err(Box::new(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("Invalid hour: {}", hour),
                            )));
                        }
                        tt.hour = Some(hour as u32);
                    }
                    if let Some(minute) = ini.getint(section, "minute")? {
                        if minute < 0 || minute > 59 {
                            return Err(Box::new(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("Invalid minute: {}", minute),
                            )));
                        }
                        tt.minute = Some(minute as u32);
                    }
                    if let Some(second) = ini.getint(section, "second")? {
                        if second < 0 || second > 59 {
                            return Err(Box::new(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("Invalid second: {}", second),
                            )));
                        }
                        tt.second = Some(second as u32);
                    }
                    TaskType::Scheduled(tt)
                }
                "async" => {
                    let max_restart = ini.get(section, "max_restart");
                    TaskType::Async(AsyncTask {
                        max_restart: if let Some(max) = max_restart {
                            Some(max.parse::<u64>()?)
                        } else {
                            None
                        },
                        has_restart: 0,
                        started_at: 0,
                        stopped_at: 0,
                    })
                }
                "periodic" => {
                    let mut tt = PeriodicTask {
                        started_after: 0,
                        interval: 60,
                        last_run: 0,
                        sync: true,
                    };
                    if let Some(started_after) = ini.getint(section, "started_after")? {
                        if started_after < 0 {
                            return Err(Box::new(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("Invalid started_after: {}", started_after),
                            )));
                        }
                        tt.started_after = started_after as u64;
                    }
                    if let Some(interval) = ini.getint(section, "interval")? {
                        if interval <= 0 {
                            return Err(Box::new(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("Invalid interval: {}", interval),
                            )));
                        }
                        tt.interval = interval as u64;
                    }
                    if let Some(sync) = ini.getbool(section, "sync")? {
                        tt.sync = sync;
                    }
                    TaskType::Periodic(tt)
                }
                _ => {
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Invalid task_type: {}", task_type),
                    )));
                }
            };
            task.task_type = tt;
            tasks.push(task);
        }

        Ok(Tasks { task: tasks })
    }

    pub fn from_toml(path: &Path) -> Result<Tasks, Box<dyn Error>> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let config: Tasks = toml::from_str(&contents)?;
        Ok(config)
    }

    pub fn from_json(path: &Path) -> Result<Tasks, Box<dyn Error>> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let tasks = Task::deserialize(&contents)?;
        Ok(Tasks { task: tasks })
    }
}

impl Task {
    pub fn deserialize(s: &str) -> Result<Vec<Task>, Box<dyn Error>> {
        let tasks: Vec<Task> = serde_json::from_str(s)?;
        Ok(tasks)
    }

    pub fn serialize(tasks: Vec<Task>) -> Result<String, Box<dyn Error>> {
        let serialized = serde_json::to_string(&tasks)?;
        Ok(serialized)
    }
}

impl Task {
    pub async fn start(&self) -> Result<Child, Box<dyn Error>> {
        let mut command = Command::new(&self.command);
        let command = command.args(&self.args);
        let command = command.envs(std::env::vars());
        let mut command = command.kill_on_drop(false);
        for (key, value) in &self.env {
            command = command.env(key, value);
        }
        if let Some(dir) = &self.dir {
            command = command.current_dir(&dir);
        }
        if let Some(stdout) = &self.stdout {
            if stdout != "" {
                let dir = Path::new(stdout).parent().unwrap();
                if !dir.exists() {
                    std::fs::create_dir_all(dir)?;
                }
                let file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(stdout)?;
                command = command.stdout(Stdio::from(file));
            } else {
                command = command.stdout(Stdio::piped());
            }
        } else {
            command = command.stdout(Stdio::null());
        }
        if let Some(stderr) = &self.stderr {
            if stderr != "" {
                let dir = Path::new(stderr).parent().unwrap();
                if !dir.exists() {
                    std::fs::create_dir_all(dir)?;
                }
                let file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(stderr)?;
                command = command.stderr(Stdio::from(file));
            } else {
                command = command.stderr(Stdio::piped());
            }
        } else {
            command = command.stderr(Stdio::null());
        }
        if let Some(_stdin) = &self.stdin {
            command = command.stdin(Stdio::piped());
        } else {
            command = command.stdin(Stdio::null());
        }

        let child = command.spawn()?;

        Ok(child)
    }
}

impl Tasks {
    pub fn new() -> Self {
        Tasks { task: Vec::new() }
    }
}

impl TaskFlag {
    pub fn new(id: i64) -> Self {
        TaskFlag {
            id,
            name: Some(String::new()),
            group: None,
            mat: false,
        }
    }
}
