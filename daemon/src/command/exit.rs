use std::error::Error;

use crate::{
    entity,
    global::{get_task_by_name, get_task_by_pid, remove_task_by_name},
};

extern "C" {
    pub fn kill(pid: i32, sig: i32) -> i32;
}

pub async fn exit_task(args: Vec<String>) -> Result<entity::Response, Box<dyn Error>> {
    let len = args.len();
    let task;
    if len == 1 {
        task = get_task_by_name(args[0].clone()).await?;
    } else if len == 2 {
        if args[0] == "-n" || args[0] == "--name" {
            task = get_task_by_name(args[1].clone()).await?;
        } else if args[0] == "-p" || args[0] == "--pid" {
            let pid = args[1].parse::<u32>();
            if pid.is_err() {
                let res = entity::Response {
                    code: 50000,
                    msg: format!("Invalid pid: '{}'", args[1]),
                    data: None,
                };
                return Ok(res);
            }
            task = get_task_by_pid(pid.unwrap()).await?;
        } else {
            let res = entity::Response {
                code: 50000,
                msg: format!("Invalid args"),
                data: None,
            };
            return Ok(res);
        }
    } else {
        let res = entity::Response {
            code: 50000,
            msg: format!("Invalid args"),
            data: None,
        };
        return Ok(res);
    }
    let pid = task.pid;
    let res = unsafe { kill(pid as i32, 15) };
    if res == 0 {
        remove_task_by_name(task.name).await?;
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
