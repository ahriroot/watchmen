use std::{
    error::Error,
    fs::remove_file,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{UnixListener, UnixStream},
};

use crate::{
    command,
    entity::{self, Task},
};

async fn handle_exec(command: entity::Command) -> Result<entity::Response, Box<dyn Error>> {
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
                cycle: 0,
            };

            let mut args = command.args.clone();
            while args.len() > 1 {
                if args[0] == "-n" || args[0] == "--name" {
                    task.name = args[1].clone();
                } else if args[0] == "-c" || args[0] == "--cycle" {
                    let cycle = args[1].parse::<u128>();
                    match cycle {
                        Ok(c) => task.cycle = c,
                        Err(e) => {
                            return Ok(entity::Response {
                                code: 50000,
                                msg: format!("cycle must be a number, {}", e),
                                data: None,
                            })
                        }
                    }
                } else if args[0] == "-i" || args[0] == "--interval" {
                    let interval = args[1].parse::<u128>();
                    match interval {
                        Ok(i) => task.interval = i,
                        Err(e) => {
                            return Ok(entity::Response {
                                code: 50000,
                                msg: format!("interval must be a number, {}", e),
                                data: None,
                            })
                        }
                    }
                } else {
                    break;
                }
                args.remove(0);
                args.remove(0);
            }
            task.command = args[0].clone();
            task.args = args[1..].to_vec();

            let result = command::run::run_task(task).await?;
            let res = entity::Response {
                code: result,
                msg: "success".to_string(),
                data: None,
            };
            return Ok(res);
        }
        "exit" => {
            if command.args.len() == 0 {
                let res = entity::Response {
                    code: 40000,
                    msg: "args error".to_string(),
                    data: None,
                };
                return Ok(res);
            } else {
                let result = command::exit::exit_task(command.args.clone()).await;
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
        }
        "start" => {
            if command.args.len() == 0 {
                let res = entity::Response {
                    code: 40000,
                    msg: "args error".to_string(),
                    data: None,
                };
                return Ok(res);
            } else {
                let result = command::start::start_task(command.args.clone()).await;
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
        }
        "stop" => {
            if command.args.len() == 0 {
                let res = entity::Response {
                    code: 40000,
                    msg: "args error".to_string(),
                    data: None,
                };
                return Ok(res);
            } else {
                let result = command::stop::stop_task(command.args.clone()).await;
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
        }
        "list" => {
            let result = command::list::list_tasks(command.args.clone()).await;
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

async fn handle_connection(mut stream: UnixStream) -> Result<(), Box<dyn Error>> {
    let (mut reader, mut writer) = stream.split();
    let mut buf: [u8; 1024] = [0; 1024];
    let n = reader.read(&mut buf).await?;

    let req: entity::Request = serde_json::from_slice(&buf[..n])?;

    let res = handle_exec(req.command).await?;

    writer.write_all(&serde_json::to_vec(&res)?).await?;
    Ok(())
}

pub async fn run_socket(path: &String) -> Result<(), Box<dyn Error>> {
    let sock_path = Path::new(path);
    remove_file(sock_path).unwrap_or_default();

    // 创建监听器 / create a listener
    let listener = UnixListener::bind(path).unwrap();
    println!("Listening on {:?}", listener.local_addr().unwrap());

    loop {
        // 等待连接 / wait connection
        match listener.accept().await {
            Ok((stream, _addr)) => {
                // 处理连接 / handle connection
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(stream).await {
                        eprintln!("failed to process connection: {}", e);
                    }
                });
            }
            Err(e) => {
                println!("failed to accept socket; error = {:?}", e);
            }
        }
    }
}
