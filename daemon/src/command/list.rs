use std::error::Error;

use crate::{
    entity::{self, Task},
    global::get_all_tasks,
};

pub async fn list_tasks(command: entity::Command) -> Result<entity::Response, Box<dyn Error>> {
    let mut task_list: Vec<Task> = Vec::new();
    let tasks = get_all_tasks().await?;

    for task in tasks.iter() {
        let mut flag = true;
        for (key, value) in command.options.iter() {
            if key == "name" {
                match value.value {
                    entity::Opt::Str(ref s) => {
                        if !task.name.contains(s) {
                            flag = false;
                        }
                    }
                    _ => {}
                }
            }
            if key == "status" {
                match value.value {
                    entity::Opt::Str(ref s) => {
                        if !task.status.contains(s) {
                            flag = false;
                        }
                    }
                    _ => {}
                }
            }
            if key == "pid" {
                match value.value {
                    entity::Opt::U32(ref s) => {
                        if task.pid != *s {
                            flag = false;
                        }
                    }
                    _ => {}
                }
            }
        }
        if flag {
            task_list.push(task.clone());
        }
    }

    let res = entity::Response {
        code: 10000,
        msg: "success".to_string(),
        data: Some(entity::Data::TaskList(task_list)),
    };
    Ok(res)
}
