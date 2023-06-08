use std::{error::Error, fs::File, io::Read, path::Path, process::Stdio};

use configparser::ini::Ini;
use tokio::process::{Child, Command};

use crate::task::{AsyncTask, PeriodicTask, ScheduledTask, Task, TaskFlag, TaskType, Tasks};

impl TaskFlag {
    pub fn from_file(path: &Path) -> Result<Vec<TaskFlag>, Box<dyn Error>> {
        let ext = path.extension().unwrap().to_str().unwrap();
        match ext {
            "ini" => TaskFlag::from_ini(path),
            "toml" => TaskFlag::from_toml(path),
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
            tasks.push(TaskFlag::new(ini.get(section.as_str(), "name").unwrap()));
        }

        Ok(tasks)
    }

    pub fn from_toml(path: &Path) -> Result<Vec<TaskFlag>, Box<dyn Error>> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let mut tasks = Vec::new();
        for i in toml::from_str::<Tasks>(&contents)?.task {
            tasks.push(TaskFlag { name: i.name });
        }
        Ok(tasks)
    }
}

impl Task {
    pub fn from_file(path: &Path) -> Result<Tasks, Box<dyn Error>> {
        let ext = path.extension().unwrap().to_str().unwrap();
        match ext {
            "ini" => Task::from_ini(path),
            "toml" => Task::from_toml(path),
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
            task.id = ini.getint(section, "id")?.unwrap();
            task.name = ini.get(section, "name").unwrap();
            task.command = ini.get(section, "command").unwrap();
            task.args = ini
                .get(section, "args")
                .unwrap_or("".to_string())
                .split(" ")
                .map(|s| s.to_string())
                .collect();
            task.dir = ini.get(section, "dir");
            if task.dir.is_none() {
                task.dir = Some(std::env::current_dir()?.to_str().unwrap().to_string());
            }
            for env in ini.get(section, "env").unwrap_or("".to_string()).split(" ") {
                let kv: Vec<&str> = env.split("=").collect();
                if kv.len() == 2 {
                    task.env.insert(kv[0].to_string(), kv[1].to_string());
                }
            }
            task.stdin = ini.getbool(section, "stdin").unwrap();
            task.stdout = ini.get(section, "stdout");
            task.stderr = ini.get(section, "stderr");

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
                        tt.year = Some(year as u64);
                    }
                    if let Some(month) = ini.getint(section, "month")? {
                        if month < 1 || month > 12 {
                            return Err(Box::new(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("Invalid month: {}", month),
                            )));
                        }
                        tt.month = Some(month as u64);
                    }
                    if let Some(day) = ini.getint(section, "day")? {
                        if day < 1 || day > 31 {
                            return Err(Box::new(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("Invalid day: {}", day),
                            )));
                        }
                        tt.day = Some(day as u64);
                    }
                    if let Some(hour) = ini.getint(section, "hour")? {
                        if hour < 0 || hour > 23 {
                            return Err(Box::new(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("Invalid hour: {}", hour),
                            )));
                        }
                        tt.hour = Some(hour as u64);
                    }
                    if let Some(minute) = ini.getint(section, "minute")? {
                        if minute < 0 || minute > 59 {
                            return Err(Box::new(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("Invalid minute: {}", minute),
                            )));
                        }
                        tt.minute = Some(minute as u64);
                    }
                    if let Some(second) = ini.getint(section, "second")? {
                        if second < 0 || second > 59 {
                            return Err(Box::new(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("Invalid second: {}", second),
                            )));
                        }
                        tt.second = Some(second as u64);
                    }
                    TaskType::Scheduled(tt)
                }
                "async" => TaskType::Async(AsyncTask {
                    started_at: 0,
                    stopped_at: 0,
                }),
                "periodic" => {
                    let mut tt = PeriodicTask {
                        started_after: 0,
                        interval: 60,
                        last_run: 0,
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
            command = command.stdout(Stdio::null());
        }
        if let Some(stderr) = &self.stderr {
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
    pub fn new(name: String) -> Self {
        TaskFlag { name }
    }
}
