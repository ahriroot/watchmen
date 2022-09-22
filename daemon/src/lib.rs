pub mod command;
pub mod entity;
pub mod socket;

pub mod global {
    use std::error::Error;

    use lazy_static::lazy_static;
    use tokio::sync::Mutex;

    use crate::entity::Task;

    lazy_static! {
        static ref TASKS: Mutex<Vec<Task>> = Mutex::new(Vec::new());
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

    pub async fn add_task(task: Task) -> Result<(), Box<dyn Error>> {
        let mut tasks = TASKS.lock().await;
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
}
