use std::error::Error;

use crate::entity::{self, Task};
use crate::global::{add_task, check_exists};

async fn register(task: Task) -> Result<(), Box<dyn Error>> {
    add_task(task).await?;
    Ok(())
}

pub async fn add_a_task(task: Task) -> Result<entity::Response, Box<dyn Error>> {
    let args_cmdline: Vec<String> = std::env::args().collect();
    if args_cmdline.len() < 3 {
        return Ok(entity::Response {
            code: 50000,
            msg: "Miss stdout path".to_string(),
            data: None,
        });
    }

    let exists = check_exists(task.name.clone()).await?;
    if exists {
        return Ok(entity::Response {
            code: 40000,
            msg: "Task already exists".to_string(),
            data: None,
        });
    }

    let t = task.clone();
    register(t).await?;

    return Ok(entity::Response {
        code: 10000,
        msg: "Task added successfully".to_string(),
        data: None,
    });
}
