pub mod command;
pub mod entity;
pub mod socket;
pub mod utils;

pub mod global {
    use std::{
        error::Error,
        time::{SystemTime, UNIX_EPOCH},
    };

    use lazy_static::lazy_static;
    use tokio::sync::Mutex;

    use crate::entity::Task;

    lazy_static! {
        static ref TASKS: Mutex<Vec<Task>> = Mutex::new(Vec::new());
    }

    pub async fn check_exists(name: String) -> Result<bool, Box<dyn Error>> {
        let tasks = get_all_tasks().await?;
        for task in tasks.iter() {
            if task.name == name {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub async fn get_task_by_name(name: String) -> Result<Task, Box<dyn Error>> {
        let tasks = TASKS.lock().await;
        for task in tasks.iter() {
            if task.name == name {
                return Ok(task.clone());
            }
        }
        let err = format!("Task named '{}' not found", name);
        return Err(err.into());
    }

    pub async fn get_task_by_pid(pid: u32) -> Result<Task, Box<dyn Error>> {
        let tasks = TASKS.lock().await;
        for task in tasks.iter() {
            if task.pid == pid {
                return Ok(task.clone());
            }
        }
        let err = format!("Task pid is '{}' not found", pid);
        return Err(err.into());
    }

    pub async fn get_all_tasks() -> Result<Vec<Task>, Box<dyn Error>> {
        let tasks = TASKS.lock().await;
        Ok(tasks.to_vec())
    }

    pub async fn remove_task_by_name(name: String) -> Result<Option<Task>, Box<dyn Error>> {
        let mut tasks = TASKS.lock().await;
        let pos = tasks.iter().position(|task| task.name == name);
        if let Some(pos) = pos {
            let res = tasks.remove(pos);
            return Ok(Some(res));
        }
        Ok(None)
    }

    pub async fn remove_task_by_pid(pid: u32) -> Result<Option<Task>, Box<dyn Error>> {
        let mut tasks = TASKS.lock().await;
        let pos = tasks.iter().position(|task| task.pid == pid);
        if let Some(pos) = pos {
            let res = tasks.remove(pos);
            return Ok(Some(res));
        }
        Ok(None)
    }

    pub async fn add_task(task: Task) -> Result<(), Box<dyn Error>> {
        let mut tasks = TASKS.lock().await;
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let task = Task {
            created_at: timestamp,
            ..task
        };
        tasks.push(task);
        Ok(())
    }

    pub async fn update_pid_by_name(name: String, pid: u32) -> Result<(), Box<dyn Error>> {
        let mut tasks = TASKS.lock().await;
        let pos = tasks.iter().position(|task| task.name == name);
        if let Some(pos) = pos {
            tasks[pos].pid = pid;
        }
        Ok(())
    }

    pub async fn update_status_by_name(name: String, status: String) -> Result<(), Box<dyn Error>> {
        let mut tasks = TASKS.lock().await;
        let pos = tasks.iter().position(|task| task.name == name);
        if let Some(pos) = pos {
            tasks[pos].status = status;
        }
        Ok(())
    }

    pub async fn update_exit_code_by_name(
        name: String,
        exit_code: u32,
    ) -> Result<(), Box<dyn Error>> {
        let mut tasks = TASKS.lock().await;
        let pos = tasks.iter().position(|task| task.name == name);
        if let Some(pos) = pos {
            tasks[pos].exit_code = exit_code;
        }
        Ok(())
    }
}
