use serde::{Deserialize, Serialize};

use crate::task::Task;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    Run,
    Stop,
    Start,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Body {
    Task(Task),
    TaskFlag(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub command: Command,
    pub body: Body,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response<T> {
    pub code: u16,
    pub msg: String,
    pub data: Option<T>,
}
