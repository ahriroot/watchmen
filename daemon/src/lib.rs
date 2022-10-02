pub mod command;
pub mod entity;
pub mod macros;
pub mod monitor;
pub mod socket;
pub mod utils;

pub mod global {
    use std::{
        error::Error,
        fs::File,
        path::Path,
    };

    use lazy_static::lazy_static;
    use serde_json;
    use tokio::sync::Mutex;

    use crate::{
        command::start::{self},
        entity::Task,
    };

    lazy_static! {
        static ref TASKS: Mutex<Vec<Task>> = Mutex::new(Vec::new());
    }

    pub async fn save_tasks(tasks: Vec<Task>) -> Result<(), Box<dyn Error>> {
        let args: Vec<String> = std::env::args().collect();
        if args.len() < 4 {
            return Err("Command line args error".into());
        }
        let home_path = args[2].clone();

        let path = Path::new(&home_path);
        let path = path.join("tasks.json");
        match File::create(path) {
            Ok(f) => match serde_json::to_writer_pretty(f, &tasks) {
                _ => Ok(()),
            },
            Err(_) => Ok(()),
        }
    }

    pub async fn load_tasks(home_path: String) -> Result<(), Box<dyn Error>> {
        let path = Path::new(&home_path);
        let path = path.join("tasks.json");

        if path.exists() && path.is_file() {
            let f = File::open(path).unwrap();
            let data: Vec<Task> = serde_json::from_reader(f).unwrap();
            let mut tasks = TASKS.lock().await;
            *tasks = data.clone();

            // 释放锁, 启动 task 时需要更改 task 状态, 需要获取锁
            drop(tasks);

            for task in data {
                if task.status == "running" {
                    match start::start_task_by_task(task.clone()).await {
                        Ok(_) => {}
                        Err(_) => {}
                    };
                }
            }
            Ok(())
        } else {
            Err("Tasks file not exists".into())
        }
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

    pub async fn get_task_by_id(id: u128) -> Result<Task, Box<dyn Error>> {
        let tasks = get_all_tasks().await?;
        for task in tasks.iter() {
            if task.id == id {
                return Ok(task.clone());
            }
        }
        let err = format!("Task id '{}' not found", id);
        return Err(err.into());
    }

    pub async fn get_task_by_name(name: String) -> Result<Task, Box<dyn Error>> {
        let tasks = get_all_tasks().await?;
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

    pub async fn remove_task_by_id(id: u128) -> Result<(), Box<dyn Error>> {
        let mut tasks = TASKS.lock().await;
        let mut index = 0;
        for (i, task) in tasks.iter().enumerate() {
            if task.id == id {
                index = i;
                break;
            }
        }
        tasks.remove(index);
        save_tasks(tasks.clone()).await?;
        Ok(())
    }

    pub async fn remove_task_by_name(name: String) -> Result<Option<Task>, Box<dyn Error>> {
        let mut tasks = TASKS.lock().await;
        let pos = tasks.iter().position(|task| task.name == name);
        if let Some(pos) = pos {
            let res = tasks.remove(pos);
            save_tasks(tasks.clone()).await?;
            return Ok(Some(res));
        }
        Ok(None)
    }

    pub async fn remove_task_by_pid(pid: u32) -> Result<Option<Task>, Box<dyn Error>> {
        let mut tasks = TASKS.lock().await;
        let pos = tasks.iter().position(|task| task.pid == pid);
        if let Some(pos) = pos {
            let res = tasks.remove(pos);
            save_tasks(tasks.clone()).await?;
            return Ok(Some(res));
        }
        Ok(None)
    }

    pub async fn add_task(task: Task) -> Result<(), Box<dyn Error>> {
        let mut tasks = TASKS.lock().await;
        let timestamp = crate::utils::get_millis().await;
        let task = Task {
            created_at: timestamp,
            ..task
        };
        tasks.push(task);
        save_tasks(tasks.clone()).await?;
        Ok(())
    }

    pub async fn update_pid_by_name(name: String, pid: u32) -> Result<(), Box<dyn Error>> {
        let mut tasks = TASKS.lock().await;
        let pos = tasks.iter().position(|task| task.name == name);
        if let Some(pos) = pos {
            tasks[pos].pid = pid;
            save_tasks(tasks.clone()).await?;
        }
        Ok(())
    }

    pub async fn update_status_by_name(name: String, status: String) -> Result<(), Box<dyn Error>> {
        let mut tasks = TASKS.lock().await;
        let pos = tasks.iter().position(|task| task.name == name);
        if let Some(pos) = pos {
            tasks[pos].status = status;
            save_tasks(tasks.clone()).await?;
        }
        Ok(())
    }

    pub async fn update_started_at_by_id(id: u128, started_at: u128) -> Result<(), Box<dyn Error>> {
        let mut tasks = TASKS.lock().await;
        let pos = tasks.iter().position(|task| task.id == id);
        if let Some(pos) = pos {
            tasks[pos].started_at = started_at;
            save_tasks(tasks.clone()).await?;
        }
        Ok(())
    }

    pub async fn update_exited_at_by_id(id: u128, exited_at: u128) -> Result<(), Box<dyn Error>> {
        let mut tasks = TASKS.lock().await;
        let pos = tasks.iter().position(|task| task.id == id);
        if let Some(pos) = pos {
            tasks[pos].exited_at = exited_at;
            save_tasks(tasks.clone()).await?;
        }
        Ok(())
    }

    pub async fn update_stopped_at_by_id(id: u128, stopped_at: u128) -> Result<(), Box<dyn Error>> {
        let mut tasks = TASKS.lock().await;
        let pos = tasks.iter().position(|task| task.id == id);
        if let Some(pos) = pos {
            tasks[pos].stopped_at = stopped_at;
            save_tasks(tasks.clone()).await?;
        }
        Ok(())
    }

    pub async fn update_laststart_at_by_id(
        id: u128,
        laststart_at: u128,
    ) -> Result<(), Box<dyn Error>> {
        let mut tasks = TASKS.lock().await;
        let pos = tasks.iter().position(|task| task.id == id);
        if let Some(pos) = pos {
            tasks[pos].laststart_at = laststart_at;
            save_tasks(tasks.clone()).await?;
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
            save_tasks(tasks.clone()).await?;
        }
        Ok(())
    }
}
