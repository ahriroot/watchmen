pub mod add;
pub mod exit;
pub mod list;
pub mod restart;
pub mod run;
pub mod start;
pub mod stop;

use std::{
    error::Error,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    command,
    entity::{self, Task},
    utils::get_id,
};

pub async fn handle_exec(command: entity::Command) -> Result<entity::Response, Box<dyn Error>> {
    match command.name.as_str() {
        "run" => {
            let id = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();
            let mut task: Task = Task {
                id: id,
                name: command.name,
                command: command.args[0].clone(),
                args: command.args[1..].to_vec(),
                status: "running".to_string(),
                pid: 0,
                created_at: 0,
                started_at: 0,
                exited_at: 0,
                stopped_at: 0,
                exit_code: 100,
                interval: id,
                origin: 0,
            };

            if command.options.contains_key("name") {
                let name = command.options.get("name").unwrap();
                match name {
                    entity::Opt::Str(ref s) => {
                        task.name = s.clone();
                    }
                    _ => {
                        return Ok(entity::Response {
                            code: 40000,
                            msg: "Arg 'name' must be a string".to_string(),
                            data: None,
                        });
                    }
                }
            } else {
                task.name = get_id().await;
            }

            if command.options.contains_key("origin") {
                let origin = command.options.get("origin").unwrap();
                match origin {
                    entity::Opt::U128(ref o) => {
                        task.origin = *o;
                    }
                    _ => {
                        return Ok(entity::Response {
                            code: 40000,
                            msg: "Arg 'origin' must be a number".to_string(),
                            data: None,
                        });
                    }
                }
            } else {
                task.origin = 0;
            }

            if command.options.contains_key("interval") {
                let interval = command.options.get("interval").unwrap();
                match interval {
                    entity::Opt::U128(ref i) => {
                        task.interval = *i;
                    }
                    _ => {
                        return Ok(entity::Response {
                            code: 40000,
                            msg: "Arg 'interval' must be a number".to_string(),
                            data: None,
                        });
                    }
                }
            } else {
                task.interval = 0;
            }

            task.command = command.args[0].clone();
            task.args = command.args[1..].to_vec();

            let result = command::run::run_task(task).await?;
            return Ok(result);
        }
        "add" => {
            let id = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();
            let mut task: Task = Task {
                id: id,
                name: command.name,
                command: command.args[0].clone(),
                args: command.args[1..].to_vec(),
                status: "added".to_string(),
                pid: 0,
                created_at: 0,
                started_at: 0,
                exited_at: 0,
                stopped_at: 0,
                exit_code: 100,
                interval: id,
                origin: 0,
            };

            if command.options.contains_key("name") {
                let name = command.options.get("name").unwrap();
                match name {
                    entity::Opt::Str(ref s) => {
                        task.name = s.clone();
                    }
                    _ => {
                        return Ok(entity::Response {
                            code: 40000,
                            msg: "Arg 'name' must be a string".to_string(),
                            data: None,
                        });
                    }
                }
            } else {
                task.name = get_id().await;
            }

            if command.options.contains_key("origin") {
                let origin = command.options.get("origin").unwrap();
                match origin {
                    entity::Opt::U128(ref o) => {
                        task.origin = *o;
                    }
                    _ => {
                        return Ok(entity::Response {
                            code: 40000,
                            msg: "Arg 'origin' must be a number".to_string(),
                            data: None,
                        });
                    }
                }
            } else {
                task.origin = 0;
            }

            if command.options.contains_key("interval") {
                let interval = command.options.get("interval").unwrap();
                match interval {
                    entity::Opt::U128(ref i) => {
                        task.interval = *i;
                    }
                    _ => {
                        return Ok(entity::Response {
                            code: 40000,
                            msg: "Arg 'interval' must be a number".to_string(),
                            data: None,
                        });
                    }
                }
            } else {
                task.interval = 0;
            }

            task.command = command.args[0].clone();
            task.args = command.args[1..].to_vec();

            let result = command::add::add_a_task(task).await?;
            return Ok(result);
        }
        "exit" => {
            let result = command::exit::exit_task(command).await;
            match result {
                Ok(res) => {
                    return Ok(res);
                }
                Err(e) => {
                    let res = entity::Response {
                        code: 40000,
                        msg: e.to_string(),
                        data: None,
                    };
                    return Ok(res);
                }
            }
        }
        "start" => {
            let result = command::start::start_task(command).await;
            match result {
                Ok(res) => {
                    return Ok(res);
                }
                Err(e) => {
                    let res = entity::Response {
                        code: 40000,
                        msg: e.to_string(),
                        data: None,
                    };
                    return Ok(res);
                }
            }
        }
        "restart" => {
            let result = command::restart::restart_task(command).await;
            match result {
                Ok(res) => {
                    return Ok(res);
                }
                Err(e) => {
                    let res = entity::Response {
                        code: 40000,
                        msg: e.to_string(),
                        data: None,
                    };
                    return Ok(res);
                }
            }
        }
        "stop" => {
            let result = command::stop::stop_task(command).await;
            match result {
                Ok(res) => {
                    return Ok(res);
                }
                Err(e) => {
                    let res = entity::Response {
                        code: 40000,
                        msg: e.to_string(),
                        data: None,
                    };
                    return Ok(res);
                }
            }
        }
        "list" => {
            let result = command::list::list_tasks(command).await;
            match result {
                Ok(res) => {
                    return Ok(res);
                }
                Err(e) => {
                    let res = entity::Response {
                        code: 40000,
                        msg: e.to_string(),
                        data: None,
                    };
                    return Ok(res);
                }
            }
        }
        _ => {
            println!("unknown command: {:?}", command);
            Ok(entity::Response {
                code: 10000,
                msg: "success".to_string(),
                data: None,
            })
        }
    }
}
