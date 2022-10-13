use std::error::Error;

use crate::entity;
use crate::global::{get_task_by_id, get_task_by_name, update_status_by_name};

pub async fn resume_task(command: entity::Command) -> Result<entity::Response, Box<dyn Error>> {
    let task;
    if command.options.contains_key("id") {
        let id = command.options.get("id").unwrap();
        if let entity::Opt::U128(ref i) = id {
            task = get_task_by_id(*i).await?;
        } else {
            return Ok(entity::Response {
                code: 50000,
                msg: "Arg 'id' must be a usize".to_string(),
                data: None,
            });
        }
    } else if command.options.contains_key("name") {
        let name = command.options.get("name").unwrap();
        if let entity::Opt::Str(ref s) = name {
            task = get_task_by_name(s.clone()).await?;
        } else {
            return Ok(entity::Response {
                code: 50000,
                msg: "Arg 'name' must be a string".to_string(),
                data: None,
            });
        }
    } else {
        if command.args.len() == 0 {
            return Ok(entity::Response {
                code: 40000,
                msg: "Arg 'name' is required".to_string(),
                data: None,
            });
        } else {
            task = get_task_by_name(command.args[0].clone()).await?;
        }
    }

    if task.status != "paused" && task.status != "waiting" {
        return Ok(entity::Response {
            code: 40000,
            msg: format!("Task with '{}' could not be resumed", task.status),
            data: None,
        });
    }

    update_status_by_name(task.name, "interval".to_string()).await?;

    let res = entity::Response {
        code: 10000,
        msg: format!("{} success", command.name),
        data: None,
    };
    return Ok(res);
}
