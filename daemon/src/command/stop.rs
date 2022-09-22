use std::error::Error;

use crate::{entity, global::remove_task_by_name};

extern "C" {
    pub fn kill(pid: i32, sig: i32) -> i32;
}

pub async fn stop_task(name: String) -> Result<entity::Response, Box<dyn Error>> {
    let task = remove_task_by_name(name.clone()).await?;
    match task {
        Some(task) => {
            let pid = task.pid;
            let res = unsafe { kill(pid as i32, 15) };
            if res == 0 {
                let res = entity::Response {
                    code: 10000,
                    msg: "success".to_string(),
                    data: None,
                };
                return Ok(res);
            }
            let res = entity::Response {
                code: 50000,
                msg: "failed".to_string(),
                data: None,
            };
            return Ok(res);
        }
        None => {
            let res = entity::Response {
                code: 50000,
                msg: format!("task '{}' not found", name),
                data: None,
            };
            return Ok(res);
        }
    }
}
