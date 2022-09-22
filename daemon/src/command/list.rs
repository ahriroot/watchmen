use std::error::Error;

use crate::{
    entity::{self, TaskItem},
    global::get_all_tasks,
};

pub async fn list_tasks(args: Vec<String>) -> Result<entity::Response, Box<dyn Error>> {
    let mut task_list: Vec<TaskItem> = Vec::new();
    let tasks = get_all_tasks().await?;
    let len = args.len();

    if len % 2 != 0 {
        let res = entity::Response {
            code: 50000,
            msg: format!("Invalid args length"),
            data: None,
        };
        return Ok(res);
    }

    let args_map = args
        .chunks(2)
        .map(|pair| (pair[0].clone(), pair[1].clone()))
        .collect::<std::collections::HashMap<String, String>>();

    let allowed_keys = vec!["-n", "--name", "-s", "--status", "-p", "--pid"];
    let keys = args_map.keys();
    for key in keys {
        if !allowed_keys.contains(&key.as_str()) {
            let res = entity::Response {
                code: 50000,
                msg: format!("Invalid arg key: '{}'", key),
                data: None,
            };
            return Ok(res);
        }
    }

    // -n --name 不能同时使用
    if args_map.contains_key("-n")
        && args_map.contains_key("--name")
        && args_map["-n"] != args_map["--name"]
    {
        let res = entity::Response {
            code: 50000,
            msg: format!("Invalid args: '-n' and '--name' cannot be used at the same time"),
            data: None,
        };
        return Ok(res);
    }

    // -s --status 不能同时使用
    if args_map.contains_key("-s")
        && args_map.contains_key("--status")
        && args_map["-s"] != args_map["--status"]
    {
        let res = entity::Response {
            code: 50000,
            msg: format!("Invalid args: '-s' and '--status' cannot be used at the same time"),
            data: None,
        };
        return Ok(res);
    }

    // -p --pid 不能同时使用
    if args_map.contains_key("-p")
        && args_map.contains_key("--pid")
        && args_map["-p"] != args_map["--pid"]
    {
        let res = entity::Response {
            code: 50000,
            msg: format!("Invalid args: '-p' and '--pid' cannot be used at the same time"),
            data: None,
        };
        return Ok(res);
    }

    if args_map.contains_key("-p") {
        if args_map.len() > 1 {
            let res = entity::Response {
                code: 50000,
                msg: format!(
                    "Incompatible arg '-p' and {}",
                    args_map
                        .keys()
                        .filter(|k| k != &"-p")
                        .map(|k| format!("'{}'", k))
                        .collect::<Vec<String>>()
                        .join(" or ")
                ),
                data: None,
            };
            return Ok(res);
        }
    }
    if args_map.contains_key("--pid") {
        if args_map.len() > 1 {
            let res = entity::Response {
                code: 50000,
                msg: format!(
                    "Incompatible arg '--pid' and {}",
                    args_map
                        .keys()
                        .filter(|k| k != &"-p")
                        .map(|k| format!("'{}'", k))
                        .collect::<Vec<String>>()
                        .join(" or ")
                ),
                data: None,
            };
            return Ok(res);
        }
    }

    for task in tasks.iter() {
        let mut flag = true;
        for (key, value) in args_map.iter() {
            if (key == "--name" || key == "-n") && !task.name.contains(value) {
                flag = false;
                break;
            }
            if (key == "--status" || key == "-s") && task.status != *value {
                flag = false;
                break;
            }
            if (key == "--pid" || key == "-p") && task.pid.to_string() != *value {
                flag = false;
                break;
            }
        }
        if flag {
            let task_item = TaskItem {
                name: task.name.clone(),
                status: task.status.clone(),
                pid: task.pid,
            };
            task_list.push(task_item);
        }
    }

    println!("{:?}", task_list);
    let res = entity::Response {
        code: 10000,
        msg: "success".to_string(),
        data: Some(entity::Data::TaskList(task_list)),
    };
    Ok(res)
}
