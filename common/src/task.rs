use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub year: Option<u64>,
    pub month: Option<u64>,
    pub day: Option<u64>,
    pub hour: Option<u64>,
    pub minute: Option<u64>,
    pub second: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncTask {
    pub started_at: u64,
    pub stopped_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodicTask {
    pub started_after: u64,
    pub interval: u64,
    pub last_run: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    Scheduled(ScheduledTask),
    Async(AsyncTask),
    Periodic(PeriodicTask),
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Task id (unique)
    pub id: i64,

    /// Task name (unique)
    pub name: String,

    /// Task command
    pub command: String,

    /// Task arguments
    pub args: Vec<String>,

    /// Task working directory
    pub dir: Option<String>,

    /// Task environment variables
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

impl Default for Task {
    fn default() -> Self {
        let now = SystemTime::now();
        let timestamp = now
            .duration_since(UNIX_EPOCH)
            .expect("Failed to get timestamp")
            .as_secs();
        Task {
            id: 0,
            name: "Default".to_string(),
            command: "".to_string(),
            args: vec![],
            dir: None,
            env: HashMap::new(),
            stdin: None,
            stdout: None,
            stderr: None,
            created_at: timestamp,
            task_type: TaskType::None,
            pid: None,
            status: None,
            code: None,
        }
    }
}

unsafe impl Send for Task {}
unsafe impl Sync for Task {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tasks {
    pub task: Vec<Task>,
}