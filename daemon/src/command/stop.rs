use std::error::Error;

use crate::{
    entity,
    global::{
        get_task_by_id, get_task_by_name, get_task_by_pid, update_pid_by_name,
        update_status_by_name,
    },
};

extern "C" {
    pub fn kill(pid: i32, sig: i32) -> i32;
}

pub async fn stop_task(command: entity::Command) -> Result<entity::Response, Box<dyn Error>> {
    let task;
    if command.options.contains_key("id") {
        let id = command.options.get("id").unwrap();
        if let entity::Opt::U128(ref i) = id.value {
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
        if let entity::Opt::Str(ref s) = name.value {
            task = get_task_by_name(s.clone()).await?;
        } else {
            return Ok(entity::Response {
                code: 1,
                msg: "Arg 'name' must be a string".to_string(),
                data: None,
            });
        }
    } else if command.options.contains_key("pid") {
        let name = command.options.get("pid").unwrap();
        if let entity::Opt::U32(ref s) = name.value {
            task = get_task_by_pid(*s).await?;
        } else {
            return Ok(entity::Response {
                code: 1,
                msg: "Arg 'pid' must be a number".to_string(),
                data: None,
            });
        }
    } else {
        if command.args.len() == 0 {
            return Ok(entity::Response {
                code: 1,
                msg: "Arg 'name' or 'pid' is required".to_string(),
                data: None,
            });
        } else {
            task = get_task_by_name(command.args[0].clone()).await?;
        }
    }

    let pid = task.pid;
    let res = unsafe { kill(pid as i32, 15) };
    if res == 0 {
        update_status_by_name(task.name.clone(), "stopped".to_string()).await?;
        update_pid_by_name(task.name, 0).await?;
        let res = entity::Response {
            code: 10000,
            msg: "success".to_string(),
            data: None,
        };
        return Ok(res);
    }
    let res = entity::Response {
        code: 40000,
        msg: "failed".to_string(),
        data: None,
    };
    return Ok(res);
}
