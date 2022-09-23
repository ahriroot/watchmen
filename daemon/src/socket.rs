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
                origin: 0,
            };

            if command.options.contains_key("name") {
                let name = command.options.get("name").unwrap();
                match name.value {
                    entity::Opt::Str(ref s) => {
                        task.name = s.clone();
                    }
                    _ => {
                        task.name = "watchmen".to_string();
                    }
                }
            }

            if command.options.contains_key("origin") {
                let origin = command.options.get("origin").unwrap();
                match origin.value {
                    entity::Opt::U128(ref o) => {
                        task.origin = *o;
                    }
                    _ => {
                        task.origin = 0;
                    }
                }
            }

            if command.options.contains_key("interval") {
                let interval = command.options.get("interval").unwrap();
                match interval.value {
                    entity::Opt::U128(ref i) => {
                        task.interval = *i;
                    }
                    _ => {
                        task.interval = 0;
                    }
                }
            }

            task.command = command.args[0].clone();
            task.args = command.args[1..].to_vec();

            let result = command::run::run_task(task).await?;
            let res = entity::Response {
                code: result,
                msg: "success".to_string(),
                data: None,
            };
            return Ok(res);
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
