use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScheduledTask {
    pub year: Option<u64>,
    pub month: Option<u64>,
    pub day: Option<u64>,
    pub hour: Option<u64>,
    pub minute: Option<u64>,
    pub second: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AsyncTask {
    pub started_at: u64,
    pub stopped_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PeriodicTask {
    pub started_after: u64,
    pub interval: u64,
    pub last_run: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskType {
    Scheduled(ScheduledTask),
    Async(AsyncTask),
    Periodic(PeriodicTask),
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    pub dir: String,

    /// Task environment variables
    pub env: HashMap<String, String>,

    pub stdin: Option<String>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,

    pub created_at: u64,
    pub task_type: TaskType,
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
            name: "".to_string(),
            command: "".to_string(),
            args: vec![],
            dir: "".to_string(),
            env: HashMap::new(),
            stdin: None,
            stdout: None,
            stderr: None,
            created_at: timestamp,
            task_type: TaskType::None,
        }
    }
}

unsafe impl Send for Task {}
unsafe impl Sync for Task {}
