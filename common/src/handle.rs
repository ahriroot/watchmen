use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::task::{Task, TaskType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub command: Command,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    Run(Task),
    Add(Task),
    Stop(String),
    Start(String),
    Remove(String),
    Write(String, String),
    List(Option<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub code: u16,
    pub msg: String,
    pub data: Option<Data>,
}

impl Response {
    pub fn new(code: u16, msg: String, data: Option<Data>) -> Self {
        Self { code, msg, data }
    }

    pub fn success(data: Option<Data>) -> Self {
        Self {
            code: 10000,
            msg: "Success".to_string(),
            data,
        }
    }

    pub fn wrong(msg: String) -> Self {
        Self {
            code: 40000,
            msg: "Wrong".to_string(),
            data: Some(Data::String(msg)),
        }
    }

    pub fn failed(msg: String) -> Self {
        Self {
            code: 50000,
            msg: "Failed".to_string(),
            data: Some(Data::String(msg)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Data {
    None,
    String(String),
    Status(Vec<Status>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Status {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub dir: Option<String>,
    pub env: HashMap<String, String>,
    pub stdin: Option<bool>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub created_at: u64,
    pub task_type: TaskType,
    pub pid: Option<u32>,
    pub status: Option<String>,
    pub code: Option<i32>,
}

impl From<crate::task::Task> for Status {
    fn from(task: crate::task::Task) -> Self {
        Self {
            name: task.name,
            command: task.command,
            args: task.args,
            dir: task.dir,
            env: task.env,
            stdin: task.stdin,
            stdout: task.stdout,
            stderr: task.stderr,
            created_at: task.created_at,
            task_type: task.task_type,
            pid: task.pid,
            status: task.status,
            code: task.code,
        }
    }
}