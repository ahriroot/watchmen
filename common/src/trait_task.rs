use std::{error::Error, fs::File, io::Read, path::Path};

use configparser::ini::Ini;
use serde::{Deserialize, Serialize};

use crate::task::{AsyncTask, PeriodicTask, ScheduledTask, Task, TaskType};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NotTask {
    pub name: String,
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IsTask {
    Task(Task),
    Error(NotTask),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tasks {
    pub task: Vec<Task>,
}

impl Task {
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
                .get(section, "ini")
                .unwrap_or("".to_string())
                .split(" ")
                .map(|s| s.to_string())
                .collect();
            task.dir = ini
                .get(section, "dir")
                .unwrap_or(std::env::current_dir()?.to_str().unwrap().to_string());
            for env in ini.get(section, "env").unwrap_or("".to_string()).split(" ") {
                let kv: Vec<&str> = env.split("=").collect();
                if kv.len() == 2 {
                    task.env.insert(kv[0].to_string(), kv[1].to_string());
                }
            }
            task.stdin = ini.get(section, "stdin");
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
                            NotTask {
                                name: task.name.clone(),
                                error: format!("Invalid year: {}", year),
                            };
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

impl Task {}
