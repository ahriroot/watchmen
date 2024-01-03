use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

fn default_i64_0() -> i64 {
    0
}

fn default_u64_0() -> u64 {
    0
}

fn default_none_u64() -> Option<u64> {
    None
}

fn default_none_string() -> Option<String> {
    None
}

fn default_false() -> bool {
    false
}

fn default_vec_string() -> Vec<String> {
    Vec::new()
}

fn default_map_string_string() -> HashMap<String, String> {
    HashMap::new()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub year: Option<i32>,
    pub month: Option<u32>,
    pub day: Option<u32>,
    pub hour: Option<u32>,
    pub minute: Option<u32>,
    pub second: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncTask {
    #[serde(default = "default_none_u64")]
    pub max_restart: Option<u64>,
    #[serde(default = "default_u64_0")]
    pub has_restart: u64,
    #[serde(default = "default_u64_0")]
    pub started_at: u64,
    #[serde(default = "default_u64_0")]
    pub stopped_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodicTask {
    #[serde(default = "default_u64_0")]
    pub started_after: u64,
    pub interval: u64,
    #[serde(default = "default_u64_0")]
    pub last_run: u64,
    #[serde(default = "default_false")]
    pub sync: bool,
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
    #[serde(default = "default_vec_string")]
    pub args: Vec<String>,

    /// Task group
    pub group: Option<String>,

    /// Task working directory
    pub dir: Option<String>,

    /// Task environment variables
    #[serde(default = "default_map_string_string")]
    pub env: HashMap<String, String>,

    pub stdin: Option<bool>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,

    #[serde(default = "default_created_at")]
    pub created_at: u64,
    pub task_type: TaskType,

    pub pid: Option<u32>,

    #[serde(default = "default_status")]
    pub status: Option<String>,
    pub code: Option<i32>,
}

fn default_created_at() -> u64 {
    let now = SystemTime::now();
    let timestamp = now
        .duration_since(UNIX_EPOCH)
        .expect("Failed to get timestamp")
        .as_secs();
    timestamp
}

fn default_status() -> Option<String> {
    Some("added".to_owned())
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
            command: String::new(),
            args: vec![],
            group: None,
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
pub struct TaskFlag {
    #[serde(default = "default_i64_0")]
    pub id: i64,
    #[serde(default = "default_none_string")]
    pub name: Option<String>,
    #[serde(default = "default_none_string")]
    pub group: Option<String>,
    #[serde(default = "default_false")]
    pub mat: bool,
}

impl Default for TaskFlag {
    fn default() -> Self {
        TaskFlag {
            id: 0,
            name: Some(String::new()),
            group: None,
            mat: false,
        }
    }
}

unsafe impl Send for TaskFlag {}
unsafe impl Sync for TaskFlag {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tasks {
    pub task: Vec<Task>,
}
