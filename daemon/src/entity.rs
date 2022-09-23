use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: u128,
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub status: String,
    pub pid: u32,
    pub created_at: u128,
    pub started_at: u128,
    pub exited_at: u128,
    pub stopped_at: u128,
    pub exit_code: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Command {
    pub name: String,
    pub args: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub name: String,
    pub command: Command,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Data {
    TaskList(Vec<Task>),
    None,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub code: u32,
    pub msg: String,
    pub data: Option<Data>,
}
